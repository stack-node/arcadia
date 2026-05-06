use std::sync::atomic::Ordering;
use std::time::Duration;

use arcadia_core::modules;
use gpui::{
    Context, Styled,
    Timer, Window,
};

use super::super::super::tui::{self, TuiSession};
use super::super::{ArcadiaRoot, ShellMode};

impl ArcadiaRoot {
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
        self.shell_history.push(format!("$ {command}"));
        self.shell_output_scroll.scroll_to_bottom();
        let output = match result {
            Ok(Some(output)) => output,
            Ok(None) => "Unknown shell command token.".to_string(),
            Err(err) => err,
        };
        self.shell_history.push(String::new());
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
                                this.shell_history.push(String::new());
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

        match TuiSession::spawn(command, tui::DEFAULT_ROWS, tui::DEFAULT_COLS) {
            Err(e) => {
                self.shell_history.push(format!("$ {command}"));
                self.shell_history.push(format!("error: {e}"));
                self.shell_output_scroll.scroll_to_bottom();
                cx.notify();
            }
            Ok(session) => {
                let parser = session.parser.clone();
                let queue = session.queue.clone();
                let done = session.done.clone();
                self.tui_session = Some(session);
                cx.notify();

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

                                if !is_done && !showed_tui {
                                    showed_tui = true;
                                    let _ = cx.update(|_, app| {
                                        let _ = view.update(app, |this, cx| {
                                            if this.tui_nonce == nonce {
                                                this.tui_ready = true;
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
                                        let _ = view.update(app, |_this, cx| cx.notify());
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
                                                    let mut line = String::new();
                                                    for c in 0..cols {
                                                        match screen.cell(r, c) {
                                                            Some(cell) => {
                                                                let s = cell.contents();
                                                                if s.is_empty() {
                                                                    line.push(' ');
                                                                } else {
                                                                    line.push_str(&s);
                                                                }
                                                            }
                                                            None => line.push(' '),
                                                        }
                                                    }
                                                    let trimmed = line.trim_end().to_string();
                                                    if trimmed.is_empty() {
                                                        None
                                                    } else {
                                                        Some(trimmed)
                                                    }
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
                                                this.tui_session = None;
                                                this.shell_history.push(String::new());
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
