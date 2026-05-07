use arcadia_core::navigation;
use gpui::{
    div, px, rgb, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowAppearance,
};

use crate::gui::app::navigation::NavGroupRef;
use crate::gui::app::splash::SPLASH_TOTAL_MS;
use crate::gui::app::{window_controls_top_padding, ArcadiaRoot};

impl Render for ArcadiaRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.splash_elapsed_ms < SPLASH_TOTAL_MS {
            self.ensure_splash_tick(window, cx);
            return self.render_splash();
        }
        self.sync_peer_remote_exec_side_effects(window, cx);
        self.ensure_shell_caret_task(window, cx);
        self.ensure_lan_poll_task(window, cx);
        self.ensure_late_poll_task(window, cx);
        if self.tui_session.is_some() {
            self.sync_tui_size(window);
        }
        let is_dark = matches!(
            window.appearance(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        );
        let visible_groups = self.visible_groups_effective();
        let fallback_group = NavGroupRef::Static(
            navigation::group_by_id(navigation::DEFAULT_GROUP_ID)
                .unwrap_or(&navigation::GROUP_DEFINITIONS[0]),
        );
        let active_group = visible_groups
            .iter()
            .find(|g| g.id() == self.active_group_id.as_str())
            .or_else(|| visible_groups.first())
            .unwrap_or(&fallback_group);
        let active_page = self
            .active_page_if_visible()
            .or_else(|| self.page_ref(self.effective_default_page()));
        let active_page_title = gpui::SharedString::from(
            active_page
                .map(|page| page.title().to_string())
                .unwrap_or_else(|| "Arcadia".to_string()),
        );
        let active_page_glyph = gpui::SharedString::from(
            active_page
                .map(|page| page.glyph().to_string())
                .unwrap_or_else(|| "tools".to_string()),
        );

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
                    if this.session_route_menu_open {
                        this.session_route_menu_open = false;
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
                    .child(
                        if self.active_page_id.as_str() == "utility.shell"
                            || self.active_page_id.as_str() == "late.now_playing"
                        {
                            div()
                                .flex_1()
                                .min_h_0()
                                .w_full()
                                .id("arcadia-page-full")
                                .child(self.render_active_content(window, cx, active_page, is_dark))
                        } else {
                            div()
                                .flex_1()
                                .w_full()
                                .id("arcadia-page-scroll")
                                .overflow_y_scroll()
                                .child(self.render_active_content(window, cx, active_page, is_dark))
                        },
                    ),
            )
            .child(self.requirements_modal(cx, is_dark))
            .child(self.kill_existing_lan_modal(cx, is_dark))
    }
}
