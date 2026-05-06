use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use super::config::normalize_node_identifier;
use super::protocol::{PeerRecord, PeerStatus};

#[derive(Default)]
pub struct NodeState {
    pub peers: BTreeMap<String, PeerRecord>,
}

static NODE_STATE: OnceLock<Mutex<NodeState>> = OnceLock::new();

pub fn node_state() -> &'static Mutex<NodeState> {
    NODE_STATE.get_or_init(|| Mutex::new(NodeState::default()))
}

pub fn record_peer(ip: String, hostname: String, status: PeerStatus) {
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

pub fn match_peer_key(identifier: &str, peers: &BTreeMap<String, PeerRecord>) -> Option<String> {
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
