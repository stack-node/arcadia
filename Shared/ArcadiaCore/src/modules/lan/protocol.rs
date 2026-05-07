pub const DISCOVERY_PORT: u16 = 46291;
pub const DISCOVERY_REQUEST: &str = "ARCADIA_LAN_DISCOVER_V1";
pub const DISCOVERY_RESPONSE_PREFIX: &str = "ARCADIA_LAN_HERE_V1";
pub const NODE_CONNECT_PREFIX: &str = "ARCADIA_NODE_CONNECT_V1";
pub const NODE_ACCEPT_PREFIX: &str = "ARCADIA_NODE_ACCEPT_V1";
pub const NODE_REJECT_PREFIX: &str = "ARCADIA_NODE_REJECT_V1";
pub const NODE_EXEC_PREFIX: &str = "ARCADIA_NODE_EXEC_V1";
pub const NODE_EXEC_RESULT_PREFIX: &str = "ARCADIA_NODE_EXEC_RESULT_V1";
pub const SCAN_WAIT_MS: u64 = 800;
pub const DEFAULT_REMOTE_TIMEOUT_MS: u64 = 2_000;
pub const RECV_BUF_SMALL: usize = 1024;
pub const RECV_BUF_LARGE: usize = 65_507;

#[derive(Clone, Copy)]
pub enum PeerStatus {
    PendingInbound,
    PendingOutbound,
    Connected,
    Rejected,
}

impl PeerStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PendingInbound => "pending-inbound",
            Self::PendingOutbound => "pending-outbound",
            Self::Connected => "connected",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone)]
pub struct PeerRecord {
    pub ip: String,
    pub hostname: String,
    pub status: PeerStatus,
}
