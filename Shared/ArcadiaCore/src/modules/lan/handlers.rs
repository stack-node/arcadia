use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

use crate::modules::ExecutionContext;

use super::config::{
    aliases_for_identifier, load_node_config, normalize_node_identifier, resolve_alias_target,
    save_node_config,
};
use super::discovery::{local_hostname, resolve_target};
use super::peers::{match_peer_key, node_state, record_peer, NodeState};
use super::protocol::{
    DISCOVERY_PORT, NODE_ACCEPT_PREFIX, NODE_CONNECT_PREFIX, NODE_REJECT_PREFIX, PeerStatus,
};

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
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

fn node_alias(target: &str, alias_parts: &[&str]) -> String {
    if alias_parts.is_empty() {
        return "Usage: lan.node alias <host/ip/alias> <custom alias>".to_string();
    }
    let custom_alias = alias_parts.join(" ");
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

pub fn node(args: &[&str], _context: &ExecutionContext) -> String {
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
