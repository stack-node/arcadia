mod cli;

use arcadia_core::modules;

fn main() {
    modules::load_all();
    arcadia_core::modules::shell::set_internal_executor(cli::handle_internal);

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
mod gui;

#[cfg(not(feature = "gui"))]
mod headless {
    use crate::cli;

    pub fn run() {
        cli::print_startup("headless");
        cli::start_loop(|| {});
    }
}
