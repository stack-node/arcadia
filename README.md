# Arcadia

Arcadia is a multi-platform runtime and shell that shares one Rust core across desktop and iOS. It ships as:

- a desktop app — headless CLI or GPU-accelerated GUI via GPUI,
- an iOS app — SwiftUI shell backed by UniFFI bindings to the Rust core,
- shared scripts for building, launching, and deploying consistently across platforms.

## Repository Layout

```
Desktop/                        Rust desktop crate (arcadia)
  src/
    main.rs                     entry point (~30 lines)
    cli/                        interactive CLI shell
      mod.rs                    REPL loop, command dispatch, settings
      args.rs                   command specs, flag parsing, execution context
      completion.rs             rustyline completer + hinter + highlighter
      config_cmds.rs            configuration get/set/reset handlers
      module_cmds.rs            module enable/disable handlers
    gui/                        GPUI desktop shell
      app.rs                    ArcadiaRoot view, shell panel, module panel
      theme.rs                  glyph rendering helpers

Shared/
  ArcadiaCore/                  Rust core library (arcadia-core)
    src/
      lib.rs                    crate root
      ffi.rs                    UniFFI exports (iOS + future bindings)
      navigation.rs             shared navigation registry, JSON serialization
      config/
        mod.rs                  ConfigFile trait, path resolution
        modules.rs              module states, dependency graph (MODULE_REGISTRY)
      modules/
        mod.rs                  command routing, ExecutionContext, load/shutdown
        lan/                    LAN module (peer discovery + remote execution)
          mod.rs                public API, ModuleCommand entries
          protocol.rs           constants, buffer sizes, PeerStatus, PeerRecord
          config.rs             LanNodeConfig, node approval helpers
          peers.rs              NodeState, peer state machine
          discovery.rs          UDP multicast, service lifecycle
          handlers.rs           node pair/connect/accept/reject/alias/save/auto
        net.rs                  net module stub
        shell.rs                shell.execute command
      platform/                 platform detection
  Scripts/
    Launcher.sh                 interactive launcher menu (bash)
    Launcher.ps1                interactive launcher menu (PowerShell)
    build-ios-framework.sh      builds xcframework + Swift bindings
    install-global-commands-macos.sh  installs arcadia / arcadia-gui / arcadia-ios

Mobile/
  iOS/
    ArcadiaApp/                 SwiftUI iOS shell
      ArcadiaApp.swift          @main entry, config root bootstrap
      ContentView.swift         top-level coordinator view (~370 lines)
      NavigationModels.swift    PageDefinition, GroupDefinition, NavigationRegistry
      AppTheme.swift            AppTheme struct — all color computed properties
      GlassComponents.swift     GlassCard<Content>, GlassMetric view structs
      SidebarView.swift         sidebar panel, group/page selection, gestures
      ShellView.swift           terminal UI — history, input, run button
      ModulesView.swift         module toggle rows
    ArcadiaCore/                generated Swift bindings + xcframework (gitignored)

Resources/                      shared non-code assets
```

## Architecture

### Shared core (`arcadia-core`)

Single source of truth for:

- **Module registry** — `MODULE_REGISTRY` const table defines all modules and their dependency graph. Adding a module is one entry.
- **Command routing** — `execute_command(token, args, context)` dispatches `module.command` tokens to the right handler, with optional `--net:as lan:<target>` remote routing.
- **LAN networking** — UDP multicast peer discovery, node pairing and approval flows, remote command execution with timeout handling.
- **Navigation registry** — shared page/group definitions serialized as JSON and consumed by all shells.
- **UniFFI FFI layer** — type-safe Rust → Swift bridge; iOS calls the same logic desktop uses.

### Desktop shell (`Desktop/`)

Two build features, one binary:

| Feature | Mode |
|---------|------|
| `headless` | Interactive rustyline CLI with tab completion, history, config commands |
| `gui` | GPUI window with sidebar navigation, module panel, embedded terminal |

Both share the same `cli/` module; GUI spawns a background CLI thread alongside the GPUI window.

### iOS shell (`Mobile/iOS/`)

SwiftUI shell driven entirely through UniFFI bindings:

- On launch, sets the config root to the iOS app support container.
- Decodes navigation registry JSON from core to build sidebar/page structure.
- Module toggles use a 300ms debounce and preflight `probeModuleToggle` before writing state.
- Shell panel routes `shell.execute` through the FFI and streams output into history.

## Prerequisites

| Tool | Required for |
|------|-------------|
| `rustup` / `cargo` | All Rust builds |
| Xcode + command line tools | iOS framework build, device deploy |
| `xcrun devicectl` | Physical device deploy |
| `rg` (ripgrep) | Launcher iOS deploy flow |

## Build and Run

### Desktop — direct cargo

Run from the repository root:

```bash
# Headless
cargo run --manifest-path Desktop/Cargo.toml --target-dir target --features headless
cargo run --manifest-path Desktop/Cargo.toml --target-dir target --release --features headless

# GUI
cargo run --manifest-path Desktop/Cargo.toml --target-dir target --features gui
cargo run --manifest-path Desktop/Cargo.toml --target-dir target --release --features gui
```

### Desktop + iOS — launcher menu

```bash
bash Shared/Scripts/Launcher.sh       # macOS / Linux
pwsh Shared/Scripts/Launcher.ps1      # PowerShell
```

Options: GUI/headless debug+release, iOS device deploy (Release or Debug).

### Install global commands (macOS)

```bash
bash Shared/Scripts/install-global-commands-macos.sh
```

Installs `arcadia`, `arcadia-gui`, `arcadia-ios` into `~/.local/bin`.

If that directory is not on `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"   # add to ~/.zshrc
```

## iOS Framework / Bindings Pipeline

Rebuild the shared iOS framework after any core API change:

```bash
bash Shared/Scripts/build-ios-framework.sh
```

This:
1. Compiles `arcadia-core` for `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
2. Generates Swift bindings via `uniffi-bindgen`.
3. Assembles `ArcadiaCore.xcframework` under `Mobile/iOS/ArcadiaCore/`.

Then build and deploy with Xcode or use the launcher/global command flow.

### Device selection

```bash
ARCADIA_IOS_DEVICE_NAME="My iPhone"  # prefer a specific device by name
ARCADIA_IOS_FORCE_UNINSTALL=1        # uninstall existing app before install
```

Both env vars work across `Launcher.sh`, `Launcher.ps1`, and the `arcadia-ios` global command.

## Core Command Model

Commands are namespaced `module.command` tokens:

```
shell.execute <command...>
lan.scan [--range <CIDR-or-ip>]
lan.node pair|connect|accept|reject|alias|save|auto|status ...
```

Desktop CLI also accepts:

```
module <name> enable|disable
module <name> enable -requirements
configuration show|get|set|reset ...
```

Global routing flags (append to any command):

```
--net:as lan:<host/ip/alias>     route command to remote node via LAN
--net:timeout <milliseconds>     override remote execution timeout
```

## Module System

Modules live in `arcadia-core/src/modules/`. Each module exposes a `NAME` constant and a `commands()` slice of `ModuleCommand` entries.

Module dependencies are declared in a single const table in `config/modules.rs`:

```rust
static MODULE_REGISTRY: &[(&str, &[&str])] = &[
    ("lan",   &["net"]),   // lan requires net
    ("net",   &[]),
    ("shell", &[]),
];
```

Adding a new module: one entry here, one `mod` in `modules/mod.rs`, implement `commands()`.

## Configuration

Config files are stored under the Arcadia configuration root:

- **Desktop** — user home scoped (`~/Arcadia/Configuration/` or platform equivalent).
- **iOS** — app support container, set at launch via `setConfigRootPath`.

Configurable areas:

| Config | Keys |
|--------|------|
| `commandline` | `input_symbol`, `output_symbol`, `input_color`, `output_color`, `clear_on_start` |
| `modules` | one key per module, `true`/`false` |

Desktop CLI access:

```
configuration show commandline
configuration get commandline.input_color
configuration set commandline.output_color cyan
configuration reset commandline
```

## CI

`.github/workflows/stable-build-matrix.yml` builds all six desktop targets (Linux/macOS/Windows × GUI/headless) plus the iOS simulator app on every PR to `stable`. Artifacts are published as a GitHub release on merge.

Rust dependency caching via `swatinem/rust-cache@v2` is applied to all desktop matrix jobs.

## Development Notes

- Core, navigation, and module interfaces live in `arcadia-core` — keep desktop and iOS aligned through that single source.
- `MODULE_REGISTRY` in `config/modules.rs` is the only place to register a new module's dependency rules.
- Generated iOS artifacts (`ArcadiaCore/`, `build/`) are not committed; run `build-ios-framework.sh` to regenerate.
- Prefer `Shared/Scripts/` entrypoints for reproducible local build and deploy flows.
