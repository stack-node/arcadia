use std::time::Duration;

use crate::cli;
use arcadia_core::config::modules::{
    ModuleManifest, ModulesConfig, NET_MODULE_NAME, REMOTE_SESSION_MODULE_NAME, SHELL_MODULE_NAME,
};
use arcadia_core::config::ConfigFile;
use arcadia_core::modules;
use arcadia_core::navigation;
use gpui::{
    canvas, div, fill, img, point, px, rgb, size, AppContext, Application, Bounds, Context, Div,
    FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement, PathBuilder, Render,
    Rgba, SharedString, StatefulInteractiveElement, Styled, Timer, Window, WindowAppearance,
    WindowOptions,
};

use super::assets::EmbeddedAssets;
use super::theme::{self, render_icon};

#[derive(Clone, Copy, PartialEq)]
pub enum ShellMode {
    Generic,
    Internal,
}

impl ShellMode {
    fn toggle(self) -> Self {
        match self {
            ShellMode::Generic => ShellMode::Internal,
            ShellMode::Internal => ShellMode::Generic,
        }
    }

    fn label(self) -> &'static str {
        match self {
            ShellMode::Generic => "shell",
            ShellMode::Internal => "internal",
        }
    }

    fn command_token(self) -> &'static str {
        match self {
            ShellMode::Generic => "shell.execute",
            ShellMode::Internal => "shell.internal",
        }
    }
}

pub struct ArcadiaRoot {
    pub title: SharedString,
    pub active_page_id: &'static str,
    pub active_group_id: &'static str,
    pub module_rows: Vec<(String, bool)>,
    pub pending_module_enable: Option<(String, Vec<String>)>,
    pub shell_enabled: bool,
    pub shell_history: Vec<String>,
    pub shell_input: String,
    pub shell_focus: FocusHandle,
    pub shell_cursor: usize,
    pub shell_command_history: Vec<String>,
    pub shell_history_index: Option<usize>,
    pub shell_caret_visible: bool,
    pub shell_caret_task_started: bool,
    pub shell_stream_nonce: u64,
    pub shell_mode: ShellMode,
    pub splash_elapsed_ms: f32,
    pub splash_tick_started: bool,
    pub sidebar_visible: bool,
}

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

        div()
            .size_full()
            .bg(if is_dark {
                rgb(0x0f1115)
            } else {
                rgb(0xffffff)
            })
            .flex()
            .child(if self.sidebar_visible {
                self.render_sidebar(cx, &visible_groups, active_group, is_dark)
            } else {
                div()
            })
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(
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
                                    .child(Self::sidebar_toggle_button(cx, is_dark))
                                    .child(Self::sidebar_global_item(
                                        cx,
                                        "Logs",
                                        "logs",
                                        "global.logs",
                                        self.active_page_id == "global.logs",
                                        is_dark,
                                    )),
                            ),
                    )
                    .child(self.render_active_content(window, cx, active_page, is_dark)),
            )
            .child(self.requirements_modal(cx, is_dark))
    }
}

impl ArcadiaRoot {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let shell_focus = cx.focus_handle();
        let module_rows = ModulesConfig::load_or_create()
            .map(|cfg| cfg.modules.into_iter().collect::<Vec<(String, bool)>>())
            .unwrap_or_default();
        let shell_enabled = module_rows
            .iter()
            .find(|(name, _)| name == SHELL_MODULE_NAME)
            .map(|(_, enabled)| *enabled)
            .unwrap_or(false);
        ArcadiaRoot {
            title: SharedString::new_static("Arcadia"),
            active_page_id: navigation::DEFAULT_PAGE_ID,
            active_group_id: navigation::DEFAULT_GROUP_ID,
            module_rows,
            pending_module_enable: None,
            shell_enabled,
            shell_history: vec!["Arcadia Terminal ready.".to_string()],
            shell_input: String::new(),
            shell_focus,
            shell_cursor: 0,
            shell_command_history: Vec::new(),
            shell_history_index: None,
            shell_caret_visible: true,
            shell_caret_task_started: false,
            shell_stream_nonce: 0,
            shell_mode: ShellMode::Generic,
            splash_elapsed_ms: 0.0,
            splash_tick_started: false,
            sidebar_visible: true,
        }
    }

    pub fn reload_modules(&mut self) {
        self.module_rows = ModulesConfig::load_or_create()
            .map(|cfg| cfg.modules.into_iter().collect())
            .unwrap_or_default();
        self.shell_enabled = self
            .module_rows
            .iter()
            .find(|(name, _)| name == SHELL_MODULE_NAME)
            .map(|(_, enabled)| *enabled)
            .unwrap_or(false);
        self.ensure_valid_navigation_selection();
    }

    fn render_sidebar(
        &self,
        cx: &mut Context<Self>,
        visible_groups: &[&'static navigation::NavigationGroupDefinition],
        active_group: &'static navigation::NavigationGroupDefinition,
        is_dark: bool,
    ) -> Div {
        div()
            .h_full()
            .w_64()
            .flex()
            .flex_col()
            .p_4()
            .gap_2()
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
                    .items_center()
                    .gap_2()
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
                    .child(if self.remote_session_enabled() {
                        div()
                            .ml_2()
                            .px_2()
                            .py_1()
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
                            .text_xs()
                            .font_weight(gpui::FontWeight::NORMAL)
                            .text_color(if is_dark {
                                rgb(0xd1d5db)
                            } else {
                                rgb(0x374151)
                            })
                            .child("local v")
                    } else {
                        div()
                    }),
            )
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
                    )
                })
            }))
    }

    fn render_active_content(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
        active_page: Option<&'static navigation::NavigationPageDefinition>,
        is_dark: bool,
    ) -> Div {
        if self.active_page_id == "utility.shell" {
            return div()
                .flex_1()
                .h_full()
                .p_2()
                .child(self.shell_panel(window, cx));
        }
        if self.active_page_id == "global.modules" {
            return div()
                .flex_1()
                .h_full()
                .p_6()
                .child(self.modules_panel(cx, is_dark));
        }
        div()
            .flex_1()
            .h_full()
            .p_6()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .gap_3()
            .child(
                div()
                    .text_3xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .text_2xl()
                    .text_color(if is_dark {
                        rgb(0xe5e7eb)
                    } else {
                        rgb(0x1f2937)
                    })
                    .child(active_page.map_or("Page", |page| page.title)),
            )
            .child(
                div()
                    .text_base()
                    .text_color(if is_dark {
                        rgb(0x9ca3af)
                    } else {
                        rgb(0x6b7280)
                    })
                    .child(
                        active_page.map_or("Page definition not found.", |page| page.description),
                    ),
            )
    }
    fn net_enabled(&self) -> bool {
        self.module_rows
            .iter()
            .find(|(name, _)| name == NET_MODULE_NAME)
            .map(|(_, enabled)| *enabled)
            .unwrap_or(false)
    }

    fn remote_session_enabled(&self) -> bool {
        self.module_rows
            .iter()
            .find(|(name, _)| name == REMOTE_SESSION_MODULE_NAME)
            .map(|(_, enabled)| *enabled)
            .unwrap_or(false)
    }

    pub fn is_page_visible(&self, page_id: &str) -> bool {
        match page_id {
            "utility.shell" => self.shell_enabled,
            "network.overview" => self.net_enabled(),
            _ => true,
        }
    }

    pub fn active_page_if_visible(&self) -> Option<&'static navigation::NavigationPageDefinition> {
        if self.is_page_visible(self.active_page_id) {
            Self::page_by_id(self.active_page_id)
        } else {
            None
        }
    }

    pub fn visible_groups(&self) -> Vec<&'static navigation::NavigationGroupDefinition> {
        navigation::GROUP_DEFINITIONS
            .iter()
            .filter(|group| {
                group
                    .pages
                    .iter()
                    .any(|page_id| self.is_page_visible(page_id))
            })
            .collect()
    }

    pub fn ensure_valid_navigation_selection(&mut self) {
        let visible_groups = self.visible_groups();
        let group_is_visible = visible_groups
            .iter()
            .any(|group| group.id == self.active_group_id);
        if !group_is_visible {
            if let Some(group) = visible_groups.first() {
                self.active_group_id = group.id;
            } else {
                self.active_group_id = navigation::DEFAULT_GROUP_ID;
            }
        }

        let active_group = visible_groups
            .iter()
            .copied()
            .find(|group| group.id == self.active_group_id)
            .or_else(|| visible_groups.first().copied());
        let page_is_visible = self.is_page_visible(self.active_page_id);
        if !page_is_visible {
            if let Some(group) = active_group {
                if let Some(first_visible_page) = group
                    .pages
                    .iter()
                    .find(|page_id| self.is_page_visible(page_id))
                {
                    self.active_page_id = first_visible_page;
                }
            }
        }
    }

    pub fn page_by_id(
        page_id: &'static str,
    ) -> Option<&'static navigation::NavigationPageDefinition> {
        navigation::page_by_id(page_id)
    }

    pub fn group_by_id(group_id: &'static str) -> &'static navigation::NavigationGroupDefinition {
        navigation::group_by_id(group_id).unwrap_or(&navigation::GROUP_DEFINITIONS[0])
    }

    pub fn modules_panel(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        if self.active_page_id != "global.modules" {
            return div();
        }
        div()
            .w_full()
            .p_4()
            .rounded_lg()
            .bg(theme::module_panel_bg(is_dark))
            .border_1()
            .border_color(theme::module_panel_stroke(is_dark))
            .flex()
            .flex_col()
            .gap_3()
            .children(self.module_rows.iter().map(|(module_name, enabled)| {
                Self::module_row_item(
                    cx,
                    module_name.clone(),
                    *enabled,
                    ModulesConfig::manifest_for(module_name),
                    is_dark,
                )
            }))
    }

    pub fn shell_panel(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.active_page_id != "utility.shell" {
            return div();
        }
        let is_focused = self.shell_focus.is_focused(window);
        let is_dark = matches!(
            window.appearance(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        );

        div()
            .w_full()
            .h_full()
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
                    .w_full()
                    .px_3()
                    .py_2()
                    .flex()
                    .justify_between()
                    .items_center()
                    .bg(if is_dark {
                        rgb(0x0f141b)
                    } else {
                        rgb(0xffffff)
                    })
                    .border_b_1()
                    .border_color(if is_dark {
                        rgb(0x2f3948)
                    } else {
                        rgb(0xe2e8f0)
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_dark {
                                        rgb(0xe5e7eb)
                                    } else {
                                        rgb(0x111827)
                                    })
                                    .child("Terminal"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .text_xs()
                                    .bg(if self.shell_mode == ShellMode::Internal {
                                        if is_dark {
                                            rgb(0x1e3a5f)
                                        } else {
                                            rgb(0xdbeafe)
                                        }
                                    } else if is_dark {
                                        rgb(0x1f2937)
                                    } else {
                                        rgb(0xf3f4f6)
                                    })
                                    .text_color(if self.shell_mode == ShellMode::Internal {
                                        if is_dark {
                                            rgb(0x93c5fd)
                                        } else {
                                            rgb(0x1d4ed8)
                                        }
                                    } else if is_dark {
                                        rgb(0x9ca3af)
                                    } else {
                                        rgb(0x6b7280)
                                    })
                                    .child(self.shell_mode.label()),
                            ),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .cursor_pointer()
                            .bg(if is_dark {
                                rgb(0x1f2937)
                            } else {
                                rgb(0xf3f4f6)
                            })
                            .text_color(if is_dark {
                                rgb(0xd1d5db)
                            } else {
                                rgb(0x374151)
                            })
                            .child("Clear")
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.shell_history.clear();
                                    cx.notify();
                                }),
                            ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .flex_1()
                    .p_3()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(self.shell_history.iter().map(|line| {
                        div()
                            .text_sm()
                            .text_color(if is_dark {
                                rgb(0xe5e7eb)
                            } else {
                                rgb(0x1f2937)
                            })
                            .child(line.clone())
                    })),
            )
            .child(
                div()
                    .w_full()
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

    fn shell_input_with_cursor(&self, is_focused: bool) -> String {
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

    pub fn ensure_shell_caret_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.shell_caret_task_started {
            return;
        }
        self.shell_caret_task_started = true;
        cx.spawn_in(
            window,
            move |view: gpui::WeakEntity<ArcadiaRoot>, cx: &mut gpui::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        Timer::after(Duration::from_millis(500)).await;
                        let should_stop = cx
                            .update(|_, app| {
                                view.update(app, |this, cx| {
                                    if !this.shell_enabled {
                                        this.shell_caret_task_started = false;
                                        return true;
                                    }
                                    this.shell_caret_visible = !this.shell_caret_visible;
                                    cx.notify();
                                    false
                                })
                                .unwrap_or(true)
                            })
                            .unwrap_or(true);
                        if should_stop {
                            break;
                        }
                    }
                }
            },
        )
        .detach();
    }

    fn ensure_splash_tick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.splash_tick_started {
            return;
        }
        self.splash_tick_started = true;
        cx.spawn_in(
            window,
            move |view: gpui::WeakEntity<ArcadiaRoot>, cx: &mut gpui::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        Timer::after(Duration::from_millis(16)).await;
                        let done = cx
                            .update(|_, app| {
                                view.update(app, |this, cx| {
                                    this.splash_elapsed_ms += 16.0;
                                    cx.notify();
                                    this.splash_elapsed_ms >= SPLASH_TOTAL_MS
                                })
                                .unwrap_or(true)
                            })
                            .unwrap_or(true);
                        if done {
                            break;
                        }
                    }
                }
            },
        )
        .detach();
    }

    fn render_splash(&self) -> Div {
        let t = self.splash_elapsed_ms;

        let hills_t = ease_out_cubic(splash_phase(t, 200.0, 900.0));
        let sun_t = ease_out_cubic(splash_phase(t, 700.0, 1100.0));
        let arch_t = ease_out_cubic(splash_phase(t, 1300.0, 1200.0));
        let stars_t = splash_phase(t, 2000.0, 800.0);
        let master_alpha = if t < 3600.0 {
            splash_phase(t, 0.0, 400.0)
        } else {
            1.0 - splash_phase(t, 3600.0, 700.0)
        };

        div().size_full().opacity(master_alpha).child(
            canvas(
                |_bounds, _window, _cx| {},
                move |bounds, _, window, _cx| {
                    splash_draw_bg(bounds, window);
                    splash_draw_horizon_glow(bounds, sun_t, window);
                    splash_draw_arch(bounds, arch_t, window);
                    splash_draw_stars(bounds, stars_t, window);
                    splash_draw_sun(bounds, sun_t, window);
                    splash_draw_hills(bounds, hills_t, window);
                },
            )
            .size_full(),
        )
    }

    fn handle_shell_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.active_page_id != "utility.shell" {
            return;
        }
        let key = event.keystroke.key.as_str();
        let mods = event.keystroke.modifiers;
        if key == "tab" && mods.shift {
            self.shell_mode = self.shell_mode.toggle();
            cx.notify();
            return;
        }
        match key {
            "enter" => {
                let command = self.shell_input.trim().to_string();
                if !command.is_empty() {
                    self.run_shell_execute(&command, _window, cx);
                    self.shell_command_history.push(command);
                }
                self.shell_input.clear();
                self.shell_cursor = 0;
                self.shell_history_index = None;
            }
            "backspace" => {
                if self.shell_cursor > 0 {
                    let mut chars = self.shell_input.chars().collect::<Vec<_>>();
                    chars.remove(self.shell_cursor - 1);
                    self.shell_input = chars.into_iter().collect();
                    self.shell_cursor -= 1;
                }
            }
            "left" => {
                self.shell_cursor = self.shell_cursor.saturating_sub(1);
            }
            "right" => {
                let len = self.shell_input.chars().count();
                self.shell_cursor = (self.shell_cursor + 1).min(len);
            }
            "up" => {
                if !self.shell_command_history.is_empty() {
                    let next_index = match self.shell_history_index {
                        Some(index) => index.saturating_sub(1),
                        None => self.shell_command_history.len().saturating_sub(1),
                    };
                    self.shell_history_index = Some(next_index);
                    self.shell_input = self.shell_command_history[next_index].clone();
                    self.shell_cursor = self.shell_input.chars().count();
                }
            }
            "down" => {
                if let Some(index) = self.shell_history_index {
                    let next_index = index + 1;
                    if next_index < self.shell_command_history.len() {
                        self.shell_history_index = Some(next_index);
                        self.shell_input = self.shell_command_history[next_index].clone();
                        self.shell_cursor = self.shell_input.chars().count();
                    } else {
                        self.shell_history_index = None;
                        self.shell_input.clear();
                        self.shell_cursor = 0;
                    }
                }
            }
            "home" => self.shell_cursor = 0,
            "end" => self.shell_cursor = self.shell_input.chars().count(),
            "space" => {
                let mut chars = self.shell_input.chars().collect::<Vec<_>>();
                chars.insert(self.shell_cursor, ' ');
                self.shell_input = chars.into_iter().collect();
                self.shell_cursor += 1;
            }
            _ => {
                if !mods.control && !mods.alt && !mods.platform && !mods.function {
                    if let Some(key_char) = &event.keystroke.key_char {
                        let mut chars = self.shell_input.chars().collect::<Vec<_>>();
                        for ch in key_char.chars() {
                            chars.insert(self.shell_cursor, ch);
                            self.shell_cursor += 1;
                        }
                        self.shell_input = chars.into_iter().collect();
                    }
                }
            }
        }
        cx.notify();
    }

    pub fn run_shell_execute(
        &mut self,
        command: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let normalized = command.trim();
        if normalized.eq_ignore_ascii_case("clear") || normalized.eq_ignore_ascii_case("cls") {
            self.shell_stream_nonce = self.shell_stream_nonce.wrapping_add(1);
            self.shell_history.clear();
            cx.notify();
            return;
        }
        let args = vec![command];
        let result = modules::execute_command(
            self.shell_mode.command_token(),
            &args,
            &modules::ExecutionContext::default(),
        );
        self.shell_stream_nonce = self.shell_stream_nonce.wrapping_add(1);
        let stream_nonce = self.shell_stream_nonce;
        self.shell_history.push(format!("$ {command}"));
        let output = match result {
            Ok(Some(output)) => output,
            Ok(None) => "Unknown shell command token.".to_string(),
            Err(err) => err,
        };
        self.shell_history.push(String::new());
        // Stream output line-by-line via async task to keep UI responsive
        let lines: Vec<String> = output.lines().map(str::to_string).collect();
        cx.spawn_in(
            window,
            move |view: gpui::WeakEntity<ArcadiaRoot>, cx: &mut gpui::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    for line in lines {
                        Timer::after(Duration::from_millis(4)).await;
                        let _ = cx.update(|_, app| {
                            let _ = view.update(app, |this, cx| {
                                if this.shell_stream_nonce != stream_nonce {
                                    return;
                                }
                                this.shell_history.push(line);
                                cx.notify();
                            });
                        });
                    }
                    let _ = cx.update(|_, app| {
                        let _ = view.update(app, |this, cx| {
                            if this.shell_stream_nonce == stream_nonce {
                                this.shell_history.push(String::new());
                                cx.notify();
                            }
                        });
                    });
                }
            },
        )
        .detach();
    }

    pub fn requirements_modal(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        let Some((module_name, missing)) = &self.pending_module_enable else {
            return div();
        };
        let requirements = missing.join(", ");

        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .bg(rgb(0x000000))
                    .opacity(0.35)
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.pending_module_enable = None;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                div()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        div()
                            .w_128()
                            .p_5()
                            .rounded_lg()
                            .bg(if is_dark { rgb(0x111827) } else { rgb(0xffffff) })
                            .border_1()
                            .border_color(if is_dark { rgb(0x374151) } else { rgb(0xe2e8f0) })
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(if is_dark { rgb(0xf9fafb) } else { rgb(0x111827) })
                                    .child("Enable with requirements?"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_dark { rgb(0xd1d5db) } else { rgb(0x374151) })
                                    .child(format!(
                                        "To enable {module_name}, Arcadia needs to enable: {requirements}."
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .justify_end()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .bg(if is_dark { rgb(0x374151) } else { rgb(0xe5e7eb) })
                                            .text_color(if is_dark { rgb(0xf3f4f6) } else { rgb(0x1f2937) })
                                            .child("Cancel")
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.pending_module_enable = None;
                                                    cx.notify();
                                                }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .bg(rgb(0xdbeafe))
                                            .text_color(rgb(0x1d4ed8))
                                            .child("Enable")
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    if let Some((module_name, _)) =
                                                        this.pending_module_enable.clone()
                                                    {
                                                        let _ = cli::handle(&format!(
                                                            "module {module_name} enable -requirements"
                                                        ));
                                                        this.reload_modules();
                                                    }
                                                    this.pending_module_enable = None;
                                                    cx.notify();
                                                }),
                                            ),
                                    ),
                            ),
                    ),
            )
    }

    pub fn module_row_item(
        cx: &mut Context<Self>,
        module_name: String,
        enabled: bool,
        manifest: Option<&'static ModuleManifest>,
        is_dark: bool,
    ) -> impl IntoElement {
        let label = if enabled { "Disable" } else { "Enable" };
        let version = manifest.map(|m| m.version).unwrap_or("unknown");
        let description = manifest
            .map(|m| m.description)
            .unwrap_or("No manifest description.");
        let state = if enabled { "Enabled" } else { "Disabled" };
        div()
            .w_full()
            .px_4()
            .py_3()
            .rounded_lg()
            .bg(theme::module_row_bg(is_dark))
            .border_1()
            .border_color(theme::module_row_stroke(is_dark))
            .flex()
            .justify_between()
            .items_center()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::module_title_text(is_dark))
                            .child(module_name.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::module_meta_text(is_dark))
                                    .child(format!("v{version}")),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .bg(if enabled {
                                        theme::module_state_enabled_bg(is_dark)
                                    } else {
                                        theme::module_state_disabled_bg(is_dark)
                                    })
                                    .text_color(if enabled {
                                        theme::module_state_enabled_text(is_dark)
                                    } else {
                                        theme::module_state_disabled_text(is_dark)
                                    })
                                    .child(state),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::module_description_text(is_dark))
                            .child(description),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .py_1p5()
                    .rounded_md()
                    .cursor_pointer()
                    .bg(if enabled {
                        theme::module_button_disable_bg(is_dark)
                    } else {
                        theme::module_button_enable_bg(is_dark)
                    })
                    .text_color(if enabled {
                        theme::module_button_disable_text(is_dark)
                    } else {
                        theme::module_button_enable_text(is_dark)
                    })
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(label)
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            if enabled {
                                let _ = cli::handle(&format!("module {module_name} disable"));
                                this.pending_module_enable = None;
                                this.reload_modules();
                                cx.notify();
                                return;
                            }
                            match ModulesConfig::load_or_create() {
                                Ok(cfg) => match cfg.missing_requirements_for(&module_name) {
                                    Ok(missing) if !missing.is_empty() => {
                                        this.pending_module_enable =
                                            Some((module_name.clone(), missing));
                                    }
                                    Ok(_) => {
                                        let _ =
                                            cli::handle(&format!("module {module_name} enable"));
                                        this.pending_module_enable = None;
                                        this.reload_modules();
                                    }
                                    Err(err) => eprintln!("{err}"),
                                },
                                Err(err) => eprintln!("{err}"),
                            }
                            cx.notify();
                        }),
                    ),
            )
    }

    pub fn sidebar_toggle_button(cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
        div()
            .w_8()
            .h_8()
            .rounded_md()
            .cursor_pointer()
            .bg(if is_dark {
                rgb(0x1f2937)
            } else {
                rgb(0xf3f4f6)
            })
            .text_color(if is_dark {
                rgb(0xe5e7eb)
            } else {
                rgb(0x1f2937)
            })
            .hover(move |style| {
                style.bg(if is_dark {
                    rgb(0x243246)
                } else {
                    rgb(0xe5e7eb)
                })
            })
            .flex()
            .items_center()
            .justify_center()
            .child(render_icon("tools").size_4())
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.sidebar_visible = !this.sidebar_visible;
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_group_item(
        cx: &mut Context<Self>,
        label: &'static str,
        system_image: &'static str,
        group_id: &'static str,
        is_active: bool,
        is_dark: bool,
    ) -> impl IntoElement {
        let icon_color = if is_active {
            if is_dark {
                rgb(0x93c5fd)
            } else {
                rgb(0x1d4ed8)
            }
        } else if is_dark {
            rgb(0xd1d5db)
        } else {
            rgb(0x374151)
        };
        div()
            .w_16()
            .h_16()
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active {
                gpui::FontWeight::BOLD
            } else {
                gpui::FontWeight::NORMAL
            })
            .bg(if is_active {
                if is_dark {
                    rgb(0x1f2a3e)
                } else {
                    rgb(0xe1e7ff)
                }
            } else if is_dark {
                rgb(0x171b22)
            } else {
                rgb(0xf6f7fb)
            })
            .text_color(icon_color)
            .hover(move |style| {
                style.bg(if is_dark {
                    rgb(0x243246)
                } else {
                    rgb(0xeef2ff)
                })
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .text_center()
                    .child(render_icon(system_image).size_5().text_color(icon_color))
                    .child(div().child(label)),
            )
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_group_id = group_id;
                    if let Some(group) = navigation::GROUP_DEFINITIONS
                        .iter()
                        .find(|group| group.id == group_id)
                    {
                        if let Some(first_page_id) = group.pages.first() {
                            this.active_page_id = first_page_id;
                        }
                    }
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_global_item(
        cx: &mut Context<Self>,
        label: &'static str,
        system_image: &'static str,
        page_id: &'static str,
        is_active: bool,
        is_dark: bool,
    ) -> impl IntoElement {
        let icon_color = if is_active {
            if is_dark {
                rgb(0x93c5fd)
            } else {
                rgb(0x1d4ed8)
            }
        } else if is_dark {
            rgb(0xd1d5db)
        } else {
            rgb(0x374151)
        };
        div()
            .px_3()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .text_sm()
            .font_weight(if is_active {
                gpui::FontWeight::BOLD
            } else {
                gpui::FontWeight::NORMAL
            })
            .bg(if is_active {
                if is_dark {
                    rgb(0x1f2a3e)
                } else {
                    rgb(0xe1e7ff)
                }
            } else if is_dark {
                rgb(0x171b22)
            } else {
                rgb(0xf6f7fb)
            })
            .text_color(icon_color)
            .hover(move |style| {
                style.bg(if is_dark {
                    rgb(0x243246)
                } else {
                    rgb(0xeef2ff)
                })
            })
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(render_icon(system_image).size_4().text_color(icon_color))
                    .child(div().child(label)),
            )
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id;
                    if page_id == "global.modules" {
                        this.reload_modules();
                    }
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_item(
        cx: &mut Context<Self>,
        label: &'static str,
        system_image: &'static str,
        page_id: &'static str,
        is_active: bool,
        is_dark: bool,
    ) -> impl IntoElement {
        let icon_color = if is_active {
            if is_dark {
                rgb(0x93c5fd)
            } else {
                rgb(0x1d4ed8)
            }
        } else if is_dark {
            rgb(0xd1d5db)
        } else {
            rgb(0x374151)
        };
        div()
            .px_3()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .text_sm()
            .font_weight(if is_active {
                gpui::FontWeight::BOLD
            } else {
                gpui::FontWeight::NORMAL
            })
            .bg(if is_active {
                if is_dark {
                    rgb(0x1f2a3e)
                } else {
                    rgb(0xe1e7ff)
                }
            } else if is_dark {
                rgb(0x171b22)
            } else {
                rgb(0xf6f7fb)
            })
            .text_color(icon_color)
            .hover(move |style| {
                style.bg(if is_dark {
                    rgb(0x243246)
                } else {
                    rgb(0xeef2ff)
                })
            })
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(render_icon(system_image).size_4().text_color(icon_color))
                    .child(div().child(label)),
            )
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id;
                    cx.notify();
                }),
            )
    }
}

const SPLASH_TOTAL_MS: f32 = 4500.0;

fn splash_phase(elapsed_ms: f32, start_ms: f32, duration_ms: f32) -> f32 {
    ((elapsed_ms - start_ms) / duration_ms).clamp(0.0, 1.0)
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_rgba(a: Rgba, b: Rgba, t: f32) -> Rgba {
    Rgba {
        r: lerp_f32(a.r, b.r, t),
        g: lerp_f32(a.g, b.g, t),
        b: lerp_f32(a.b, b.b, t),
        a: lerp_f32(a.a, b.a, t),
    }
}

fn alpha_rgba(color: Rgba, alpha: f32) -> Rgba {
    Rgba {
        a: color.a * alpha,
        ..color
    }
}

fn splash_scene_width(w: f32, h: f32) -> f32 {
    w.min(h * 1.52)
}

fn splash_gradient_color(t: f32) -> Rgba {
    if t < 0.45 {
        lerp_rgba(theme::SPLASH_BG_TOP, theme::SPLASH_BG_MID, t / 0.45)
    } else if t < 0.72 {
        lerp_rgba(
            theme::SPLASH_BG_MID,
            theme::SPLASH_BG_HORIZON,
            (t - 0.45) / 0.27,
        )
    } else {
        lerp_rgba(
            theme::SPLASH_BG_HORIZON,
            theme::SPLASH_BG_BOTTOM,
            (t - 0.72) / 0.28,
        )
    }
}

fn splash_draw_bg(bounds: Bounds<gpui::Pixels>, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let strips = 180u32;
    for i in 0..strips {
        let t = i as f32 / (strips - 1) as f32;
        let strip_h = h / strips as f32;
        let strip_bounds = Bounds {
            origin: point(px(ox), px(oy + i as f32 * strip_h)),
            size: size(px(w), px(strip_h + 1.0)),
        };
        window.paint_quad(fill(strip_bounds, splash_gradient_color(t)));
    }
}

fn splash_draw_horizon_glow(bounds: Bounds<gpui::Pixels>, t: f32, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let cx = ox + w * 0.5;
    let cy = oy + h * 0.665;

    for i in 0..14 {
        let u = i as f32 / 13.0;
        let gw = w * lerp_f32(0.18, 0.92, u);
        let gh = h * lerp_f32(0.08, 0.34, u);
        let alpha = (1.0 - u).powi(2) * 0.11 * t;
        let color = if i < 5 {
            theme::SPLASH_HORIZON_GOLD
        } else {
            theme::SPLASH_HORIZON_PINK
        };
        let gb = Bounds {
            origin: point(px(cx - gw * 0.5), px(cy - gh * 0.5)),
            size: size(px(gw), px(gh)),
        };
        window.paint_quad(fill(gb, alpha_rgba(color, alpha)).corner_radii(px(gh * 0.5)));
    }
}

fn splash_draw_hills(bounds: Bounds<gpui::Pixels>, t: f32, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let offset = (1.0 - t) * h * 0.35;
    let p = |fx: f32, fy: f32| -> gpui::Point<gpui::Pixels> {
        point(px(ox + fx * w), px(oy + fy * h + offset))
    };

    // Back atmospheric ridge
    {
        let mut pb = PathBuilder::fill();
        pb.move_to(p(-0.05, 0.97));
        pb.cubic_bezier_to(p(0.50, 0.770), p(0.15, 0.920), p(0.34, 0.760));
        pb.cubic_bezier_to(p(1.05, 0.97), p(0.66, 0.760), p(0.85, 0.920));
        pb.line_to(p(1.05, 1.10));
        pb.line_to(p(-0.05, 1.10));
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_HILL_BACK, 0.72));
        }
    }

    // Left lit hill, drawn full-width so there is no center seam.
    {
        let mut pb = PathBuilder::fill();
        pb.move_to(p(-0.06, 0.905));
        pb.cubic_bezier_to(p(0.28, 0.665), p(0.06, 0.850), p(0.16, 0.690));
        pb.cubic_bezier_to(p(0.50, 0.710), p(0.38, 0.650), p(0.45, 0.690));
        pb.cubic_bezier_to(p(1.06, 0.930), p(0.66, 0.760), p(0.86, 0.900));
        pb.line_to(p(1.06, 1.10));
        pb.line_to(p(-0.06, 1.10));
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_HILL_LEFT, 0.82));
        }
    }

    // Right lit hill, also full-width to blend cleanly with the left layer.
    {
        let mut pb = PathBuilder::fill();
        pb.move_to(p(1.06, 0.905));
        pb.cubic_bezier_to(p(0.72, 0.665), p(0.94, 0.850), p(0.84, 0.690));
        pb.cubic_bezier_to(p(0.50, 0.710), p(0.62, 0.650), p(0.55, 0.690));
        pb.cubic_bezier_to(p(-0.06, 0.930), p(0.34, 0.760), p(0.14, 0.900));
        pb.line_to(p(-0.06, 1.10));
        pb.line_to(p(1.0, 1.10));
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_HILL_RIGHT, 0.78));
        }
    }

    // Front shadow ridge
    {
        let mut pb = PathBuilder::fill();
        pb.move_to(p(-0.05, 1.025));
        pb.cubic_bezier_to(p(1.05, 1.025), p(0.25, 0.885), p(0.75, 0.885));
        pb.line_to(p(1.05, 1.10));
        pb.line_to(p(-0.05, 1.10));
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_HILL_FRONT, 0.55));
        }
    }

    // Deep base, kept very low like the reference image.
    {
        let mut pb = PathBuilder::fill();
        pb.move_to(p(-0.05, 1.085));
        pb.cubic_bezier_to(p(1.05, 1.085), p(0.28, 0.985), p(0.72, 0.985));
        pb.line_to(p(1.05, 1.10));
        pb.line_to(p(-0.05, 1.10));
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_HILL_FRONT, 0.85));
        }
    }
}

fn splash_draw_sun(bounds: Bounds<gpui::Pixels>, t: f32, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let cx = ox + w * 0.5;
    let base_y = oy + h * 0.652;
    let cy = lerp_f32(oy + h, base_y, t);
    let r = (splash_scene_width(w, h) * 0.050).max(34.0);
    for i in 0..16 {
        let u = i as f32 / 15.0;
        let rm = lerp_f32(6.0, 1.35, u);
        let alpha_mult = (1.0 - u).powf(1.7) * 0.038 + 0.006;
        let base_color = if i < 9 {
            theme::SPLASH_HORIZON_PINK
        } else {
            theme::SPLASH_HORIZON_GOLD
        };
        let gr = r * rm;
        let gb = Bounds {
            origin: point(px(cx - gr), px(cy - gr)),
            size: size(px(gr * 2.0), px(gr * 2.0)),
        };
        window.paint_quad(fill(gb, alpha_rgba(base_color, alpha_mult * t)).corner_radii(px(gr)));
    }

    let soft_edge_layers = [
        (1.28, 0.22, theme::SPLASH_HORIZON_GOLD),
        (1.12, 0.34, theme::SPLASH_SUN_LAYERS[3].2),
    ];
    for (rm, alpha_mult, base_color) in soft_edge_layers {
        let gr = r * rm;
        let gb = Bounds {
            origin: point(px(cx - gr), px(cy - gr)),
            size: size(px(gr * 2.0), px(gr * 2.0)),
        };
        window.paint_quad(fill(gb, alpha_rgba(base_color, alpha_mult * t)).corner_radii(px(gr)));
    }

    let core_bounds = Bounds {
        origin: point(px(cx - r), px(cy - r)),
        size: size(px(r * 2.0), px(r * 2.0)),
    };
    window.paint_quad(fill(core_bounds, alpha_rgba(theme::SPLASH_SUN_LAYERS[4].2, t)).corner_radii(px(r)));
}

fn splash_draw_arch(bounds: Bounds<gpui::Pixels>, t: f32, window: &mut Window) {
    if t <= 0.001 {
        return;
    }
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let cx = ox + w * 0.5;
    let scene_w = splash_scene_width(w, h);
    let apex_x = cx;
    let apex_y = oy + h * 0.195;
    let base_y = oy + h * 0.680;
    let left_x = cx - scene_w * 0.205;
    let right_x = cx + scene_w * 0.205;

    // All coordinates unfold from the apex as t grows 0→1
    let fp = |x: f32, y: f32| -> gpui::Point<gpui::Pixels> {
        point(px(apex_x + (x - apex_x) * t), px(apex_y + (y - apex_y) * t))
    };

    let draw_arch = |width: f32, color: Rgba, window: &mut Window| {
        let mut pb = PathBuilder::stroke(px(width));
        pb.move_to(fp(left_x, base_y));
        pb.cubic_bezier_to(
            fp(apex_x, apex_y),
            fp(left_x + scene_w * 0.035, oy + h * 0.520),
            fp(apex_x - scene_w * 0.135, apex_y),
        );
        pb.cubic_bezier_to(
            fp(right_x, base_y),
            fp(apex_x + scene_w * 0.135, apex_y),
            fp(right_x - scene_w * 0.035, oy + h * 0.520),
        );
        if let Ok(path) = pb.build() {
            window.paint_path(path, color);
        }
    };

    let arch_width = (scene_w * 0.050).clamp(52.0, 88.0);
    draw_arch(
        arch_width * 1.85,
        alpha_rgba(theme::SPLASH_ARCH_GLOW, t * 0.040),
        window,
    );
    draw_arch(
        arch_width * 1.35,
        alpha_rgba(theme::SPLASH_ARCH_GLOW, t * 0.105),
        window,
    );
    draw_arch(
        arch_width,
        alpha_rgba(theme::SPLASH_ARCH_CORE, t.min(1.0)),
        window,
    );
}

fn splash_draw_stars(bounds: Bounds<gpui::Pixels>, t: f32, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);

    // (fx, fy, radius, delay_fraction, is_sparkle)
    let stars: &[(f32, f32, f32, f32, bool)] = &[
        (0.500, 0.380, 5.0, 0.00, true),
        (0.460, 0.295, 1.8, 0.15, false),
        (0.525, 0.275, 1.4, 0.25, false),
        (0.572, 0.345, 1.4, 0.35, false),
        (0.442, 0.418, 1.2, 0.10, false),
        (0.551, 0.448, 1.2, 0.20, false),
        (0.610, 0.318, 1.5, 0.30, false),
        (0.398, 0.362, 1.2, 0.40, false),
    ];

    for &(fx, fy, star_r, delay, is_sparkle) in stars {
        let local_t = ((t - delay) / (1.0 - delay.min(0.9))).clamp(0.0, 1.0);
        if local_t <= 0.0 {
            continue;
        }
        let cx = ox + fx * w;
        let cy = oy + fy * h;
        let alpha = local_t;

        if is_sparkle {
            splash_draw_sparkle(cx, cy, star_r, alpha, window);
        } else {
            let sb = Bounds {
                origin: point(px(cx - star_r), px(cy - star_r)),
                size: size(px(star_r * 2.0), px(star_r * 2.0)),
            };
            window.paint_quad(
                fill(sb, alpha_rgba(theme::SPLASH_STAR, alpha)).corner_radii(px(star_r)),
            );
        }
    }
}

fn splash_draw_sparkle(cx: f32, cy: f32, r: f32, alpha: f32, window: &mut Window) {
    // 4-pointed star using two thin diamond paths
    for angle_offset in [0.0_f32, std::f32::consts::PI / 4.0] {
        let mut pb = PathBuilder::fill();
        let inner = r * 0.18;
        let pts = 4usize;
        for i in 0..(pts * 2) {
            let angle = angle_offset + (i as f32 * std::f32::consts::PI) / pts as f32
                - std::f32::consts::FRAC_PI_2;
            let rad = if i % 2 == 0 { r } else { inner };
            let x = cx + angle.cos() * rad;
            let y = cy + angle.sin() * rad;
            if i == 0 {
                pb.move_to(point(px(x), px(y)));
            } else {
                pb.line_to(point(px(x), px(y)));
            }
        }
        pb.close();
        if let Ok(path) = pb.build() {
            window.paint_path(path, alpha_rgba(theme::SPLASH_STAR, alpha));
        }
    }
    // Small glow circle behind sparkle
    let glow_r = r * 1.6;
    let gb = Bounds {
        origin: point(px(cx - glow_r), px(cy - glow_r)),
        size: size(px(glow_r * 2.0), px(glow_r * 2.0)),
    };
    window.paint_quad(
        fill(gb, alpha_rgba(theme::SPLASH_STAR, alpha * 0.18)).corner_radii(px(glow_r)),
    );
}

pub fn run() {
    use std::process;
    use std::thread;

    cli::print_startup("gui");

    thread::spawn(|| {
        cli::start_loop(|| process::exit(0));
    });

    Application::new().with_assets(EmbeddedAssets).run(|app| {
        app.open_window(WindowOptions::default(), |_window, app| {
            app.new(|cx| ArcadiaRoot::new(cx))
        })
        .expect("failed to open GPUI window");
    });
}
