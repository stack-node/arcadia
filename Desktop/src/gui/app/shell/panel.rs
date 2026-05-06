use std::env;

use gpui::{
    div, rgb, Context, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, Window, WindowAppearance,
};

use super::super::ArcadiaRoot;

impl ArcadiaRoot {
    pub(crate) fn shell_panel(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        if self.active_page_id != "utility.shell" {
            return div();
        }
        let is_focused = self.shell_focus.is_focused(window);
        let is_dark = matches!(
            window.appearance(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        );

        if self.tui_session.is_some() && self.tui_ready {
            return self.render_tui_screen(is_dark, cx);
        }

        div()
            .w_full()
            .h_full()
            .overflow_hidden()
            .p_1()
            .rounded_lg()
            .bg(if is_dark {
                rgb(0x151a22)
            } else {
                rgb(0xf8fafc)
            })
            .border_1()
            .border_color(if is_dark {
                rgb(0x2f3948)
            } else {
                rgb(0xe2e8f0)
            })
            .flex()
            .flex_col()
            .gap_0()
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h_0()
                    .id("arcadia-shell-output")
                    .overflow_y_scroll()
                    .track_scroll(&self.shell_output_scroll)
                    .child(div().w_full().p_3().flex().flex_col().gap_2().children(
                        self.shell_history.iter().map(|line| {
                            div()
                                .text_sm()
                                .text_color(if is_dark {
                                    rgb(0xe5e7eb)
                                } else {
                                    rgb(0x1f2937)
                                })
                                .child(line.clone())
                        }),
                    )),
            )
            .child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .px_3()
                    .py_2()
                    .flex()
                    .gap_2()
                    .items_center()
                    .border_t_1()
                    .border_color(if is_focused {
                        rgb(0x3b82f6)
                    } else if is_dark {
                        rgb(0x2f3948)
                    } else {
                        rgb(0xe2e8f0)
                    })
                    .bg(if is_dark {
                        rgb(0x0f141b)
                    } else {
                        rgb(0xffffff)
                    })
                    .track_focus(&self.shell_focus)
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, window, _| {
                            this.shell_focus.focus(window);
                        }),
                    )
                    .on_key_down(cx.listener(Self::handle_shell_key_down))
                    .child(
                        div()
                            .text_sm()
                            .text_color(if is_dark {
                                rgb(0x60a5fa)
                            } else {
                                rgb(0x1d4ed8)
                            })
                            .child("$"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(if is_dark {
                                rgb(0xe5e7eb)
                            } else {
                                rgb(0x111827)
                            })
                            .child(self.shell_input_with_cursor(is_focused)),
                    ),
            )
    }

    pub(crate) fn shell_input_with_cursor(&self, is_focused: bool) -> String {
        let chars = self.shell_input.chars().collect::<Vec<_>>();
        let cursor = self.shell_cursor.min(chars.len());
        let mut out = String::with_capacity(chars.len() + 1);
        for (idx, ch) in chars.iter().enumerate() {
            if idx == cursor && is_focused && self.shell_caret_visible {
                out.push('|');
            }
            out.push(*ch);
        }
        if cursor == chars.len() && is_focused && self.shell_caret_visible {
            out.push('|');
        }
        out
    }

    pub(crate) fn shell_working_directory_label(&self) -> String {
        env::current_dir()
            .ok()
            .and_then(|path| path.to_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| "cwd: unavailable".to_string())
    }
}
