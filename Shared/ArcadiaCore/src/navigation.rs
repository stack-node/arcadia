use serde::Serialize;

use crate::config::modules::{LAN_MODULE_NAME, NET_MODULE_NAME, SHELL_MODULE_NAME};

#[derive(Clone, Copy, Serialize)]
pub struct NavigationPageDefinition {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub glyph: &'static str,
    pub system_image: &'static str,
    /// Theme key for sidebar-selected fills and icon tint (`Desktop/src/gui/theme.rs`, `AppTheme` on iOS).
    pub accent: &'static str,
    /// When set, the page is shown only if this module is enabled (`MODULE_REGISTRY` name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_module: Option<&'static str>,
}

#[derive(Clone, Copy, Serialize)]
pub struct NavigationGroupDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub glyph: &'static str,
    pub system_image: &'static str,
    pub pages: &'static [&'static str],
    pub accent: &'static str,
}

#[derive(Serialize)]
pub struct NavigationRegistry {
    pub pages: Vec<NavigationPageDefinition>,
    pub groups: Vec<NavigationGroupDefinition>,
    pub global_pages: Vec<&'static str>,
    pub default_group: &'static str,
    pub default_page: &'static str,
}

pub const PAGE_DEFINITIONS: &[NavigationPageDefinition] = &[
    NavigationPageDefinition {
        id: "utility.shell",
        title: "Shell",
        description: "Run and manage shell utility actions.",
        glyph: "terminal",
        system_image: "terminal",
        accent: "emerald",
        required_module: Some(SHELL_MODULE_NAME),
    },
    NavigationPageDefinition {
        id: "global.dashboard",
        title: "Dashboard",
        description: "Overview of the Arcadia application surface.",
        glyph: "home",
        system_image: "house",
        accent: "violet",
        required_module: None,
    },
    NavigationPageDefinition {
        id: "global.logs",
        title: "Logs",
        description: "Recent logs and activity stream appear here.",
        glyph: "logs",
        system_image: "doc.text.magnifyingglass",
        accent: "sky",
        required_module: None,
    },
    NavigationPageDefinition {
        id: "global.settings",
        title: "Settings",
        description: "App preferences and configuration controls appear here.",
        glyph: "settings",
        system_image: "gearshape",
        accent: "indigo",
        required_module: None,
    },
    NavigationPageDefinition {
        id: "global.modules",
        title: "Modules",
        description: "Manage global module availability and dependency requirements.",
        glyph: "modules",
        system_image: "switch.2",
        accent: "fuchsia",
        required_module: None,
    },
    NavigationPageDefinition {
        id: "network.overview",
        title: "Overview",
        description: "Network status and module connectivity overview.",
        glyph: "logs",
        system_image: "network",
        accent: "teal",
        required_module: Some(NET_MODULE_NAME),
    },
    NavigationPageDefinition {
        id: "network.nodes",
        title: "Nodes",
        description: "Discover LAN peers and manage pairing with lan.scan / lan.node.",
        glyph: "nodes",
        system_image: "rectangle.connected.to.line.under.fill",
        accent: "cyan",
        required_module: Some(LAN_MODULE_NAME),
    },
];

pub const GROUP_DEFINITIONS: &[NavigationGroupDefinition] = &[
    NavigationGroupDefinition {
        id: "utilities",
        label: "Utilities",
        glyph: "tools",
        system_image: "wrench.and.screwdriver",
        pages: &["utility.shell"],
        accent: "amber",
    },
    NavigationGroupDefinition {
        id: "network",
        label: "Network",
        glyph: "logs",
        system_image: "network",
        pages: &["network.overview", "network.nodes"],
        accent: "cyan",
    },
];

pub const GLOBAL_PAGE_IDS: &[&str] = &["global.dashboard", "global.settings", "global.modules"];
pub const DEFAULT_GROUP_ID: &str = "utilities";
pub const DEFAULT_PAGE_ID: &str = "global.dashboard";

pub fn page_by_id(page_id: &str) -> Option<&'static NavigationPageDefinition> {
    PAGE_DEFINITIONS.iter().find(|page| page.id == page_id)
}

pub fn group_by_id(group_id: &str) -> Option<&'static NavigationGroupDefinition> {
    GROUP_DEFINITIONS.iter().find(|group| group.id == group_id)
}

pub fn default_navigation_registry() -> NavigationRegistry {
    NavigationRegistry {
        pages: PAGE_DEFINITIONS.to_vec(),
        groups: GROUP_DEFINITIONS.to_vec(),
        global_pages: GLOBAL_PAGE_IDS.to_vec(),
        default_group: DEFAULT_GROUP_ID,
        default_page: DEFAULT_PAGE_ID,
    }
}

pub fn default_navigation_registry_json() -> String {
    serde_json::to_string(&default_navigation_registry())
        .expect("navigation registry serialization should always succeed")
}
