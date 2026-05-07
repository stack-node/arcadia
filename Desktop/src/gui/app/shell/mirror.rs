use arcadia_core::modules::remote_mirror::{
    drain_formatted_mirror_lines, take_host_ui_sync_pending,
};
use gpui::{Context, Window};

use super::super::ArcadiaRoot;

impl ArcadiaRoot {
    /// Transcript lines + reload module/nav state when this surface shows **local** host (`remote_route` unset).
    pub(crate) fn sync_peer_remote_exec_side_effects(
        &mut self,
        _window: &Window,
        cx: &mut Context<Self>,
    ) {
        let lines = drain_formatted_mirror_lines();
        let reload_host = take_host_ui_sync_pending();

        let mut dirty = false;
        if !lines.is_empty() {
            self.shell_stream_nonce = self.shell_stream_nonce.wrapping_add(1);
            for line in lines {
                self.shell_history.push(line);
            }
            self.shell_output_scroll.scroll_to_bottom();
            dirty = true;
        }
        if reload_host && self.remote_route.is_none() {
            self.reload_modules();
            dirty = true;
        }
        if dirty {
            cx.notify();
        }
    }
}
