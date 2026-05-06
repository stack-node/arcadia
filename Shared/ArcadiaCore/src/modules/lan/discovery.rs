use std::collections::BTreeMap;
use std::env;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::config::modules::{LAN_MODULE_NAME, ModulesConfig};
use crate::config::ConfigFile;
use crate::modules::ExecutionContext;

use super::config::{is_auto_allowed, is_identifier_approved, load_node_config};
use super::peers::{node_state, record_peer};
use super::protocol::{
    DISCOVERY_PORT, DISCOVERY_REQUEST, DISCOVERY_RESPONSE_PREFIX, NODE_ACCEPT_PREFIX,
    NODE_CONNECT_PREFIX, NODE_EXEC_PREFIX, NODE_EXEC_RESULT_PREFIX, NODE_REJECT_PREFIX,
    RECV_BUF_SMALL, SCAN_WAIT_MS,
};
use super::protocol::PeerStatus;

pub static SERVICE_RUNNING: AtomicBool = AtomicBool::new(false);
static SERVICE_THREAD: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();

fn service_thread_slot() -> &'static Mutex<Option<JoinHandle<()>>> {
    SERVICE_THREAD.get_or_init(|| Mutex::new(None))
}

pub fn lan_enabled() -> bool {
    ModulesConfig::load_or_create()
        .ok()
        .and_then(|cfg| cfg.modules.get(LAN_MODULE_NAME).copied())
        .unwrap_or(false)
}

pub fn local_hostname() -> String {
    env::var("HOSTNAME")
        .or_else(|_| env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown-host".to_string())
}

pub fn resolve_target(target: &str) -> Result<SocketAddrV4, String> {
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

pub fn parse_cidr_target(value: &str) -> Option<SocketAddrV4> {
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

pub fn parse_targets(range: Option<&str>) -> Result<Vec<SocketAddrV4>, String> {
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

pub fn scan(args: &[&str], _context: &ExecutionContext) -> String {
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
    let mut buf = [0_u8; RECV_BUF_SMALL];
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
        let Ok(socket) =
            UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DISCOVERY_PORT))
        else {
            SERVICE_RUNNING.store(false, Ordering::SeqCst);
            return;
        };
        let _ = socket.set_read_timeout(Some(Duration::from_millis(200)));

        let mut buf = [0_u8; RECV_BUF_SMALL];
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
                    Ok(g) => g,
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
                let owned_args = parts.map(|v| v.to_string()).collect::<Vec<_>>();
                let args = owned_args.iter().map(String::as_str).collect::<Vec<_>>();
                let context = crate::modules::ExecutionContext::default();
                drop(guard);
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
