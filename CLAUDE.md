# Arcadia — Claude Code Guide

## What Arcadia Is

Multi-platform runtime and shell: one Rust core (`Shared/ArcadiaCore`) consumed by two thin surface shells — a GPUI desktop app (`Desktop/`) and a SwiftUI iOS app (`Mobile/iOS/`). The core owns all logic; surfaces only render and dispatch.

## Repository Layout

```
Shared/ArcadiaCore/src/
  config/modules.rs     MODULE_REGISTRY — single source of truth for all modules
  navigation.rs         PAGE_DEFINITIONS, GROUP_DEFINITIONS — single source of truth for all nav
  modules/              per-module command handlers (shell.rs, lan/, net.rs, …)
  ffi.rs                UniFFI bridge — Rust → Swift for iOS

Desktop/src/
  cli/                  REPL loop, command dispatch, config/module CLI handlers
  gui/                  GPUI view (app.rs), theme helpers (theme.rs), embedded assets

Mobile/iOS/ArcadiaApp/
  NavigationModels.swift  Swift mirror of NavigationRegistry (deserialized from JSON)
  AppTheme.swift          all colors via computed properties — no raw hex elsewhere
  ContentView.swift       top-level coordinator
  SidebarView.swift       sidebar rendering
  ShellView.swift         terminal UI
  ModulesView.swift       module toggle rows

Configuration/
  modules.toml          user module enable/disable state
  commandline.toml      CLI settings
```

## Core Architecture Principles

### Single source of truth — never duplicate

| What | Lives in | Consumed by |
|------|----------|-------------|
| Module list + deps | `config/modules.rs` `MODULE_REGISTRY` | everything |
| Navigation pages/groups | `navigation.rs` `PAGE_DEFINITIONS` / `GROUP_DEFINITIONS` | Desktop gui, iOS via JSON |
| Colors / theme | `gui/theme.rs` + `AppTheme.swift` | their respective surfaces |
| Config schema | `config/modules.rs` `ModulesConfig` | CLI, GUI, iOS |

### Non-monolithic — thin surfaces, fat core

Surface code (Desktop `gui/`, iOS `ArcadiaApp/`) must:
- **Read** from registries and configs
- **Render** what the registry says
- **Dispatch** user actions to `arcadia_core`

Surface code must NOT:
- Re-implement business logic that belongs in `arcadia_core`
- Hard-code module names, page IDs, or feature flags in render/layout logic
- Add per-module booleans (`shell_enabled`, `net_enabled`) — query the config dynamically
- Duplicate navigation structure that already exists in `navigation.rs`

## How to Add Things

### New module
1. Add a `ModuleManifest` entry to `MODULE_REGISTRY` in `Shared/ArcadiaCore/src/config/modules.rs`
2. Add `pub const X_MODULE_NAME: &str = "x";` constant in the same file
3. Create `Shared/ArcadiaCore/src/modules/x.rs` with a `commands()` fn returning `&[ModuleCommand]`
4. Register it in `Shared/ArcadiaCore/src/modules/mod.rs`
5. That's it — GUI, CLI, and iOS all pick it up from the registry automatically

### New navigation page
1. Add a `NavigationPageDefinition` entry to `PAGE_DEFINITIONS` in `navigation.rs`
2. Add the page ID to the relevant `NavigationGroupDefinition.pages` slice, or create a new `NavigationGroupDefinition`
3. If the page should only show when a module is enabled, the page definition should carry that information — **do not add a new match arm in `is_page_visible`**
4. Implement the page panel in the surface (Desktop `app.rs`, iOS view file)
5. Route it in the surface's content switch — but derive the condition from module state, not a hardcoded string check

### New icon/glyph
- Add SVG to `Desktop/assets/icons/`
- Add a match arm to `icon_path()` in `Desktop/src/gui/theme.rs`
- Use the glyph key in `NavigationPageDefinition.glyph` / `NavigationGroupDefinition.glyph`

### New theme color
- Desktop: add to `Desktop/src/gui/theme.rs` as named constants or helper fns — never inline `rgb(0x...)` in `app.rs`
- iOS: add a computed property to `AppTheme` in `AppTheme.swift` — never inline `Color(hex:)` in view files

## What Not to Do

```
// BAD — hardcoded page ID match in surface render logic
fn is_page_visible(&self, page_id: &str) -> bool {
    match page_id {
        "utility.shell" => self.shell_enabled,
        "network.overview" => self.net_enabled(),
        _ => true,
    }
}

// GOOD — derive from module state via registry
fn is_page_visible(&self, page_id: &str) -> bool {
    let Some(page) = navigation::page_by_id(page_id) else { return false };
    match page.required_module {
        Some(module_name) => self.is_module_enabled(module_name),
        None => true,
    }
}
```

```
// BAD — named per-module bool fields
pub shell_enabled: bool,
fn net_enabled(&self) -> bool { … }
fn remote_session_enabled(&self) -> bool { … }

// GOOD — single generic query
fn is_module_enabled(&self, name: &str) -> bool {
    self.module_rows.iter()
        .find(|(n, _)| n == name)
        .map(|(_, enabled)| *enabled)
        .unwrap_or(false)
}
```

```
// BAD — special-case reload on hardcoded page ID
.onChange(of: activePageID) { pageID in
    if pageID == "global.modules" { reloadModules() }
}

// GOOD — modules page is just a page; reload via onAppear of ModulesView
```

```
// BAD — adding a module means editing surface render code
if self.active_page_id == "utility.shell" { … shell panel … }
else if self.active_page_id == "global.modules" { … modules panel … }
else { … placeholder … }

// GOOD — dispatch via a page panel registry / lookup so new pages
// register themselves, not get added to a growing if-else chain
```

## Key Invariants

- `MODULE_REGISTRY` drives module availability everywhere — don't bypass it
- `NavigationRegistry` (serialized to JSON in `navigation.rs`) is the iOS navigation contract — iOS deserializes it, never hardcodes page/group lists
- `ConfigFile::merge_defaults()` handles migration — when renaming a module, add a migration there, not a one-off rename elsewhere
- All cross-platform logic lives in `arcadia_core` — if you find yourself writing the same logic in both `app.rs` and `ContentView.swift`, it belongs in the core instead

## Build

```sh
# Desktop
cd Desktop && cargo build

# Shared core (tests)
cd Shared && cargo test

# iOS framework (requires macOS + Xcode)
bash Shared/Scripts/build-ios-framework.sh
```
