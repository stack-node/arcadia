# Arcadia

Arcadia is a multi-platform runtime and shell: one Rust core (`arcadia-core`) with thin native shells on macOS (CLI + GPUI) and iOS (SwiftUI + UniFFI). The core owns modules, commands, configuration, and navigation; surfaces render and dispatch.

## What ships

| Surface | Location | Notes |
|--------|----------|--------|
| **Desktop CLI** | `Desktop/` · feature `headless` | Interactive REPL (rustyline), completion, config/module commands |
| **Desktop GUI** | `Desktop/` · feature `gui` | GPUI window: splash, sidebar, shell/TUI, modules, registry-driven pages |
| **iOS app** | `Mobile/iOS/ArcadiaApp/` | SwiftUI + `ArcadiaCore.xcframework`; navigation from JSON exported by core |
| **Scripts** | `Shared/Scripts/` | Launcher menus, iOS framework build, optional global CLI wrappers |
| **Dev launcher (macOS)** | `Launchers/Development/OSX/` | Optional menu-bar helper; `build-app.sh` builds `.app` (uses production app icon source) |

## Repository layout

```
Shared/ArcadiaCore/              # Rust library (crate: arcadia-core)
  src/
    lib.rs
    ffi.rs                       # UniFFI exports → Swift
    navigation.rs                # PAGE_DEFINITIONS, GROUP_DEFINITIONS, JSON export (incl. optional required_module per page)
    config/                      # modules.toml schema, MODULE_REGISTRY (single module list + deps)
    modules/                     # shell, net, lan/, … — command handlers
  Scripts/
    build-ios-framework.sh       # release libs + Swift bindings + ArcadiaCore.xcframework
    Launcher.sh / Launcher.ps1   # interactive build/deploy menu
    install-global-commands-macos.sh

Desktop/                         # Binary crate: arcadia (CLI + GUI features)
  src/
    main.rs
    cli/                         # REPL, args, completion, config/module CLI
    gui/
      assets.rs                  # Embedded SVGs; app badge PNG from Resources/Icons/Production
      app/                       # GPUI root: entry, lifecycle, navigation helpers, sidebar, shell, splash, root UI
      theme/                     # Colors, icons, nav accent palettes, module chrome
      tui/                       # Embedded terminal UI session helpers

Mobile/iOS/
  ArcadiaApp/                    # SwiftUI sources (ContentView split across ContentView+*.swift)
  ArcadiaApp.xcodeproj/
  ArcadiaCore/                   # Generated Swift bindings + xcframework (rebuild after core FFI changes)

Resources/
  Icons/
    Production/                  # Canonical raster icons (app store 1024, in-app badge master)
    Prototypes/                  # Draft / exploratory artwork (not referenced by build)
  Wallpapers/                    # Optional shared imagery
  Sounds/                        # Notification samples

Configuration/                   # User config layout (see Arcadia config root on disk)
Launchers/Development/OSX/       # macOS dev launcher package + build-app.sh

.github/workflows/               # CI (multi-platform desktop + iOS simulator)
```

## Architecture

### Single sources of truth (core)

- **`MODULE_REGISTRY`** in `Shared/ArcadiaCore/src/config/modules.rs` — module names, versions, and `required_modules` edges. GUI, CLI, and iOS consume this indirectly via config and command routing.
- **`PAGE_DEFINITIONS` / `GROUP_DEFINITIONS`** in `navigation.rs` — navigation metadata. Each page may set **`required_module`** (e.g. shell for `utility.shell`, net for `network.overview`). Serialized to JSON for iOS and used by the desktop shell for visibility.
- **`execute_command`**, LAN helpers, and module commands live under `modules/`; FFI in `ffi.rs` exposes the subset needed by mobile.

### Desktop (`Desktop/`)

- **`headless`** and **`gui`** are Cargo features; default feature set is defined in `Desktop/Cargo.toml`.
- GUI code under `gui/app/` composes the window: splash sequence, top bar, sidebar, active page content, modules UI, shell/TUI.
- **`gui/theme/`** centralizes colors and icon paths; do not scatter raw RGB in view code.
- Embedded UI badge: virtual asset `icons/app-icon.png` is compiled from **`Resources/Icons/Production/Final-1-appicon.png`**.

### iOS (`Mobile/iOS/`)

- On launch, the app sets the config root (app support), then loads **`navigationRegistryJson()`** from core to populate **`NavigationRegistry`** / **`PageDefinition`** (including optional **`requiredModule`**).
- **`ModuleNames.swift`** mirrors core module name strings for clarity.
- Module toggles debounce and use **`probeModuleToggle`** when enabling with dependency checks.

## Prerequisites

| Tool | Used for |
|------|-----------|
| Rust (`rustup`, `cargo`) | All desktop + core builds |
| Xcode + CLI tools | iOS app and xcframework |
| `rg` (ripgrep) | Some launcher deploy flows |

## Build and run

### Desktop (from repo root)

```sh
cd Desktop && cargo build --features gui && cargo run --features gui
```

Headless CLI:

```sh
cd Desktop && cargo build --features headless && cargo run --features headless
```

Release:

```sh
cd Desktop && cargo build --release --features gui
```

### Core tests

```sh
cd Shared && cargo test -p arcadia-core
```

### iOS framework + bindings

After any change to `ArcadiaCore` APIs used from Swift:

```sh
bash Shared/Scripts/build-ios-framework.sh
```

Outputs Swift bindings and **`Mobile/iOS/ArcadiaCore/ArcadiaCore.xcframework`**. Then build in Xcode or use **`Shared/Scripts/Launcher.sh`** for common flows.

### Launcher menus

```sh
bash Shared/Scripts/Launcher.sh       # macOS / Unix-like
pwsh Shared/Scripts/Launcher.ps1      # PowerShell
```

### Global commands (macOS)

```sh
bash Shared/Scripts/install-global-commands-macos.sh
```

Installs wrappers (e.g. `arcadia`, `arcadia-gui`, `arcadia-ios`) into `~/.local/bin`. Ensure that directory is on your **`PATH`**.

### macOS development launcher app

See **`Launchers/Development/OSX/README.md`**. **`build-app.sh`** rasterizes icons from **`Resources/Icons/Production/Final-1-appicon-1024.png`** into the app icon set.

### iOS deploy environment variables

```text
ARCADIA_IOS_DEVICE_NAME="My iPhone"   # pin a device by name
ARCADIA_IOS_FORCE_UNINSTALL=1         # uninstall existing app before install
```

## Command model

Commands are **`module.command`** tokens, e.g. **`shell.execute`**, **`lan.scan`**, **`lan.node …`**. The CLI adds **`module …`** and **`configuration …`** helpers. Remote routing appends flags such as **`--net:as lan:<target>`** and **`--net:timeout <ms>`** where supported.

## Module registration

Add a **`ModuleManifest`** entry to **`MODULE_REGISTRY`**, a name constant, **`modules/<name>.rs`** with **`commands()`**, and register the module in **`modules/mod.rs`**. Dependency edges are **only** declared on the manifest rows in **`config/modules.rs`**.

## Configuration

- **Desktop**: config under the Arcadia config directory (e.g. under the user home layout described in core).
- **iOS**: app sets config root at startup.

Key files: **`modules.toml`**, **`commandline.toml`** (CLI appearance and behavior).

## CI

**`.github/workflows/stable-build-matrix.yml`** builds desktop targets and the iOS simulator configuration on pushes to **`stable`**.

## Contributing

- **`CLAUDE.md`** and **`AGENTS.md`** describe registry-driven conventions (navigation, modules, theme colors). Prefer extending registries over hardcoding page or module IDs in surface code.
- Regenerate the iOS framework when changing **`ffi.rs`** or exported core behavior relied on by Swift.
