pub const NAME: &str = "lan";

mod config;
mod discovery;
mod handlers;
mod peers;
mod protocol;

use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::Duration;

use crate::modules::ModuleCommand;

use config::{is_identifier_approved, load_node_config, resolve_alias_target};
use discovery::resolve_target;
use peers::node_state;
use protocol::{
    DEFAULT_REMOTE_TIMEOUT_MS, DISCOVERY_PORT, NODE_EXEC_PREFIX, NODE_EXEC_RESULT_PREFIX,
    RECV_BUF_LARGE,
};
use protocol::PeerStatus;

pub use discovery::{start_service, stop_service};

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

    let mut buf = [0_u8; RECV_BUF_LARGE];
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

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "scan",
            description: "discover Arcadia LAN peers (--range supported)",
            run: discovery::scan,
        },
        ModuleCommand {
            name: "node",
            description: "manage LAN nodes: pair|connect|accept|reject|alias|save|auto|status",
            run: handlers::node,
        },
    ]
}
