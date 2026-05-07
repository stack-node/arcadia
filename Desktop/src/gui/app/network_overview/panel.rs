use arcadia_core::config::modules::{LAN_MODULE_NAME, NET_MODULE_NAME};
use arcadia_core::modules::lan::{lan_service_info, start_service, stop_service};
use gpui::{
    div, rgb, Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

impl ArcadiaRoot {
    pub fn network_overview_panel(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let net_enabled = self.is_module_enabled(NET_MODULE_NAME);
        let lan_enabled = self.is_module_enabled(LAN_MODULE_NAME);

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.net_status_row(net_enabled, is_dark))
            .child(if lan_enabled {
                self.lan_service_row(cx, is_dark).into_any_element()
            } else {
                div()
                    .text_sm()
                    .text_color(theme::module_description_text(is_dark))
                    .child("Enable the LAN module to manage the discovery service.")
                    .into_any_element()
            })
    }

    fn net_status_row(&self, net_enabled: bool, is_dark: bool) -> impl IntoElement {
        div()
            .w_full()
            .px_4()
            .py_3()
            .rounded_lg()
            .bg(theme::module_panel_bg(is_dark))
            .border_1()
            .border_color(theme::module_panel_stroke(is_dark))
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child("Network Module"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::module_meta_text(is_dark))
                            .child("Networking foundation — required by LAN and remote session"),
                    ),
            )
            .child(self.status_badge(net_enabled, is_dark))
    }

    fn lan_service_row(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let info = lan_service_info();
        let running = info.running;
        div()
            .w_full()
            .px_4()
            .py_3()
            .rounded_lg()
            .bg(theme::module_panel_bg(is_dark))
            .border_1()
            .border_color(theme::module_panel_stroke(is_dark))
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child("LAN Discovery Service"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::module_meta_text(is_dark))
                            .child(format!("UDP :{} · {}", info.port, info.hostname)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(self.service_running_badge(running, is_dark))
                    .child(
                        div()
                            .cursor_pointer()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(theme::module_row_bg(is_dark))
                            .border_1()
                            .border_color(theme::module_row_stroke(is_dark))
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child(if running { "Stop" } else { "Start" })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_, _, _, cx| {
                                    if running {
                                        stop_service();
                                    } else {
                                        start_service();
                                    }
                                    cx.notify();
                                }),
                            ),
                    ),
            )
    }

    fn status_badge(&self, enabled: bool, is_dark: bool) -> impl IntoElement {
        let _ = is_dark;
        div()
            .px_2()
            .py_0p5()
            .rounded_full()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .bg(if enabled {
                rgb(0x166534)
            } else {
                rgb(0x374151)
            })
            .text_color(if enabled {
                rgb(0x86efac)
            } else {
                rgb(0x9ca3af)
            })
            .child(if enabled { "enabled" } else { "disabled" })
    }

    fn service_running_badge(&self, running: bool, is_dark: bool) -> impl IntoElement {
        let _ = is_dark;
        div()
            .px_2()
            .py_0p5()
            .rounded_full()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .bg(if running {
                rgb(0x166534)
            } else {
                rgb(0x7f1d1d)
            })
            .text_color(if running {
                rgb(0x86efac)
            } else {
                rgb(0xfca5a5)
            })
            .child(if running { "running" } else { "stopped" })
    }
}
