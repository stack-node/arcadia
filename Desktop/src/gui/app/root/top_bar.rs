use arcadia_core::navigation;
use gpui::{div, rgb, Context, InteractiveElement, IntoElement, ParentElement, Styled};

use crate::gui::app::{ArcadiaRoot, ShellMode};
use crate::gui::theme;

impl ArcadiaRoot {
    pub(crate) fn render_main_top_bar(
        &self,
        cx: &mut Context<Self>,
        active_page_title: &'static str,
        active_page_glyph: &'static str,
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
                            .child(Self::sidebar_toggle_button(cx, active_page_glyph, is_dark))
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
                            .child(if self.active_page_id == "utility.shell" {
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
                                if self.active_page_id == "utility.shell"
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
                            .child(if self.active_page_id == "utility.shell" {
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
                            .child(if self.active_page_id == "utility.shell" {
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
                        "Logs",
                        "logs",
                        "global.logs",
                        self.active_page_id == "global.logs",
                        is_dark,
                        navigation::page_by_id("global.logs")
                            .map(|p| p.accent)
                            .unwrap_or("sky"),
                    )),
            )
    }
}
