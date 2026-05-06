# Arcadia

Arcadia is a multi-platform runtime and shell that shares one Rust core across desktop and iOS.
It currently ships as:

- a desktop app (`headless` CLI or `gui` via GPUI),
- an iOS app (SwiftUI shell backed by UniFFI bindings),
- shared scripts to build, launch, and deploy consistently.

## Repository Layout

- `Desktop/` Rust desktop application crate (`arcadia`)
- `Shared/ArcadiaCore/` shared Rust core crate (`arcadia-core`)
- `Shared/Tools/uniffi-bindgen/` local UniFFI Swift binding generator wrapper
- `Shared/Scripts/` launcher + build + install helper scripts
- `Mobile/iOS/ArcadiaApp/` SwiftUI iOS shell application
- `Mobile/iOS/ArcadiaCore/` generated Swift bindings + `ArcadiaCore.xcframework`
- `Resources/` shared non-code assets
- `build/` local build/deploy outputs

## Architecture

### 1) Shared core (`arcadia-core`)

`arcadia-core` is the source of truth for:

- module configuration and dependency rules,
- command routing and execution context,
- LAN discovery + node pairing + remote command execution,
- FFI APIs consumed by iOS and desktop surfaces,
- shared navigation registry JSON used by multiple shells.

The core exports UniFFI records/functions for:

- listing modules and commands,
- toggling modules (with and without dependency auto-enable),
- probing module toggle requirements before applying,
- executing commands (local or routed via `--net:as lan:<target>`),
- starting/stopping LAN service,
- loading shared navigation definitions.

### 2) Desktop shell (`Desktop/`)

The desktop crate supports two features:

- `headless` (default) - interactive CLI shell
- `gui` - GPUI-based shell with sidebar navigation and module controls

Both shells call into `arcadia-core`, so command semantics are shared.

### 3) iOS shell (`Mobile/iOS/`)

The iOS app initializes a dedicated app-support configuration root and passes it into core FFI on startup.
SwiftUI uses shared navigation registry payloads from core and drives module/command interactions through generated UniFFI bindings.

## Prerequisites

- Rust toolchain (`rustup`, `cargo`)
- For iOS work:
  - Xcode + command line tools
  - `xcodebuild`
  - `xcrun devicectl` (device deploy flow)
- Optional but recommended:
  - `rg` (ripgrep), used by launcher/install helpers

## Build And Run

### Desktop (direct cargo)

From repository root:

- Headless debug:
  - `cargo run --manifest-path Desktop/Cargo.toml --target-dir target --features headless`
- Headless release:
  - `cargo run --manifest-path Desktop/Cargo.toml --target-dir target --release --features headless`
- GUI debug:
  - `cargo run --manifest-path Desktop/Cargo.toml --target-dir target --features gui`
- GUI release:
  - `cargo run --manifest-path Desktop/Cargo.toml --target-dir target --release --features gui`

### Desktop + iOS launcher menu

- `bash Shared/Scripts/Launcher.sh` (macOS/Linux shell)
- `pwsh Shared/Scripts/Launcher.ps1` (PowerShell)

Launcher options include desktop run modes and physical iOS device deploy flows.

### Install Global Commands (macOS)

Install shortcuts into `~/.local/bin`:

- `bash Shared/Scripts/install-global-commands-macos.sh`

Commands installed:

- `arcadia` (headless runner)
- `arcadia-gui` (GUI runner)
- `arcadia-ios` (build + deploy iOS app to connected physical device)

If `~/.local/bin` is not on `PATH`, add:

- `export PATH="$HOME/.local/bin:$PATH"`

## iOS Framework / Bindings Pipeline

To rebuild shared iOS artifacts:

- `bash Shared/Scripts/build-ios-framework.sh`

This pipeline:

1. builds `arcadia-core` for device and simulator targets,
2. generates Swift bindings via UniFFI,
3. assembles `ArcadiaCore.xcframework` under `Mobile/iOS/ArcadiaCore/`.

Then build/deploy app with Xcode or launcher/global command flow.

### Device Selection

Device deploy scripts can auto-pick first connected physical iOS device or prefer a named one:

- `ARCADIA_IOS_DEVICE_NAME="<Your Device Name>"`

Optional uninstall before install:

- `ARCADIA_IOS_FORCE_UNINSTALL=1`

## Core Command Model

Arcadia commands are namespaced by module and executed as tokens like:

- `shell.execute <command...>`
- `lan.scan [--range <CIDR-or-ip>]`
- `lan.node pair|connect|accept|reject|alias|save|auto|status ...`

Desktop CLI also supports:

- `module <name> enable|disable`
- `module <name> enable -requirements`
- `configuration show|get|set|reset ...`
- global routing flags:
  - `--net:as lan:<host/ip/alias>`
  - `--net:timeout <milliseconds>`

## Configuration

Config files are persisted under the Arcadia configuration root:

- desktop defaults to user-home scoped config storage,
- iOS sets config root to app support container at launch.

Current configurable areas include commandline shell preferences and module states.

## Recent Work Summary

This branch includes a major cross-surface update:

- Reworked desktop GPUI app into a real shell surface with:
  - sidebar groups/pages sourced from shared navigation definitions,
  - module management UI,
  - in-app shell panel with command history and caret behavior.
- Expanded iOS SwiftUI shell with:
  - shared navigation registry decoding,
  - richer sidebar/page handling,
  - module toggling and shell interaction improvements,
  - app-support config root bootstrap in `ArcadiaApp`.
- Added shared navigation registry in core (`navigation` module) and exported JSON over UniFFI.
- Upgraded module dependency logic:
  - requirement graph support (`lan` requires `net`),
  - preflight probing for missing requirements,
  - requirement-aware enable flows exposed to FFI.
- Extended LAN module capabilities:
  - peer discovery, pairing and approval flows,
  - aliases/rules/auto-approval controls,
  - remote command execution with validation and timeout handling.
- Hardened shell module command execution output handling (including ANSI cleanup).
- Updated launcher scripts to use workspace manifest paths and consistent `target` directory.
- Added iOS physical-device deploy options to launcher scripts and global `arcadia-ios` helper.
- Refreshed iOS app icon assets and Xcode project integration.
- Adjusted stable build matrix trigger behavior to focus on PR validation.

## Development Notes

- The repository may contain generated outputs (`build/`, framework artifacts) during active development.
- Prefer script entrypoints in `Shared/Scripts/` for reproducible local flows.
- Keep core APIs and navigation definitions in `arcadia-core` so desktop/iOS stay aligned.