use gpui::{div, rgb, Context, InteractiveElement, IntoElement, ParentElement, Styled};

use crate::gui::app::{ArcadiaRoot, ShellMode};
use crate::gui::theme;

const LATE_ROOMS: &[(&str, u32)] = &[("1", 1), ("2", 2), ("3", 3), ("4", 4), ("5", 5)];

impl ArcadiaRoot {
    pub(crate) fn render_main_top_bar(
        &self,
        cx: &mut Context<Self>,
        active_page_title: gpui::SharedString,
        active_page_glyph: gpui::SharedString,
        is_dark: bool,
    ) -> impl IntoElement {
        div()
            .w_full()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(if is_dark {
                rgb(0x2a3340)
            } else {
                rgb(0xe6e8ef)
            })
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(Self::sidebar_toggle_button(cx, active_page_glyph.as_ref(), is_dark))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(if is_dark {
                                        rgb(0xe5e7eb)
                                    } else {
                                        rgb(0x1f2937)
                                    })
                                    .child(active_page_title),
                            )
                            .child(if self.active_page_id.as_str() == "late.now_playing" {
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .children(LATE_ROOMS.iter().map(|(label, room_id)| {
                                        let rid = *room_id;
                                        let is_active = rid == self.late_active_room;
                                        div()
                                            .cursor_pointer()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_md()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .bg(if is_active {
                                                if is_dark { rgb(0x0d9488) } else { rgb(0x99f6e4) }
                                            } else {
                                                theme::top_bar_pill_bg(is_dark)
                                            })
                                            .text_color(if is_active {
                                                if is_dark { rgb(0xf0fdfa) } else { rgb(0x134e4a) }
                                            } else {
                                                theme::top_bar_pill_text(is_dark)
                                            })
                                            .hover(move |style| {
                                                if !is_active {
                                                    style.bg(theme::top_bar_pill_hover_bg(is_dark))
                                                } else {
                                                    style
                                                }
                                            })
                                            .child(*label)
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    this.late_active_room = rid;
                                                    arcadia_core::modules::late::send_ws(
                                                        format!(r#"{{"type":"subscribe","room_id":{rid}}}"#),
                                                    );
                                                    cx.notify();
                                                }),
                                            )
                                    }))
                            } else {
                                div()
                            })
                            .child(if self.active_page_id.as_str() == "utility.shell" {
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .text_xs()
                                    .bg(if self.shell_mode == ShellMode::Generic {
                                        if is_dark {
                                            rgb(0x1e3a5f)
                                        } else {
                                            rgb(0xdbeafe)
                                        }
                                    } else {
                                        if is_dark {
                                            rgb(0x422006)
                                        } else {
                                            rgb(0xffedd5)
                                        }
                                    })
                                    .text_color(if self.shell_mode == ShellMode::Generic {
                                        if is_dark {
                                            rgb(0x93c5fd)
                                        } else {
                                            rgb(0x1d4ed8)
                                        }
                                    } else {
                                        if is_dark {
                                            rgb(0xfdba74)
                                        } else {
                                            rgb(0xc2410c)
                                        }
                                    })
                                    .child(self.shell_mode.label())
                            } else {
                                div()
                            })
                            .child(
                                if self.active_page_id.as_str() == "utility.shell"
                                    && self.shell_mode == ShellMode::Generic
                                {
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .text_xs()
                                        .bg(theme::top_bar_pill_bg(is_dark))
                                        .text_color(theme::top_bar_pill_text(is_dark))
                                        .child(self.shell_working_directory_label())
                                } else {
                                    div().hidden()
                                },
                            )
                            .child(if self.active_page_id.as_str() == "utility.shell" {
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .text_xs()
                                    .bg(theme::top_bar_pill_bg(is_dark))
                                    .text_color(theme::top_bar_pill_text(is_dark))
                                    .hover(move |style| {
                                        style.bg(theme::top_bar_pill_hover_bg(is_dark))
                                    })
                                    .child("Reset")
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.reset_shell_state();
                                            cx.notify();
                                        }),
                                    )
                            } else {
                                div()
                            })
                            .child(if self.active_page_id.as_str() == "utility.shell" {
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .text_xs()
                                    .bg(theme::top_bar_pill_bg(is_dark))
                                    .text_color(theme::top_bar_pill_text(is_dark))
                                    .hover(move |style| {
                                        style.bg(theme::top_bar_pill_hover_bg(is_dark))
                                    })
                                    .child("Clear")
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.shell_history.clear();
                                            this.shell_output_scroll.scroll_to_bottom();
                                            cx.notify();
                                        }),
                                    )
                            } else {
                                div()
                            }),
                    )
                    .child(Self::top_bar_global_item(
                        cx,
                        "Logs".into(),
                        "logs".into(),
                        "global.logs".into(),
                        self.active_page_id.as_str() == "global.logs",
                        is_dark,
                        self.page_ref("global.logs")
                            .map(|p| p.accent().to_string())
                            .unwrap_or_else(|| "sky".to_string()),
                    )),
            )
    }
}
