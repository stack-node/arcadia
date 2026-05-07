use crate::cli;
use gpui::{Context, KeyDownEvent, Window};

use super::super::super::tui;
use super::super::ArcadiaRoot;

impl ArcadiaRoot {
    pub(crate) fn handle_shell_key_down(
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

        // When a TUI session is active, forward all keys to the PTY.
        if self.tui_session.is_some() {
            let bytes = tui::key_to_bytes(key, mods).or_else(|| {
                if !mods.control && !mods.alt && !mods.platform {
                    event
                        .keystroke
                        .key_char
                        .as_ref()
                        .map(|c| c.as_bytes().to_vec())
                } else {
                    None
                }
            });
            if let (Some(bytes), Some(session)) = (bytes, self.tui_session.as_mut()) {
                session.write_input(&bytes);
            }
            self.tui_scroll.scroll_to_bottom();
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

    pub(crate) fn handle_global_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.key.as_str() == "escape" && self.app_menu_open {
            self.app_menu_open = false;
            cx.notify();
            return;
        }
        if self.active_page_id != "utility.shell" {
            return;
        }
        let key = event.keystroke.key.as_str();
        let mods = event.keystroke.modifiers;
        if key == "tab" && mods.shift && self.tui_session.is_none() {
            self.shell_mode = self.shell_mode.toggle();
            cx.notify();
        }
    }

    pub(crate) fn run_internal_quit_command(&mut self) {
        if let crate::cli::CommandResult::Quit = cli::handle("quit") {
            std::process::exit(0);
        }
    }
}
