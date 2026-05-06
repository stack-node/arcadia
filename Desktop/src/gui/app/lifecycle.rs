use std::time::Duration;

use arcadia_core::config::modules::{ModulesConfig, SHELL_MODULE_NAME, SHELL_MOTD_MODULE_NAME};
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::shell_motd;
use arcadia_core::navigation;
use gpui::{Context, Timer, Window};

use super::ArcadiaRoot;

impl ArcadiaRoot {
    pub(super) fn reset_shell_state(&mut self) {
        self.shell_stream_nonce = self.shell_stream_nonce.wrapping_add(1);
        self.shell_history = Self::initial_shell_history();
        self.shell_input.clear();
        self.shell_cursor = 0;
        self.shell_history_index = None;
        self.shell_output_scroll.scroll_to_bottom();
    }

    fn initial_shell_history() -> Vec<String> {
        let Ok(cfg) = ModulesConfig::load_or_create() else {
            return vec!["Arcadia Terminal ready.".to_string()];
        };
        let shell_on = cfg.modules.get(SHELL_MODULE_NAME).copied().unwrap_or(false);
        let motd_on = cfg
            .modules
            .get(SHELL_MOTD_MODULE_NAME)
            .copied()
            .unwrap_or(false);
        if shell_on && motd_on {
            shell_motd::motd_lines()
        } else {
            vec!["Arcadia Terminal ready.".to_string()]
        }
    }

    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        let shell_focus = cx.focus_handle();
        let module_rows = ModulesConfig::load_or_create()
            .map(|cfg| cfg.modules.into_iter().collect::<Vec<(String, bool)>>())
            .unwrap_or_default();
        ArcadiaRoot {
            title: gpui::SharedString::new_static("Arcadia"),
            active_page_id: navigation::DEFAULT_PAGE_ID,
            active_group_id: navigation::DEFAULT_GROUP_ID,
            module_rows,
            pending_module_enable: None,
            shell_history: Self::initial_shell_history(),
            shell_input: String::new(),
            shell_focus,
            shell_cursor: 0,
            shell_command_history: Vec::new(),
            shell_history_index: None,
            shell_caret_visible: true,
            shell_caret_task_started: false,
            shell_stream_nonce: 0,
            shell_output_scroll: gpui::ScrollHandle::new(),
            shell_mode: super::ShellMode::Generic,
            tui_session: None,
            tui_nonce: 0,
            tui_ready: false,
            splash_elapsed_ms: 0.0,
            splash_tick_started: false,
            sidebar_visible: true,
            app_menu_open: false,
        }
    }

    pub fn reload_modules(&mut self) {
        self.module_rows = ModulesConfig::load_or_create()
            .map(|cfg| cfg.modules.into_iter().collect())
            .unwrap_or_default();
        self.ensure_valid_navigation_selection();
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
                                    if !this.is_module_enabled(SHELL_MODULE_NAME) {
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
}
