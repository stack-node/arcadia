use arcadia_core::navigation;
use gpui::{
    div, img, px, rgb, Context, Div, InteractiveElement, ParentElement,
    StatefulInteractiveElement, Styled, Window,
};

use crate::gui::app::{window_controls_top_padding, ArcadiaRoot};
use crate::gui::theme::{self};

impl ArcadiaRoot {
    pub(crate) fn render_sidebar(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
        visible_groups: &[&'static navigation::NavigationGroupDefinition],
        active_group: &'static navigation::NavigationGroupDefinition,
        is_dark: bool,
    ) -> Div {
        let top_inset = window_controls_top_padding(window);
        // Pad content below traffic lights; outer column keeps full-height bg + border into titlebar.
        let content_top_pad = top_inset + px(12.);
        div()
            .h_full()
            .w_64()
            .flex()
            .flex_col()
            .bg(if is_dark {
                rgb(0x171b22)
            } else {
                rgb(0xf6f7fb)
            })
            .border_r_1()
            .border_color(if is_dark {
                rgb(0x2a3340)
            } else {
                rgb(0xe6e8ef)
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .relative()
                    .px_5()
                    .pt(content_top_pad)
                    .pb_6()
                    .gap_4()
                    .child(
                        div()
                            .relative()
                            .flex()
                            .items_center()
                            .gap_2()
                            .on_mouse_down(
                                gpui::MouseButton::Right,
                                cx.listener(|this, _, _, cx| {
                                    this.app_menu_open = true;
                                    cx.notify();
                                }),
                            )
                            .child(img("icons/app-icon.png").size_8().rounded_sm())
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(if is_dark {
                                        rgb(0xe5e7eb)
                                    } else {
                                        rgb(0x111827)
                                    })
                                    .child("Arcadia"),
                            )
                            .child(
                                div()
                                    .ml_2()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .border_1()
                                    .border_color(theme::sidebar_session_chip_border(is_dark))
                                    .bg(theme::sidebar_session_chip_bg(is_dark))
                                    .hover(move |style| {
                                        style.bg(theme::sidebar_session_chip_hover_bg(is_dark))
                                    })
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme::sidebar_session_chip_text(is_dark))
                                            .child("local"),
                                    ),
                            ),
                    )
                    .child(if self.app_menu_open {
                        div()
                            .absolute()
                            .top(px(40.))
                            .left(px(0.))
                            .w(px(112.))
                            .p_1()
                            .rounded_md()
                            .border_1()
                            .border_color(if is_dark {
                                rgb(0x374151)
                            } else {
                                rgb(0xd1d5db)
                            })
                            .bg(if is_dark {
                                rgb(0x111827)
                            } else {
                                rgb(0xffffff)
                            })
                            .child(
                                div()
                                    .w_full()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .text_sm()
                                    .text_color(if is_dark {
                                        rgb(0xfca5a5)
                                    } else {
                                        rgb(0x991b1b)
                                    })
                                    .hover(move |style| {
                                        style.bg(if is_dark {
                                            rgb(0x1f2937)
                                        } else {
                                            rgb(0xfef2f2)
                                        })
                                    })
                                    .child("Quit")
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, _, _| {
                                            this.app_menu_open = false;
                                            this.run_internal_quit_command();
                                        }),
                                    ),
                            )
                    } else {
                        div().hidden()
                    })
                    .child(
                        div()
                            .id("sidebar-group-tabs")
                            .w_full()
                            .overflow_x_scroll()
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .w_full()
                                    .justify_center()
                                    .items_start()
                                    .children(visible_groups.iter().map(|group| {
                                        Self::sidebar_group_item(
                                            cx,
                                            group.label,
                                            group.glyph,
                                            group.id,
                                            self.active_group_id == group.id,
                                            is_dark,
                                            group.accent,
                                        )
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .id("sidebar-subtabs")
                            .flex_1()
                            .overflow_y_scroll()
                            .child(div().flex().flex_col().gap_1().children(
                                active_group.pages.iter().filter_map(|page_id| {
                                    if !self.is_page_visible(page_id) {
                                        return None;
                                    }
                                    Self::page_by_id(*page_id).map(|page| {
                                        Self::sidebar_item(
                                            cx,
                                            page.title,
                                            page.glyph,
                                            page.id,
                                            self.active_page_id == page.id,
                                            is_dark,
                                            page.accent,
                                        )
                                    })
                                }),
                            )),
                    )
                    .children(navigation::GLOBAL_PAGE_IDS.iter().filter_map(|page_id| {
                        Self::page_by_id(*page_id).map(|page| {
                            Self::sidebar_global_item(
                                cx,
                                page.title,
                                page.glyph,
                                page.id,
                                self.active_page_id == page.id,
                                is_dark,
                                page.accent,
                            )
                        })
                    })),
            )
    }
}
