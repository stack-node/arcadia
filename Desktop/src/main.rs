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
        cli::print_startup("gui");

        thread::spawn(|| {
            cli::start_loop(|| process::exit(0));
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
    use crate::cli;

    pub fn run() {
        cli::print_startup("headless");
        cli::start_loop(|| {});
    }
}
