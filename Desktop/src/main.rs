mod cli;

use arcadia_core::modules;

fn main() {
    modules::load_all();

    #[cfg(feature = "gui")]
    {
        gui::run();
        modules::shutdown_all();
        return;
    }

    #[cfg(not(feature = "gui"))]
    {
        headless::run();
        modules::shutdown_all();
    }
}

#[cfg(feature = "gui")]
mod gui {
    use std::process;
    use std::time::Duration;
    use std::thread;

    use crate::cli;
    use arcadia_core::modules;
    use arcadia_core::config::modules::ModulesConfig;
    use arcadia_core::config::ConfigFile;
    use arcadia_core::navigation;
    use gpui::{
        div, rgb, AppContext, Application, Context, FocusHandle, InteractiveElement,
        IntoElement, KeyDownEvent, ParentElement, Render, SharedString, StatefulInteractiveElement,
        Styled, Timer, Window, WindowAppearance, WindowOptions,
    };

    struct ArcadiaRoot {
        title: SharedString,
        active_page_id: &'static str,
        active_group_id: &'static str,
        module_rows: Vec<(String, bool)>,
        pending_module_enable: Option<(String, Vec<String>)>,
        shell_enabled: bool,
        shell_history: Vec<String>,
        shell_input: String,
        shell_focus: FocusHandle,
        shell_cursor: usize,
        shell_command_history: Vec<String>,
        shell_history_index: Option<usize>,
        shell_caret_visible: bool,
        shell_caret_task_started: bool,
        shell_stream_nonce: u64,
    }

    impl Render for ArcadiaRoot {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                .bg(if is_dark { rgb(0x0f1115) } else { rgb(0xffffff) })
                .flex()
                .child(
                    div()
                        .h_full()
                        .w_64()
                        .flex()
                        .flex_col()
                        .p_4()
                        .gap_2()
                        .bg(if is_dark { rgb(0x171b22) } else { rgb(0xf6f7fb) })
                        .border_r_1()
                        .border_color(if is_dark { rgb(0x2a3340) } else { rgb(0xe6e8ef) })
                        .child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(if is_dark { rgb(0xe5e7eb) } else { rgb(0x111827) })
                                .child("Arcadia"),
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
                        })),
                )
                .child(if self.active_page_id == "utility.shell" {
                    div()
                        .flex_1()
                        .h_full()
                        .p_2()
                        .child(self.shell_panel(window, cx))
                } else {
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
                                .text_color(if is_dark { rgb(0xe5e7eb) } else { rgb(0x1f2937) })
                                .child(active_page.map_or("Page", |page| page.title)),
                        )
                        .child(
                            div().text_base().text_color(if is_dark { rgb(0x9ca3af) } else { rgb(0x6b7280) }).child(
                                active_page
                                    .map_or("Page definition not found.", |page| page.description),
                            ),
                        )
                        .child(self.modules_panel(cx, is_dark))
                })
                .child(self.requirements_modal(cx, is_dark))
        }
    }

    impl ArcadiaRoot {
        fn render_glyph(glyph_key: &'static str) -> &'static str {
            match glyph_key {
                "group_one" => "◼",
                "group_two" => "◻",
                "grid" => "⊞",
                "queue" => "☰",
                "alert" => "⚠",
                "metrics" => "◷",
                "terminal" => ">_",
                "tools" => "🛠",
                "home" => "⌂",
                "logs" => "☷",
                "settings" => "⛭",
                "modules" => "⌘",
                _ => "•",
            }
        }

        fn modules_panel(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
            if self.active_page_id != "global.modules" {
                return div();
            }

            let panel = div()
                .mt_4()
                .w_full()
                .p_4()
                .rounded_lg()
                .bg(if is_dark { rgb(0x151a22) } else { rgb(0xf8fafc) })
                .border_1()
                .border_color(if is_dark { rgb(0x2f3948) } else { rgb(0xe2e8f0) })
                .flex()
                .flex_col()
                .gap_3()
                .children(self.module_rows.iter().map(|(module_name, enabled)| {
                    Self::module_row_item(cx, module_name.clone(), *enabled, is_dark)
                }));
            panel
        }

        fn shell_panel(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                .bg(if is_dark { rgb(0x151a22) } else { rgb(0xf8fafc) })
                .border_1()
                .border_color(if is_dark { rgb(0x2f3948) } else { rgb(0xe2e8f0) })
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
                        .bg(if is_dark { rgb(0x0f141b) } else { rgb(0xffffff) })
                        .border_b_1()
                        .border_color(if is_dark { rgb(0x2f3948) } else { rgb(0xe2e8f0) })
                        .child(div().text_sm().text_color(if is_dark { rgb(0xe5e7eb) } else { rgb(0x111827) }).child("Terminal"))
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .cursor_pointer()
                                .bg(if is_dark { rgb(0x1f2937) } else { rgb(0xf3f4f6) })
                                .text_color(if is_dark { rgb(0xd1d5db) } else { rgb(0x374151) })
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
                            div().text_sm().text_color(if is_dark { rgb(0xe5e7eb) } else { rgb(0x1f2937) }).child(line.clone())
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
                        } else {
                            if is_dark { rgb(0x2f3948) } else { rgb(0xe2e8f0) }
                        })
                        .bg(if is_dark { rgb(0x0f141b) } else { rgb(0xffffff) })
                        .track_focus(&self.shell_focus)
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _, window, _| {
                                this.shell_focus.focus(window);
                            }),
                        )
                        .on_key_down(cx.listener(Self::handle_shell_key_down))
                        .child(div().text_sm().text_color(if is_dark { rgb(0x60a5fa) } else { rgb(0x1d4ed8) }).child("$"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(if is_dark { rgb(0xe5e7eb) } else { rgb(0x111827) })
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

        fn ensure_shell_caret_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
                            let _ = cx.update(|_, app| {
                                let _ = view.update(app, |this, cx| {
                                    this.shell_caret_visible = !this.shell_caret_visible;
                                    cx.notify();
                                });
                            });
                        }
                    }
                },
            )
                .detach();
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
                    let mods = event.keystroke.modifiers;
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

        fn requirements_modal(&self, cx: &mut Context<Self>, is_dark: bool) -> impl IntoElement {
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
                                    div().text_sm().text_color(if is_dark { rgb(0xd1d5db) } else { rgb(0x374151) }).child(format!(
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

        fn shell_action_button(
            cx: &mut Context<Self>,
            label: &'static str,
            command: &'static str,
        ) -> impl IntoElement {
            div()
                .px_3()
                .py_2()
                .rounded_md()
                .cursor_pointer()
                .bg(rgb(0x1f2937))
                .text_color(rgb(0xe5e7eb))
                .child(label)
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, window, cx| {
                        this.run_shell_execute(command, window, cx);
                        cx.notify();
                    }),
                )
        }

        fn shell_toolbar(&self, _cx: &mut Context<Self>) -> impl IntoElement { div() }

        fn module_row_item(
            cx: &mut Context<Self>,
            module_name: String,
            enabled: bool,
            is_dark: bool,
        ) -> impl IntoElement {
            let label = if enabled { "Disable" } else { "Enable" };
            div()
                .w_full()
                .px_3()
                .py_2()
                .rounded_md()
                .bg(if is_dark { rgb(0x111827) } else { rgb(0xffffff) })
                .border_1()
                .border_color(if is_dark { rgb(0x374151) } else { rgb(0xe2e8f0) })
                .flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(if is_dark { rgb(0xe5e7eb) } else { rgb(0x111827) })
                        .child(module_name.clone()),
                )
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .cursor_pointer()
                        .bg(if enabled {
                            rgb(0xfee2e2)
                        } else {
                            rgb(0xdcfce7)
                        })
                        .text_color(if enabled {
                            rgb(0x991b1b)
                        } else {
                            rgb(0x166534)
                        })
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
                                            let _ = cli::handle(&format!(
                                                "module {module_name} enable"
                                            ));
                                            this.pending_module_enable = None;
                                            this.reload_modules();
                                        }
                                        Err(err) => {
                                            eprintln!("{err}");
                                        }
                                    },
                                    Err(err) => {
                                        eprintln!("{err}");
                                    }
                                }
                                cx.notify();
                            }),
                        ),
                )
        }

        fn reload_modules(&mut self) {
            self.module_rows = ModulesConfig::load_or_create()
                .map(|cfg| cfg.modules.into_iter().collect())
                .unwrap_or_default();
            self.shell_enabled = self
                .module_rows
                .iter()
                .find(|(name, _)| name == "shell")
                .map(|(_, enabled)| *enabled)
                .unwrap_or(false);
            self.ensure_valid_navigation_selection();
        }

        fn run_shell_execute(&mut self, command: &str, window: &mut Window, cx: &mut Context<Self>) {
            let normalized = command.trim();
            if normalized.eq_ignore_ascii_case("clear") || normalized.eq_ignore_ascii_case("cls") {
                self.shell_stream_nonce = self.shell_stream_nonce.wrapping_add(1);
                self.shell_history.clear();
                cx.notify();
                return;
            }
            let args = vec![command];
            let result = modules::execute_command(
                "shell.execute",
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
            let chars = output.chars().collect::<Vec<_>>();
            self.shell_history.push(String::new());
            cx.spawn_in(
                window,
                move |view: gpui::WeakEntity<ArcadiaRoot>, cx: &mut gpui::AsyncWindowContext| {
                    let mut cx = cx.clone();
                    async move {
                        for ch in chars {
                            Timer::after(Duration::from_millis(10)).await;
                            let _ = cx.update(|_, app| {
                                let _ = view.update(app, |this, cx| {
                                    if this.shell_stream_nonce != stream_nonce {
                                        return;
                                    }
                                    if ch == '\n' {
                                        this.shell_history.push(String::new());
                                    } else if let Some(current_line) = this.shell_history.last_mut() {
                                        current_line.push(ch);
                                    }
                                    cx.notify();
                                });
                            });
                        }
                        let _ = cx.update(|_, app| {
                            let _ = view.update(app, |this, cx| {
                                if this.shell_stream_nonce != stream_nonce {
                                    return;
                                }
                                this.shell_history.push(String::new());
                                cx.notify();
                            });
                        });
                    }
                },
            )
                .detach();
        }

        fn is_page_visible(&self, page_id: &str) -> bool {
            match page_id {
                "utility.shell" => self.shell_enabled,
                _ => true,
            }
        }

        fn active_page_if_visible(&self) -> Option<&'static navigation::NavigationPageDefinition> {
            if self.is_page_visible(self.active_page_id) {
                Self::page_by_id(self.active_page_id)
            } else {
                None
            }
        }

        fn visible_groups(&self) -> Vec<&'static navigation::NavigationGroupDefinition> {
            navigation::GROUP_DEFINITIONS
                .iter()
                .filter(|group| group.pages.iter().any(|page_id| self.is_page_visible(page_id)))
                .collect()
        }

        fn ensure_valid_navigation_selection(&mut self) {
            let visible_groups = self.visible_groups();
            let group_is_visible = visible_groups.iter().any(|group| group.id == self.active_group_id);
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
                    if let Some(first_visible_page) =
                        group.pages.iter().find(|page_id| self.is_page_visible(page_id))
                    {
                        self.active_page_id = first_visible_page;
                    }
                }
            }
        }

        fn page_by_id(
            page_id: &'static str,
        ) -> Option<&'static navigation::NavigationPageDefinition> {
            navigation::page_by_id(page_id)
        }

        fn group_by_id(group_id: &'static str) -> &'static navigation::NavigationGroupDefinition {
            navigation::group_by_id(group_id).unwrap_or(&navigation::GROUP_DEFINITIONS[0])
        }

        fn sidebar_group_item(
            cx: &mut Context<Self>,
            label: &'static str,
            system_image: &'static str,
            group_id: &'static str,
            is_active: bool,
            is_dark: bool,
        ) -> impl IntoElement {
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
                    if is_dark { rgb(0x1f2a3e) } else { rgb(0xe1e7ff) }
                } else {
                    if is_dark { rgb(0x171b22) } else { rgb(0xf6f7fb) }
                })
                .text_color(if is_active {
                    if is_dark { rgb(0x93c5fd) } else { rgb(0x1d4ed8) }
                } else {
                    if is_dark { rgb(0xd1d5db) } else { rgb(0x374151) }
                })
                .hover(move |style| style.bg(if is_dark { rgb(0x243246) } else { rgb(0xeef2ff) }))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_1()
                        .text_center()
                        .child(div().child(Self::render_glyph(system_image)))
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

        fn sidebar_global_item(
            cx: &mut Context<Self>,
            label: &'static str,
            system_image: &'static str,
            page_id: &'static str,
            is_active: bool,
            is_dark: bool,
        ) -> impl IntoElement {
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
                    if is_dark { rgb(0x1f2a3e) } else { rgb(0xe1e7ff) }
                } else {
                    if is_dark { rgb(0x171b22) } else { rgb(0xf6f7fb) }
                })
                .text_color(if is_active {
                    if is_dark { rgb(0x93c5fd) } else { rgb(0x1d4ed8) }
                } else {
                    if is_dark { rgb(0xd1d5db) } else { rgb(0x374151) }
                })
                .hover(move |style| style.bg(if is_dark { rgb(0x243246) } else { rgb(0xeef2ff) }))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .items_center()
                        .child(div().child(Self::render_glyph(system_image)))
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

        fn sidebar_item(
            cx: &mut Context<Self>,
            label: &'static str,
            system_image: &'static str,
            page_id: &'static str,
            is_active: bool,
            is_dark: bool,
        ) -> impl IntoElement {
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
                    if is_dark { rgb(0x1f2a3e) } else { rgb(0xe1e7ff) }
                } else {
                    if is_dark { rgb(0x171b22) } else { rgb(0xf6f7fb) }
                })
                .text_color(if is_active {
                    if is_dark { rgb(0x93c5fd) } else { rgb(0x1d4ed8) }
                } else {
                    if is_dark { rgb(0xd1d5db) } else { rgb(0x374151) }
                })
                .hover(move |style| style.bg(if is_dark { rgb(0x243246) } else { rgb(0xeef2ff) }))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .items_center()
                        .child(div().child(Self::render_glyph(system_image)))
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

    pub fn run() {
        cli::print_startup("gui");

        thread::spawn(|| {
            cli::start_loop(|| process::exit(0));
        });

        Application::new().run(|app| {
            app.open_window(WindowOptions::default(), |_window, app| {
                app.new(|_cx| {
                    let shell_focus = _cx.focus_handle();
                    let module_rows = ModulesConfig::load_or_create()
                        .map(|cfg| cfg.modules.into_iter().collect::<Vec<(String, bool)>>())
                        .unwrap_or_default();
                    let shell_enabled = module_rows
                        .iter()
                        .find(|(name, _)| name == "shell")
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
                    }
                })
            })
            .expect("failed to open GPUI window");
        });
    }
}

#[cfg(not(feature = "gui"))]
mod headless {
    use crate::cli;

    pub fn run() {
        cli::print_startup("headless");
        cli::start_loop(|| {});
    }
}
