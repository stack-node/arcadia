//! LAN routing gate (`execute_command` with `net_as: lan:…`). No dedicated mirror verbs — use [`crate::modules::surface`].

use crate::modules::ModuleCommand;

pub const NAME: &str = "remote-session";

pub fn commands() -> &'static [ModuleCommand] {
    &[]
}
