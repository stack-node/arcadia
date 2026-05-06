# Arcadia — Agent Instructions

Read CLAUDE.md first. This file adds agent-specific rules on top of it.

## Prime Directive: Registry-Driven, Not Hardcoded

Every module, every page, every group has **one registration point**. When asked to add, change, or remove any of these, touch the registry entry — let the rest of the system derive from it. Do not scatter the change across surface files.

## Anti-Patterns — Refuse to Write These

### 1. Named module booleans
```rust
// NEVER add fields like these to ArcadiaRoot or any surface state
pub shell_enabled: bool,
pub lan_enabled: bool,
pub net_enabled: bool,

// NEVER add methods like these
fn shell_enabled(&self) -> bool { … }
fn net_enabled(&self) -> bool { … }
```
One method: `fn is_module_enabled(&self, name: &str) -> bool`. Query it with the `*_MODULE_NAME` constants from `config/modules.rs`.

### 2. Hardcoded page ID match arms in visibility logic
```rust
// NEVER write this pattern
match page_id {
    "utility.shell" => self.shell_enabled,
    "network.overview" => self.net_enabled(),
    _ => true,
}
```
Page visibility must derive from the page's declared module dependency in `NavigationPageDefinition`, not from a surface-level match.

### 3. Growing if-else chains for page content dispatch
```rust
// NEVER grow this pattern
if self.active_page_id == "utility.shell" { … }
else if self.active_page_id == "global.modules" { … }
else if self.active_page_id == "network.overview" { … }  // don't add
```
Each new page added this way makes the problem worse. If this pattern exists and you must add a page, flag it as technical debt and ask before extending it.

### 4. Special-casing page IDs in event handlers
```swift
// NEVER hardcode magic behavior on specific page IDs in generic handlers
.onChange(of: activePageID) { pageID in
    if pageID == "global.modules" { reloadModules() }
}
```
Lifecycle side-effects belong in the view for that page, not in a global observer.

### 5. Inline raw hex colors in render code
```rust
// NEVER inline hex colors directly in app.rs or any view file
.bg(rgb(0x151a22))
.text_color(rgb(0x93c5fd))
```
Colors belong in `Desktop/src/gui/theme.rs` (Rust) or `AppTheme.swift` (iOS) as named values.

### 6. Duplicating core logic in surface code
If the same logic appears in both `Desktop/src/gui/app.rs` and `Mobile/iOS/ArcadiaApp/ContentView.swift`, it belongs in `Shared/ArcadiaCore`. Surface code calls the core; it does not re-implement it.

## Correct Extension Patterns

### Adding a module (all platforms, zero surface edits required)
```rust
// 1. In Shared/ArcadiaCore/src/config/modules.rs — add constant + registry entry
pub const FOO_MODULE_NAME: &str = "foo";

static MODULE_REGISTRY: &[ModuleManifest] = &[
    // … existing entries …
    ModuleManifest {
        name: FOO_MODULE_NAME,
        version: "1.0.0",
        description: "What foo does.",
        required_modules: &[],
    },
];

// 2. Create Shared/ArcadiaCore/src/modules/foo.rs with commands() fn
// 3. Register in Shared/ArcadiaCore/src/modules/mod.rs
// Done — GUI, CLI, iOS module list updates automatically
```

### Adding a navigation page
```rust
// 1. In Shared/ArcadiaCore/src/navigation.rs — add to PAGE_DEFINITIONS
NavigationPageDefinition {
    id: "utilities.foo",
    title: "Foo",
    description: "Foo does things.",
    glyph: "foo",           // must have matching entry in theme.rs icon_path()
    system_image: "star",   // SF Symbol name for iOS
    // When this page needs a module to be enabled, that relationship
    // must be declared here, not scattered in surface match arms
},

// 2. Add "utilities.foo" to the relevant GROUP_DEFINITIONS.pages slice

// 3. Desktop: add a panel render method + route it via the page ID
// 4. iOS: add a view + route it in ContentView page dispatch
```

### Checking if a module is enabled (surface code)
```rust
// Rust (Desktop)
fn is_module_enabled(&self, name: &str) -> bool {
    self.module_rows.iter()
        .find(|(n, _)| n == name)
        .map(|(_, enabled)| *enabled)
        .unwrap_or(false)
}
// Call as: self.is_module_enabled(SHELL_MODULE_NAME)
```
```swift
// Swift (iOS)
func isModuleEnabled(_ name: String) -> Bool {
    modules.first(where: { $0.name == name })?.enabled ?? false
}
```

### Renaming a module
Edit `MODULE_REGISTRY` name and constant. Add a migration in `ModulesConfig::merge_defaults()` (see `LEGACY_LAN_MODULE_NAME` pattern). Do not do ad-hoc renames elsewhere.

## Before Writing Any Code

Ask these questions:
1. **Does a registry entry exist for this?** If not, create one — don't work around it.
2. **Am I adding a name check on a specific module or page ID in surface code?** If yes, stop — that logic belongs in the registry or the core.
3. **Am I adding a new field/property that tracks a specific module's state?** If yes, use the generic `is_module_enabled` pattern instead.
4. **Am I writing the same logic for both Desktop and iOS?** If yes, it belongs in `arcadia_core`.
5. **Am I inlining a color value?** Put it in the theme layer.

## File Ownership

| File | Purpose | Agent rule |
|------|---------|------------|
| `config/modules.rs` | Module registry + config | Extend `MODULE_REGISTRY`; never add per-module booleans |
| `navigation.rs` | Page/group registry | Extend `PAGE_DEFINITIONS` / `GROUP_DEFINITIONS`; never add parallel lists |
| `modules/shell.rs`, `modules/lan/`, etc. | Module command handlers | One file per module; no cross-module logic |
| `gui/app.rs` | Desktop render + event handling | Consumes registries; no hardcoded IDs in logic |
| `gui/theme.rs` | Desktop icon + color helpers | All color/icon lookups go here |
| `AppTheme.swift` | iOS color tokens | All iOS colors go here |
| `ContentView.swift` | iOS coordinator | Thin — consumes registry + module state; no business logic |

## When Asked to "Just Make It Work" With a Hardcoded Value

Do not. If time is the constraint, implement the proper registry-driven pattern and leave a `// TODO` comment explaining what's missing — do not leave hardcoded strings in surface logic. A hardcoded page ID match arm today becomes five hardcoded match arms after the next agent touches the file.
