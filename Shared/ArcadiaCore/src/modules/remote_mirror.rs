//! After **every** inbound LAN NODE_EXEC completes (`execute_command` on the host), enqueue token + args + result here.
//! Same path for all modules — not shell-specific. Surfaces drain formatted lines into their transcript / log UI.
//!
//! Also raises [`request_host_ui_sync_after_peer_exec`] so local shells can reload module/nav state from disk.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

const CAP: usize = 256;

static HOST_UI_SYNC_PENDING: AtomicBool = AtomicBool::new(false);

/// Peer NODE_EXEC finished on this host — surfaces should reload host-backed UI when showing local state.
pub fn request_host_ui_sync_after_peer_exec() {
    HOST_UI_SYNC_PENDING.store(true, Ordering::SeqCst);
}

pub fn take_host_ui_sync_pending() -> bool {
    HOST_UI_SYNC_PENDING.swap(false, Ordering::SeqCst)
}

pub struct RemoteMirrorEvent {
    pub token: String,
    pub args: Vec<String>,
    pub output: String,
}

fn queue() -> &'static Mutex<VecDeque<RemoteMirrorEvent>> {
    static Q: OnceLock<Mutex<VecDeque<RemoteMirrorEvent>>> = OnceLock::new();
    Q.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn enqueue_remote_exec_mirror(token: String, args: Vec<String>, output: String) {
    let Ok(mut q) = queue().lock() else {
        return;
    };
    while q.len() >= CAP {
        q.pop_front();
    }
    q.push_back(RemoteMirrorEvent {
        token,
        args,
        output,
    });
}

/// Plain-text lines for the host transcript (one block per mirrored NODE_EXEC).
pub fn drain_formatted_mirror_lines() -> Vec<String> {
    let Ok(mut q) = queue().lock() else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    while let Some(ev) = q.pop_front() {
        let header = if ev.args.is_empty() {
            format!("⟵ remote  {}", ev.token)
        } else {
            format!("⟵ remote  {} {}", ev.token, ev.args.join(" "))
        };
        lines.push(header);
        lines.extend(ev.output.lines().map(|s| s.to_string()));
        lines.push(String::new());
    }
    lines
}
