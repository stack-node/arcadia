use arcadia_core::navigation;
use gpui::{
    div, px, rgb, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowAppearance,
};

use crate::gui::app::splash::SPLASH_TOTAL_MS;
use crate::gui::app::{window_controls_top_padding, ArcadiaRoot};

impl Render for ArcadiaRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.splash_elapsed_ms < SPLASH_TOTAL_MS {
            self.ensure_splash_tick(window, cx);
            return self.render_splash();
        }
        self.ensure_shell_caret_task(window, cx);
        let is_dark = matches!(
            window.appearance(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        );
        let visible_groups = self.visible_groups();
        let active_page = self
            .active_page_if_visible()
            .or_else(|| Self::page_by_id(navigation::DEFAULT_PAGE_ID));
        let active_group = visible_groups
            .iter()
            .copied()
            .find(|group| group.id == self.active_group_id)
            .or_else(|| visible_groups.first().copied())
            .unwrap_or(Self::group_by_id(navigation::DEFAULT_GROUP_ID));
        let active_page_title = active_page.map(|page| page.title).unwrap_or("Arcadia");
        let active_page_glyph = active_page.map(|page| page.glyph).unwrap_or("tools");

        div()
            .size_full()
            .bg(if is_dark {
                rgb(0x0f1115)
            } else {
                rgb(0xffffff)
            })
            .flex()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    if this.app_menu_open {
                        this.app_menu_open = false;
                        cx.notify();
                    }
                }),
            )
            .on_key_down(cx.listener(Self::handle_global_key_down))
            .child(if self.sidebar_visible {
                self.render_sidebar(window, cx, &visible_groups, active_group, is_dark)
            } else {
                div()
            })
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .pt(if self.sidebar_visible {
                        px(0.)
                    } else {
                        window_controls_top_padding(window)
                    })
                    .child(self.render_main_top_bar(
                        cx,
                        active_page_title,
                        active_page_glyph,
                        is_dark,
                    ))
                    .child(if self.active_page_id == "utility.shell" {
                        div()
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .id("arcadia-page-shell")
                            .child(self.render_active_content(window, cx, active_page, is_dark))
                    } else {
                        div()
                            .flex_1()
                            .w_full()
                            .id("arcadia-page-scroll")
                            .overflow_y_scroll()
                            .child(self.render_active_content(window, cx, active_page, is_dark))
                    }),
            )
            .child(self.requirements_modal(cx, is_dark))
    }
}
