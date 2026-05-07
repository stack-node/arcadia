use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::Duration;

use arcadia_core::modules;
use gpui::{
    Context,
    Timer, Window, WindowAppearance,
};

use super::super::super::tui::TuiSession;
use super::super::{window_controls_top_padding, ArcadiaRoot, ShellMode};

/// Approximate character width/height for monospace text_sm (14 px font).
const CHAR_W: f32 = 8.4;
const CHAR_H: f32 = 18.0;
/// Layout overhead: shell panel p_2 (8×2) + tui_screen p_1 (4×2) + border (1×2).
const PADDING_H: f32 = 26.0;
const PADDING_V: f32 = 26.0;
/// Top-bar height: py_2 (8×2) + text_sm content (~14px) + border_b_1.
const TOP_BAR_H: f32 = 37.0;
/// Sidebar width when visible (w_64 = 256 px).
const SIDEBAR_W: f32 = 256.0;

/// Matches shell input `$` styling in `shell/panel.rs` (`rgb(0x60a5fa)` / `rgb(0x1d4ed8)`).
fn shell_history_prompt_prefix(window: &Window) -> String {
    let is_dark = matches!(
        window.appearance(),
        WindowAppearance::Dark | WindowAppearance::VibrantDark
    );
    if is_dark {
        "\x1b[38;2;96;165;250m$\x1b[0m ".to_string()
    } else {
        "\x1b[38;2;29;78;216m$\x1b[0m ".to_string()
    }
}

fn compute_tui_size(window: &Window, sidebar_visible: bool) -> (u16, u16) {
    let vp = window.viewport_size();
    let sidebar = if sidebar_visible { SIDEBAR_W } else { 0.0 };
    let chrome = window_controls_top_padding(window).to_f64() as f32;
    let w = vp.width.to_f64() as f32;
    let h = vp.height.to_f64() as f32;
    let usable_w = (w - sidebar - PADDING_H).max(CHAR_W * 40.0);
    let usable_h = (h - chrome - TOP_BAR_H - PADDING_V).max(CHAR_H * 10.0);
    ((usable_h / CHAR_H) as u16, (usable_w / CHAR_W) as u16)
}

impl ArcadiaRoot {
    pub fn sync_tui_size(&mut self, window: &Window) {
        let (rows, cols) = compute_tui_size(window, self.sidebar_visible);
        if cols != self.tui_cols || rows != self.tui_rows {
            self.tui_cols = cols;
            self.tui_rows = rows;
            if let Some(session) = &self.tui_session {
                session.resize(rows, cols);
            }
        }
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
            self.shell_output_scroll.scroll_to_bottom();
            cx.notify();
            return;
        }
        if self.shell_mode == ShellMode::Generic {
            self.spawn_tui_command(normalized, window, cx);
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
        self.shell_history.push(format!(
            "{}{command}",
            shell_history_prompt_prefix(window)
        ));
        self.shell_output_scroll.scroll_to_bottom();
        let output = match result {
            Ok(Some(output)) => output,
            Ok(None) => "Unknown shell command token.".to_string(),
            Err(err) => err,
        };
        self.shell_output_scroll.scroll_to_bottom();
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
                                this.shell_output_scroll.scroll_to_bottom();
                                cx.notify();
                            });
                        });
                    }
                    let _ = cx.update(|_, app| {
                        let _ = view.update(app, |this, cx| {
                            if this.shell_stream_nonce == stream_nonce {
                                this.shell_output_scroll.scroll_to_bottom();
                                cx.notify();
                            }
                        });
                    });
                }
            },
        )
        .detach();
    }

    fn spawn_tui_command(&mut self, command: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.tui_session = None;
        self.tui_ready = false;
        self.tui_nonce = self.tui_nonce.wrapping_add(1);
        let nonce = self.tui_nonce;

        let (rows, cols) = compute_tui_size(window, self.sidebar_visible);
        self.tui_rows = rows;
        self.tui_cols = cols;

        self.shell_history.push(format!(
            "{}{command}",
            shell_history_prompt_prefix(window)
        ));
        self.shell_output_scroll.scroll_to_bottom();

        let cwd_at_spawn = self.shell_working_dir.clone();
        match TuiSession::spawn(command, rows, cols, &cwd_at_spawn) {
            Err(e) => {
                self.shell_history.push(format!("error: {e}"));
                self.shell_output_scroll.scroll_to_bottom();
                cx.notify();
            }
            Ok(session) => {
                self.shell_display_cwd = session
                    .foreground_cwd()
                    .or_else(|| {
                        self.shell_working_dir
                            .clone()
                            .into_os_string()
                            .into_string()
                            .ok()
                    })
                    .unwrap_or_else(|| "cwd: unavailable".to_string());
                let parser = session.parser.clone();
                let queue = session.queue.clone();
                let done = session.done.clone();
                self.tui_scroll.scroll_to_bottom();
                self.tui_session = Some(session);
                cx.notify();

                let command_owned = command.to_string();
                cx.spawn_in(
                    window,
                    move |view: gpui::WeakEntity<ArcadiaRoot>,
                          cx: &mut gpui::AsyncWindowContext| {
                        let mut cx = cx.clone();
                        async move {
                            let mut showed_tui = false;
                            loop {
                                Timer::after(Duration::from_millis(16)).await;

                                let chunks: Vec<Vec<u8>> = queue
                                    .lock()
                                    .map(|mut q| q.drain(..).collect())
                                    .unwrap_or_default();
                                let is_done = done.load(Ordering::SeqCst);

                                let _ = cx.update(|_, app| {
                                    let _ = view.update(app, |this, cx| {
                                        if this.tui_nonce != nonce {
                                            return;
                                        }
                                        if let Some(ref sess) = this.tui_session {
                                            if let Some(cwd) = sess.foreground_cwd() {
                                                if cwd != this.shell_display_cwd {
                                                    this.shell_display_cwd = cwd.clone();
                                                    this.shell_working_dir = PathBuf::from(cwd);
                                                    this.tui_scroll.scroll_to_bottom();
                                                    cx.notify();
                                                }
                                            }
                                        }
                                    });
                                });

                                if !is_done && !showed_tui {
                                    showed_tui = true;
                                    let _ = cx.update(|_, app| {
                                        let _ = view.update(app, |this, cx| {
                                            if this.tui_nonce == nonce {
                                                this.tui_ready = true;
                                                this.tui_scroll.scroll_to_bottom();
                                                cx.notify();
                                            }
                                        });
                                    });
                                }

                                if !chunks.is_empty() {
                                    if let Ok(mut p) = parser.lock() {
                                        for chunk in &chunks {
                                            p.process(chunk);
                                        }
                                    }
                                    let _ = cx.update(|_, app| {
                                        let _ = view.update(app, |this, cx| {
                                            this.tui_scroll.scroll_to_bottom();
                                            cx.notify();
                                        });
                                    });
                                }

                                if is_done && chunks.is_empty() {
                                            let screen_lines: Vec<String> = parser
                                        .lock()
                                        .map(|p| {
                                            let screen = p.screen();
                                            let (rows, cols) = screen.size();
                                            (0..rows)
                                                .filter_map(|r| {
                                                    crate::gui::tui::vt100_row_for_shell_history(
                                                        screen, r, cols,
                                                    )
                                                })
                                                .collect()
                                        })
                                        .unwrap_or_default();
                                    let _ = cx.update(|_, app| {
                                        let _ = view.update(app, |this, cx| {
                                            if this.tui_nonce == nonce {
                                                for line in screen_lines {
                                                    this.shell_history.push(line);
                                                }
                                                if let Some(ref sess) = this.tui_session {
                                                    let from_fg =
                                                        sess.foreground_cwd().map(PathBuf::from);
                                                    let from_cd = crate::gui::tui::resolve_simple_cd(
                                                        &cwd_at_spawn,
                                                        &command_owned,
                                                    );
                                                    if let Some(p) = from_fg.or(from_cd) {
                                                        this.shell_working_dir = p.clone();
                                                        this.shell_display_cwd =
                                                            p.to_string_lossy().into_owned();
                                                    }
                                                }
                                                this.tui_session = None;
                                                this.shell_output_scroll.scroll_to_bottom();
                                                cx.notify();
                                            }
                                        });
                                    });
                                    break;
                                }
                            }
                        }
                    },
                )
                .detach();
            }
        }
    }
}
