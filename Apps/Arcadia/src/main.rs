mod config;
mod platform;

fn main() {
    #[cfg(feature = "gui")]
    {
        gui::run();
        return;
    }

    #[cfg(not(feature = "gui"))]
    headless::run();
}

mod cli {
    use std::io::{self, Write};
    use std::sync::OnceLock;

    use crate::config::commandline::CommandlineConfig;

    pub enum CommandResult {
        Continue,
        Quit,
    }

    fn settings() -> &'static CommandlineConfig {
        static SETTINGS: OnceLock<CommandlineConfig> = OnceLock::new();
        SETTINGS.get_or_init(|| match CommandlineConfig::load_or_create() {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Failed to load commandline config; using defaults: {err}");
                CommandlineConfig::default()
            }
        })
    }

    pub fn print_prompt() {
        let cfg = settings();
        print!("{}{}\x1b[0m ", cfg.input_ansi_code(), cfg.input_symbol);
        let _ = io::stdout().flush();
    }

    pub fn print_response(message: &str) {
        let cfg = settings();
        println!("{}{}\x1b[0m {message}", cfg.output_ansi_code(), cfg.output_symbol);
    }

    pub fn handle(input: &str) -> CommandResult {
        match input.trim() {
            "ping" => {
                print_response("pong");
                CommandResult::Continue
            }
            "quit" => CommandResult::Quit,
            "" => CommandResult::Continue,
            other => {
                print_response(&format!("Unknown command: {other}"));
                CommandResult::Continue
            }
        }
    }
}

#[cfg(feature = "gui")]
mod gui {
    use std::io;
    use std::process;
    use std::thread;

    use crate::cli;
    use gpui::{
        AppContext, Application, Context, IntoElement, ParentElement, Render, SharedString, Styled,
        Window, WindowOptions, div, white,
    };

    struct ArcadiaRoot {
        title: SharedString,
    }

    impl Render for ArcadiaRoot {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .bg(white())
                .flex()
                .justify_center()
                .items_center()
                .text_3xl()
                .child(self.title.clone())
        }
    }

    pub fn run() {
        thread::spawn(|| {
            let stdin = io::stdin();
            let mut buffer = String::new();

            loop {
                cli::print_prompt();
                buffer.clear();
                match stdin.read_line(&mut buffer) {
                    Ok(0) => break,
                    Ok(_) => {
                        if let cli::CommandResult::Quit = cli::handle(&buffer) {
                            process::exit(0);
                        }
                    }
                    Err(err) => {
                        eprintln!("CLI input error: {err}");
                        break;
                    }
                }
            }
        });

        Application::new().run(|app| {
            app.open_window(WindowOptions::default(), |_window, app| {
                app.new(|_cx| ArcadiaRoot {
                    title: SharedString::new_static("Arcadia"),
                })
            })
            .expect("failed to open GPUI window");
        });
    }
}

#[cfg(not(feature = "gui"))]
mod headless {
    use std::io;

    use crate::cli;
    use crate::platform;
    use crate::platform::PlatformInfo;

    pub fn run() {
        println!("Arcadia base app");
        println!("Detected platform: {}", platform::current().name());
        println!("Mode: headless");
        println!("Status: bootstrap complete");
        println!("CLI ready. Commands: ping, quit");

        let stdin = io::stdin();
        let mut buffer = String::new();

        loop {
            cli::print_prompt();
            buffer.clear();
            match stdin.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => {
                    if let cli::CommandResult::Quit = cli::handle(&buffer) {
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("CLI input error: {err}");
                    break;
                }
            }
        }
    }
}
