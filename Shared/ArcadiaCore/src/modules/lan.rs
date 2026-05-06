pub const NAME: &str = "lan";

use std::collections::BTreeMap;
use std::env;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::config::modules::{ModulesConfig, LAN_MODULE_NAME};
use crate::config::ConfigFile;
use crate::modules::{ExecutionContext, ModuleCommand};
use serde::{Deserialize, Serialize};

const DISCOVERY_PORT: u16 = 46291;
const DISCOVERY_REQUEST: &str = "ARCADIA_LAN_DISCOVER_V1";
const DISCOVERY_RESPONSE_PREFIX: &str = "ARCADIA_LAN_HERE_V1";
const NODE_CONNECT_PREFIX: &str = "ARCADIA_NODE_CONNECT_V1";
const NODE_ACCEPT_PREFIX: &str = "ARCADIA_NODE_ACCEPT_V1";
const NODE_REJECT_PREFIX: &str = "ARCADIA_NODE_REJECT_V1";
const NODE_EXEC_PREFIX: &str = "ARCADIA_NODE_EXEC_V1";
const NODE_EXEC_RESULT_PREFIX: &str = "ARCADIA_NODE_EXEC_RESULT_V1";
const SCAN_WAIT_MS: u64 = 800;
const NODE_CONFIG_FILE_NAME: &str = "lan_nodes.toml";
const DEFAULT_REMOTE_TIMEOUT_MS: u64 = 2_000;

static SERVICE_RUNNING: AtomicBool = AtomicBool::new(false);
static SERVICE_THREAD: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();
static NODE_STATE: OnceLock<Mutex<NodeState>> = OnceLock::new();

#[derive(Clone, Copy)]
enum PeerStatus {
    PendingInbound,
    PendingOutbound,
    Connected,
    Rejected,
}

impl PeerStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::PendingInbound => "pending-inbound",
            Self::PendingOutbound => "pending-outbound",
            Self::Connected => "connected",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone)]
struct PeerRecord {
    ip: String,
    hostname: String,
    status: PeerStatus,
}

#[derive(Default)]
struct NodeState {
    peers: BTreeMap<String, PeerRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanNodeConfig {
    auto: bool,
    approved_nodes: Vec<String>,
    node_rules: BTreeMap<String, bool>,
    aliases: BTreeMap<String, String>,
}

impl Default for LanNodeConfig {
    fn default() -> Self {
        Self {
            auto: false,
            approved_nodes: Vec::new(),
            node_rules: BTreeMap::new(),
            aliases: BTreeMap::new(),
        }
    }
}

impl ConfigFile for LanNodeConfig {
    fn file_name() -> &'static str {
        NODE_CONFIG_FILE_NAME
    }
}

fn service_thread_slot() -> &'static Mutex<Option<JoinHandle<()>>> {
    SERVICE_THREAD.get_or_init(|| Mutex::new(None))
}

fn node_state() -> &'static Mutex<NodeState> {
    NODE_STATE.get_or_init(|| Mutex::new(NodeState::default()))
}

fn record_peer(ip: String, hostname: String, status: PeerStatus) {
    if let Ok(mut guard) = node_state().lock() {
        let record = guard.peers.entry(ip.clone()).or_insert(PeerRecord {
            ip,
            hostname: hostname.clone(),
            status,
        });
        record.hostname = hostname;
        record.status = status;
    }
}

fn normalize_node_identifier(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn load_node_config() -> Result<LanNodeConfig, String> {
    LanNodeConfig::load_or_create().map_err(|err| err.to_string())
}

fn save_node_config(config: &LanNodeConfig) -> Result<(), String> {
    config.save().map_err(|err| err.to_string())
}

fn resolve_identifier_for_config(target: &str, state: &NodeState) -> String {
    if let Some(key) = match_peer_key(target, &state.peers) {
        return normalize_node_identifier(&key);
    }
    if let Ok(addr) = resolve_target(target) {
        return normalize_node_identifier(&addr.ip().to_string());
    }
    normalize_node_identifier(target)
}

fn resolve_alias_target(target: &str, cfg: &LanNodeConfig) -> String {
    let key = normalize_node_identifier(target);
    cfg.aliases.get(&key).cloned().unwrap_or(key)
}

fn aliases_for_identifier(identifier: &str, cfg: &LanNodeConfig) -> Vec<String> {
    let id = normalize_node_identifier(identifier);
    cfg.aliases
        .iter()
        .filter_map(|(alias, mapped)| {
            if normalize_node_identifier(mapped) == id {
                Some(alias.clone())
            } else {
                None
            }
        })
        .collect()
}

fn is_auto_allowed(ip: &str, hostname: &str) -> bool {
    let Ok(cfg) = load_node_config() else {
        return false;
    };
    if !cfg.auto {
        return false;
    }

    let ip_key = normalize_node_identifier(ip);
    let host_key = normalize_node_identifier(hostname);
    if let Some(value) = cfg
        .node_rules
        .get(&ip_key)
        .or_else(|| cfg.node_rules.get(&host_key))
    {
        return *value;
    }

    cfg.approved_nodes.iter().any(|node| {
        let key = normalize_node_identifier(node);
        key == ip_key || key == host_key
    })
}

fn is_identifier_approved(cfg: &LanNodeConfig, ip: &str, hostname: &str) -> bool {
    let ip_key = normalize_node_identifier(ip);
    let host_key = normalize_node_identifier(hostname);
    cfg.approved_nodes.iter().any(|node| {
        let key = normalize_node_identifier(node);
        key == ip_key || key == host_key
    })
}

fn match_peer_key(identifier: &str, peers: &BTreeMap<String, PeerRecord>) -> Option<String> {
    let key = normalize_node_identifier(identifier);
    if let Some((ip, _)) = peers
        .iter()
        .find(|(ip, _)| normalize_node_identifier(ip) == key)
    {
        return Some(ip.clone());
    }
    peers
        .iter()
        .find(|(_, peer)| normalize_node_identifier(&peer.hostname) == key)
        .map(|(ip, _)| ip.clone())
}

pub fn start_service() {
    if SERVICE_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    let mut slot = match service_thread_slot().lock() {
        Ok(guard) => guard,
        Err(_) => {
            SERVICE_RUNNING.store(false, Ordering::SeqCst);
            return;
        }
    };

    let handle = thread::spawn(|| {
        let Ok(socket) = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DISCOVERY_PORT))
        else {
            SERVICE_RUNNING.store(false, Ordering::SeqCst);
            return;
        };
        let _ = socket.set_read_timeout(Some(Duration::from_millis(200)));

        let mut buf = [0_u8; 1024];
        while SERVICE_RUNNING.load(Ordering::SeqCst) {
            let Ok((len, src)) = socket.recv_from(&mut buf) else {
                continue;
            };

            let payload = String::from_utf8_lossy(&buf[..len]);
            if payload.trim() == DISCOVERY_REQUEST {
                if !lan_enabled() {
                    continue;
                }
                let hostname = local_hostname();
                let response = format!("{DISCOVERY_RESPONSE_PREFIX}\t{hostname}");
                let _ = socket.send_to(response.as_bytes(), src);
                continue;
            }

            let Some((prefix, remote_hostname)) = payload.split_once('\t') else {
                continue;
            };
            let SocketAddr::V4(src_v4) = src else {
                continue;
            };
            let ip = src_v4.ip().to_string();
            let remote_hostname = remote_hostname.trim().to_string();

            if prefix == NODE_CONNECT_PREFIX {
                if lan_enabled() {
                    if is_auto_allowed(&ip, &remote_hostname) {
                        record_peer(ip.clone(), remote_hostname.clone(), PeerStatus::Connected);
                        let response = format!("{NODE_ACCEPT_PREFIX}\t{}", local_hostname());
                        let _ = socket.send_to(response.as_bytes(), src);
                    } else {
                        record_peer(ip, remote_hostname, PeerStatus::PendingInbound);
                    }
                }
                continue;
            }
            if prefix == NODE_ACCEPT_PREFIX {
                if lan_enabled() {
                    record_peer(ip, remote_hostname, PeerStatus::Connected);
                }
                continue;
            }
            if prefix == NODE_REJECT_PREFIX {
                if lan_enabled() {
                    record_peer(ip, remote_hostname, PeerStatus::Rejected);
                }
                continue;
            }
            if prefix == NODE_EXEC_PREFIX {
                if !lan_enabled() {
                    continue;
                }
                let cfg = load_node_config().unwrap_or_default();
                let guard = match node_state().lock() {
                    Ok(guard) => guard,
                    Err(_) => continue,
                };
                let Some(peer) = guard.peers.get(&ip) else {
                    continue;
                };
                if !matches!(peer.status, PeerStatus::Connected)
                    || !is_identifier_approved(&cfg, &ip, &peer.hostname)
                {
                    continue;
                }

                let mut parts = remote_hostname.split('\t');
                let Some(token) = parts.next() else {
                    continue;
                };
                let owned_args = parts.map(|value| value.to_string()).collect::<Vec<_>>();
                let args = owned_args.iter().map(String::as_str).collect::<Vec<_>>();
                let context = crate::modules::ExecutionContext::default();
                let result = match crate::modules::execute_command(token, &args, &context) {
                    Ok(Some(message)) => message,
                    Ok(None) => format!("Unknown remote command: {token}"),
                    Err(err) => err,
                };
                let response = format!("{NODE_EXEC_RESULT_PREFIX}\t{result}");
                let _ = socket.send_to(response.as_bytes(), src);
            }
        }
    });
    *slot = Some(handle);
}

pub fn stop_service() {
    SERVICE_RUNNING.store(false, Ordering::SeqCst);
    if let Ok(mut slot) = service_thread_slot().lock() {
        if let Some(handle) = slot.take() {
            let _ = handle.join();
        }
    }
}

fn lan_enabled() -> bool {
    ModulesConfig::load_or_create()
        .ok()
        .and_then(|cfg| cfg.modules.get(LAN_MODULE_NAME).copied())
        .unwrap_or(false)
}

fn local_hostname() -> String {
    env::var("HOSTNAME")
        .or_else(|_| env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown-host".to_string())
}

fn parse_cidr_target(value: &str) -> Option<SocketAddrV4> {
    let (ip_part, prefix_part) = value.split_once('/')?;
    let ip = ip_part.parse::<Ipv4Addr>().ok()?;
    let prefix = prefix_part.parse::<u32>().ok()?;
    if prefix > 32 {
        return None;
    }

    let ip_u32 = u32::from(ip);
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    let broadcast = Ipv4Addr::from(ip_u32 | !mask);
    Some(SocketAddrV4::new(broadcast, DISCOVERY_PORT))
}

fn parse_targets(range: Option<&str>) -> Result<Vec<SocketAddrV4>, String> {
    match range {
        None => Ok(vec![SocketAddrV4::new(
            Ipv4Addr::new(255, 255, 255, 255),
            DISCOVERY_PORT,
        )]),
        Some(value) if value.contains('/') => {
            let Some(target) = parse_cidr_target(value) else {
                return Err("Invalid --range CIDR value".to_string());
            };
            Ok(vec![target])
        }
        Some(value) => {
            let ip = value
                .parse::<Ipv4Addr>()
                .map_err(|_| "Invalid --range IP value".to_string())?;
            Ok(vec![SocketAddrV4::new(ip, DISCOVERY_PORT)])
        }
    }
}

fn scan(args: &[&str], _context: &ExecutionContext) -> String {
    let mut range: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--range" => {
                let Some(value) = args.get(i + 1) else {
                    return "Usage: lan.scan [--range <CIDR-or-ip>]".to_string();
                };
                range = Some(*value);
                i += 2;
            }
            unknown => {
                return format!(
                    "Unknown argument: {unknown}. Usage: lan.scan [--range <CIDR-or-ip>]"
                );
            }
        }
    }

    let Ok(targets) = parse_targets(range) else {
        return "Invalid range. Use --range <CIDR> (e.g. 192.168.1.0/24) or --range <IP>"
            .to_string();
    };

    let Ok(socket) = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)) else {
        return "Failed to bind UDP socket for lan.scan".to_string();
    };
    if socket.set_broadcast(true).is_err() {
        return "Failed to enable UDP broadcast for lan.scan".to_string();
    }
    let _ = socket.set_read_timeout(Some(Duration::from_millis(100)));

    for target in targets {
        let _ = socket.send_to(DISCOVERY_REQUEST.as_bytes(), target);
    }

    let deadline = Instant::now() + Duration::from_millis(SCAN_WAIT_MS);
    let mut peers = BTreeMap::<String, String>::new();
    let mut buf = [0_u8; 1024];
    while Instant::now() < deadline {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                let payload = String::from_utf8_lossy(&buf[..len]);
                let Some((prefix, hostname)) = payload.split_once('\t') else {
                    continue;
                };
                if prefix != DISCOVERY_RESPONSE_PREFIX {
                    continue;
                }
                if let SocketAddr::V4(addr_v4) = src {
                    peers.insert(addr_v4.ip().to_string(), hostname.trim().to_string());
                }
            }
            Err(_) => continue,
        }
    }

    if peers.is_empty() {
        return "No Arcadia peers found with LAN module enabled".to_string();
    }

    let mut lines = vec!["Arcadia LAN peers:".to_string()];
    for (ip, hostname) in peers {
        lines.push(format!("- {ip} ({hostname})"));
    }
    lines.join("\n")
}

fn resolve_target(target: &str) -> Result<SocketAddrV4, String> {
    let mut resolved = (target, DISCOVERY_PORT)
        .to_socket_addrs()
        .map_err(|err| format!("Failed to resolve target {target}: {err}"))?;
    for addr in resolved.by_ref() {
        if let SocketAddr::V4(v4) = addr {
            return Ok(v4);
        }
    }
    Err(format!("Target {target} did not resolve to IPv4"))
}

pub fn execute_remote_command(
    target: &str,
    token: &str,
    args: &[&str],
    timeout_ms: Option<u64>,
) -> Result<String, String> {
    let cfg = load_node_config().map_err(|_| "Failed to load LAN node config".to_string())?;
    let resolved_target = resolve_alias_target(target, &cfg);
    let addr = resolve_target(&resolved_target)?;
    let ip = addr.ip().to_string();

    let guard = node_state()
        .lock()
        .map_err(|_| "Failed to access node state".to_string())?;
    let Some(peer) = guard.peers.get(&ip) else {
        return Err(format!("Target {target} is not a known node"));
    };
    if !matches!(peer.status, PeerStatus::Connected) {
        return Err(format!("Target {target} is not connected"));
    }
    if !is_identifier_approved(&cfg, &peer.ip, &peer.hostname) {
        return Err(format!("Target {target} is not approved"));
    }
    drop(guard);

    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
        .map_err(|err| format!("Failed to create UDP socket: {err}"))?;
    let _ = socket.set_read_timeout(Some(Duration::from_millis(
        timeout_ms.unwrap_or(DEFAULT_REMOTE_TIMEOUT_MS),
    )));

    let mut payload = format!("{NODE_EXEC_PREFIX}\t{token}");
    for arg in args {
        payload.push('\t');
        payload.push_str(arg);
    }

    socket
        .send_to(
            payload.as_bytes(),
            SocketAddrV4::new(*addr.ip(), DISCOVERY_PORT),
        )
        .map_err(|err| format!("Failed to send remote command: {err}"))?;

    let mut buf = [0_u8; 65_507];
    let (len, _) = socket
        .recv_from(&mut buf)
        .map_err(|_| "Remote command timed out".to_string())?;
    let payload = String::from_utf8_lossy(&buf[..len]);
    let Some((prefix, result)) = payload.split_once('\t') else {
        return Err("Invalid remote response".to_string());
    };
    if prefix != NODE_EXEC_RESULT_PREFIX {
        return Err("Unexpected remote response".to_string());
    }
    Ok(result.to_string())
}

fn node_connect(target: &str) -> String {
    let cfg = load_node_config().unwrap_or_default();
    let resolved_target = resolve_alias_target(target, &cfg);
    let Ok(addr) = resolve_target(&resolved_target) else {
        return format!("Failed to connect: unable to resolve {target}");
    };
    let Ok(socket) = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)) else {
        return "Failed to create UDP socket for node connect".to_string();
    };
    let payload = format!("{NODE_CONNECT_PREFIX}\t{}", local_hostname());
    if socket.send_to(payload.as_bytes(), addr).is_err() {
        return format!("Failed to send node connect request to {}", addr.ip());
    }
    record_peer(
        addr.ip().to_string(),
        resolved_target,
        PeerStatus::PendingOutbound,
    );
    format!(
        "Connection request sent to {}. Waiting for lan.node accept on peer.",
        addr.ip()
    )
}

fn node_accept(target: &str) -> String {
    let cfg = load_node_config().unwrap_or_default();
    let resolved_target = resolve_alias_target(target, &cfg);
    let (peer_ip, peer_hostname) = {
        let Ok(mut guard) = node_state().lock() else {
            return "Failed to access node state".to_string();
        };
        let Some(key) = match_peer_key(&resolved_target, &guard.peers) else {
            return format!("No known peer matching {target}");
        };
        let Some(peer) = guard.peers.get_mut(&key) else {
            return format!("No known peer matching {target}");
        };
        if !matches!(
            peer.status,
            PeerStatus::PendingInbound | PeerStatus::PendingOutbound
        ) {
            return format!("Peer {} is already {}", peer.ip, peer.status.as_str());
        }
        peer.status = PeerStatus::Connected;
        (peer.ip.clone(), peer.hostname.clone())
    };

    let Ok(ip) = peer_ip.parse::<Ipv4Addr>() else {
        return format!("Cannot accept peer with invalid IP {peer_ip}");
    };
    let Ok(socket) = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)) else {
        return "Failed to create UDP socket for node accept".to_string();
    };
    let payload = format!("{NODE_ACCEPT_PREFIX}\t{}", local_hostname());
    let _ = socket.send_to(payload.as_bytes(), SocketAddrV4::new(ip, DISCOVERY_PORT));
    format!("Accepted node {peer_hostname} ({peer_ip})")
}

fn node_status(target: Option<&str>) -> String {
    let cfg = load_node_config().unwrap_or_default();
    let Ok(guard) = node_state().lock() else {
        return "Failed to access node state".to_string();
    };
    if guard.peers.is_empty() {
        return "No known LAN nodes".to_string();
    }

    if let Some(identifier) = target {
        let resolved = resolve_alias_target(identifier, &cfg);
        let Some(key) = match_peer_key(&resolved, &guard.peers) else {
            return format!("No node found for {identifier}");
        };
        let peer = &guard.peers[&key];
        let aliases = aliases_for_identifier(&peer.ip, &cfg);
        if aliases.is_empty() {
            return format!(
                "{} ({}) -> {}",
                peer.hostname,
                peer.ip,
                peer.status.as_str()
            );
        }
        return format!(
            "{} ({}) [{}] -> {}",
            peer.hostname,
            peer.ip,
            aliases.join(", "),
            peer.status.as_str()
        );
    }

    let mut lines = vec!["LAN node status:".to_string()];
    for peer in guard.peers.values() {
        let aliases = aliases_for_identifier(&peer.ip, &cfg);
        let alias_suffix = if aliases.is_empty() {
            String::new()
        } else {
            format!(" [{}]", aliases.join(", "))
        };
        lines.push(format!(
            "- {} ({}){} -> {}",
            peer.hostname,
            peer.ip,
            alias_suffix,
            peer.status.as_str()
        ));
    }
    lines.join("\n")
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn node_save(target: Option<&str>) -> String {
    let Ok(mut cfg) = load_node_config() else {
        return "Failed to load LAN node config".to_string();
    };
    let Ok(guard) = node_state().lock() else {
        return "Failed to access node state".to_string();
    };

    let mut added = Vec::new();
    match target {
        None => {
            for peer in guard.peers.values() {
                if !matches!(peer.status, PeerStatus::Connected) {
                    continue;
                }
                let ip_key = normalize_node_identifier(&peer.ip);
                if !cfg.approved_nodes.contains(&ip_key) {
                    cfg.approved_nodes.push(ip_key.clone());
                    added.push(ip_key);
                }
            }
        }
        Some(identifier) => {
            let target_value = resolve_alias_target(identifier, &cfg);
            let resolved = resolve_identifier_for_config(&target_value, &guard);
            if !cfg.approved_nodes.contains(&resolved) {
                cfg.approved_nodes.push(resolved.clone());
                added.push(resolved);
            }
        }
    }

    if save_node_config(&cfg).is_err() {
        return "Failed to save LAN node config".to_string();
    }
    if added.is_empty() {
        "No new node entries were added".to_string()
    } else {
        format!("Saved node entries: {}", added.join(", "))
    }
}

fn node_set_auto(value: &str) -> String {
    let Some(enabled) = parse_bool(value) else {
        return "Usage: lan.node auto <true|false>".to_string();
    };
    let Ok(mut cfg) = load_node_config() else {
        return "Failed to load LAN node config".to_string();
    };
    cfg.auto = enabled;
    if save_node_config(&cfg).is_err() {
        return "Failed to save LAN node config".to_string();
    }
    format!("LAN node auto mode set to {enabled}")
}

fn node_set_rule(target: &str, value: &str) -> String {
    let Some(allowed) = parse_bool(value) else {
        return "Usage: lan.node <host/ip> <true|false>".to_string();
    };
    let Ok(mut cfg) = load_node_config() else {
        return "Failed to load LAN node config".to_string();
    };
    if !cfg.auto {
        return "lan.node <host/ip> <true|false> requires lan.node auto true".to_string();
    }

    let Ok(guard) = node_state().lock() else {
        return "Failed to access node state".to_string();
    };
    let target_value = resolve_alias_target(target, &cfg);
    let key = resolve_identifier_for_config(&target_value, &guard);
    cfg.node_rules.insert(key.clone(), allowed);
    if save_node_config(&cfg).is_err() {
        return "Failed to save LAN node config".to_string();
    }
    format!("Rule set: {key} -> {allowed}")
}

fn node_reject(target: &str) -> String {
    let cfg = load_node_config().unwrap_or_default();
    let resolved_target = resolve_alias_target(target, &cfg);
    let (peer_ip, peer_hostname) = {
        let Ok(mut guard) = node_state().lock() else {
            return "Failed to access node state".to_string();
        };
        let Some(key) = match_peer_key(&resolved_target, &guard.peers) else {
            return format!("No known peer matching {target}");
        };
        let Some(peer) = guard.peers.get(&key) else {
            return format!("No known peer matching {target}");
        };
        let peer_ip = peer.ip.clone();
        let peer_hostname = peer.hostname.clone();
        guard.peers.remove(&key);
        (peer_ip, peer_hostname)
    };

    let Ok(ip) = peer_ip.parse::<Ipv4Addr>() else {
        return format!("Rejected node {peer_hostname} ({peer_ip})");
    };
    if let Ok(socket) = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)) {
        let payload = format!("{NODE_REJECT_PREFIX}\t{}", local_hostname());
        let _ = socket.send_to(payload.as_bytes(), SocketAddrV4::new(ip, DISCOVERY_PORT));
    }
    format!("Rejected node {peer_hostname} ({peer_ip})")
}

fn node_alias(target: &str, custom_alias_parts: &[&str]) -> String {
    if custom_alias_parts.is_empty() {
        return "Usage: lan.node alias <host/ip/alias> <custom alias>".to_string();
    }
    let custom_alias = custom_alias_parts.join(" ");
    let alias_key = normalize_node_identifier(&custom_alias);
    if alias_key.is_empty() {
        return "Alias cannot be empty".to_string();
    }

    let Ok(mut cfg) = load_node_config() else {
        return "Failed to load LAN node config".to_string();
    };
    let Ok(guard) = node_state().lock() else {
        return "Failed to access node state".to_string();
    };
    let target_value = resolve_alias_target(target, &cfg);
    let resolved = resolve_identifier_for_config(&target_value, &guard);
    cfg.aliases.insert(alias_key.clone(), resolved.clone());
    if save_node_config(&cfg).is_err() {
        return "Failed to save LAN node config".to_string();
    }

    format!("Alias set: {alias_key} -> {resolved}")
}

fn node_pair(target: &str) -> String {
    let cfg = load_node_config().unwrap_or_default();
    let resolved_target = resolve_alias_target(target, &cfg);

    let maybe_inbound = {
        let Ok(guard) = node_state().lock() else {
            return "Failed to access node state".to_string();
        };
        match_peer_key(&resolved_target, &guard.peers)
            .and_then(|key| guard.peers.get(&key).cloned())
            .filter(|peer| matches!(peer.status, PeerStatus::PendingInbound))
    };

    if let Some(peer) = maybe_inbound {
        let accepted = node_accept(&peer.ip);
        let saved = node_save(Some(&peer.ip));
        return format!("{accepted}\n{saved}");
    }

    let connected = node_connect(&resolved_target);
    let saved = node_save(Some(&resolved_target));
    format!("{connected}\n{saved}")
}

fn node(args: &[&str], _context: &ExecutionContext) -> String {
    match args {
        ["pair", target] => node_pair(target),
        ["connect", target] => node_connect(target),
        ["accept", target] => node_accept(target),
        ["reject", target] => node_reject(target),
        ["alias", target, alias_parts @ ..] => node_alias(target, alias_parts),
        ["save"] => node_save(None),
        ["save", target] => node_save(Some(target)),
        ["auto", value] => node_set_auto(value),
        ["status"] => node_status(None),
        ["status", target] => node_status(Some(target)),
        [target, value] => node_set_rule(target, value),
        _ => "Usage: lan.node pair <hostname/ip/alias> | lan.node connect <hostname/ip> | lan.node accept <hostname/ip> | lan.node reject <hostname/ip> | lan.node alias <hostname/ip/alias> <custom alias> | lan.node save [hostname/ip] | lan.node auto <true|false> | lan.node status [hostname/ip] | lan.node <hostname/ip> <true|false>".to_string(),
    }
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "scan",
            description: "discover Arcadia LAN peers (--range supported)",
            run: scan,
        },
        ModuleCommand {
            name: "node",
            description: "manage LAN nodes: pair|connect|accept|reject|alias|save|auto|status",
            run: node,
        },
    ]
}
