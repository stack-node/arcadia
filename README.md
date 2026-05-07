# Arcadia

Arcadia is a multi-platform runtime and shell: one Rust core (**`arcadia-core`**) with thin native surfaces — Desktop (**CLI + GPUI**) and iOS (**SwiftUI + UniFFI**). The core owns modules, commands, configuration, navigation metadata, and LAN-linked execution; surfaces render state and dispatch **`execute_command`**.

Design goals:

- **Registry-driven**: modules and navigation pages are registered once in core; CLI, GUI, and mobile derive lists from there.
- **Thin clients**: any surface can treat a LAN peer as the **host of record** for modules + navigation JSON via **`surface.snapshot`**, while **`thin-client.toml`** and **`ARCADIA_NET_AS`** persist/bootstrap the route.

See **`CLAUDE.md`** / **`AGENTS.md`** for agent/contributor rules (no hardcoded page IDs in visibility logic, theme tokens only in theme layers, etc.).

---

## What ships

| Surface | Location | Notes |
|--------|----------|--------|
| **Desktop CLI** | `Desktop/` · default **`headless`** | Interactive REPL (rustyline), **`module`**, config helpers |
| **Desktop GUI** | `Desktop/` · **`--features gui`** | GPUI: sidebar, registry-driven pages, shell/TUI, modules, LAN nodes, **session route** (**`lan:…`**) for thin client |
| **iOS app** | `Mobile/iOS/ArcadiaApp/` | SwiftUI + **`ArcadiaCore.xcframework`**; navigation from core JSON and/or **remote `surface.snapshot`** |
| **Headless “server”** | Same **`Desktop`** binary without **`gui`** | Runs core + LAN service + REPL; GUI peers route commands with **`net_as: lan:…`** |

---

## Repository layout

```
Shared/ArcadiaCore/                 # Crate: arcadia-core (staticlib / cdylib / lib)
  src/
    lib.rs
    ffi.rs                          # UniFFI → Swift (execute_command, navigation JSON, thin_client_*, mirror drain, …)
    navigation.rs                   # PAGE_DEFINITIONS, GROUP_DEFINITIONS; NavigationRegistryOwned JSON for snapshots
    config/
      modules.rs                    # MODULE_REGISTRY, ModulesConfig → ~/Arcadia/Configuration/modules.toml
      thin_client.rs                # ThinClientConfig → thin-client.toml (preferred route + surface_client_id)
      …
    modules/
      shell.rs, net.rs, lan/, …
      surface.rs                    # surface.snapshot / surface.patch (modules + revision + nav mirror)
      remote_session.rs             # Routing gate only (no commands); enables net_as lan:… locally
      remote_mirror.rs              # Host-side NODE_EXEC transcript + UI sync batch for FFI

Desktop/                            # Binary: arcadia
  src/main.rs                       # gui vs headless via features
  cli/
  gui/                              # GPUI app (features gui)

Mobile/iOS/
  ArcadiaApp/                       # SwiftUI (ContentView+, SidebarView, LanNodesView, …)
  ArcadiaCore/                      # Generated Swift + ArcadiaCore.xcframework (rebuild after core/FFI changes)

Shared/Scripts/
  build-ios-framework.sh            # iOS device/sim libs + UniFFI Swift + xcframework
  Launcher.sh / Launcher.ps1
  install-global-commands-macos.sh

Resources/                          # Icons, wallpapers, sounds (see tree in repo)
Configuration/                      # Layout reference (runtime uses ~/Arcadia/Configuration on Desktop)
Launchers/Development/OSX/        # Optional dev launcher; see Launchers/Development/OSX/README.md
.github/workflows/                # CI (desktop + iOS simulator paths)
```

---

## Architecture

### Single sources of truth (core)

| Concern | Location | Consumed by |
|--------|----------|-------------|
| Module list + dependency edges | **`MODULE_REGISTRY`** in **`config/modules.rs`** | CLI, GPUI, iOS module list |
| Static navigation shape | **`PAGE_DEFINITIONS`**, **`GROUP_DEFINITIONS`** in **`navigation.rs`** | Compiled Desktop helpers; JSON export |
| Serializable navigation for snapshots | **`NavigationRegistryOwned`** | Embedded in **`surface.snapshot.extra.navigation_registry`** |
| Module on/off state | **`ModulesConfig`** (**`modules.toml`**) | Every surface when showing **local** host |

### Command execution

- Tokens are **`module.command`** (e.g. **`shell.execute`**, **`lan.scan`**, **`surface.snapshot`**).
- **`execute_command`** in **`modules/mod.rs`** dispatches locally or forwards when **`ExecutionContext.net_as`** is set (e.g. **`lan:192.168.1.10`**). LAN forwarding requires **local** **`remote-session`**, **`lan`**, and **`net`** enabled; the peer runs normal module checks.
- **`remote-session`** has **no command verbs** — it is only the **permission flag** to route over LAN.

### Surface mirror API (thin client / multi-peer)

- **`surface.snapshot`** returns JSON: **`modules`**, monotonic **`revision`** (bumped after successful **`surface.patch`** batches), and **`extra.navigation_registry`** (host nav JSON). Clients use this when **`remote_route`** / **`net_as`** points at a peer.
- **`surface.patch`** accepts a JSON array of tagged operations (today **`modules_set`**). Optional **`client_id`** identifies the GUI peer (persisted per machine in **`thin-client.toml`**).
- **`lan.session_targets`** returns JSON for connected, approved LAN peers (route picker).

**Multi-client caveat:** Host **`modules.toml`** is shared. Concurrent GUIs see the same truth after reload; simultaneous toggles are **last writer wins** — **`revision`** helps detect change but does not merge conflicts.

### Desktop GUI behavior

- **Session chip**: **`lan:<host>`** vs local; persists **`preferred_remote_route`** via **`thin-client.toml`** when you pick a peer (also clears on Local).
- **Startup route:** **`ARCADIA_NET_AS`** overrides saved preference; otherwise **`thin-client.toml`** if **`lan`** + **`remote-session`** are enabled.
- When routed remotely, sidebar/nav can reflect **`surface.snapshot`** **`navigation_registry`** so newer host layouts appear without rebuilding the client binary.

### iOS behavior

- Calls **`set_config_root_path`** early (app sandbox).
- **`navigationRegistry`** loads from **`navigation_registry_json()`** locally; when **`remoteRoute`** is set, **`surface.snapshot`** can replace registry from **`extra.navigation_registry`**.
- **`thinClientPreferredRouteGet` / `thinClientPreferredRouteSet` / `thinClientSurfaceClientId`** mirror Desktop **`thin-client.toml`** + patch attribution.
- **`ARCADIA_NET_AS`** in the scheme/environment behaves like Desktop for bootstrap.

---

## Configuration (user disk)

Default root: **`~/Arcadia/Configuration/`** (see **`Shared/ArcadiaCore/src/config/mod.rs`**).

| File | Purpose |
|------|---------|
| **`modules.toml`** | Per-module enable/disable ( **`ModulesConfig`** ) |
| **`commandline.toml`** | CLI-only preferences |
| **`thin-client.toml`** | **`preferred_remote_route`** (`lan:…`), **`surface_client_id`** (UUID for **`surface.patch`**) |
| LAN pairing | Managed via **`lan.node`** / handlers (see **`modules/lan/`**) |

---

## Prerequisites

| Tool | Used for |
|------|-----------|
| Rust (`rustup`, `cargo`) | Core + Desktop |
| Xcode + CLI tools | iOS app and xcframework |
| **`rustup target`** `aarch64-apple-ios`, `aarch64-apple-ios-sim` | **`build-ios-framework.sh`** |

---

## Build and run

### Desktop GUI

```sh
cd Desktop && cargo build --features gui && cargo run --features gui
```

### Desktop CLI (headless)

Default Cargo features are **`headless`** only:

```sh
cd Desktop && cargo run
```

With **`--features gui`**, the **`gui`** feature takes **`main`** (GPUI window); headless remains declared but unused at runtime.

### Desktop release

```sh
cd Desktop && cargo build --release --features gui
```

### Core tests

```sh
cd Shared && cargo test -p arcadia-core
```

### iOS framework + Swift bindings (**required after `ffi.rs` or exported Rust API changes**)

```sh
bash Shared/Scripts/build-ios-framework.sh
```

Refreshes **`Mobile/iOS/ArcadiaCore/Generated/`** and **`ArcadiaCore.xcframework`** (device + simulator static libs). Open Xcode and build **`ArcadiaApp`** afterward.

### Launcher menus

```sh
bash Shared/Scripts/Launcher.sh
pwsh Shared/Scripts/Launcher.ps1
```

### Global CLI wrappers (macOS)

```sh
bash Shared/Scripts/install-global-commands-macos.sh
```

Ensures **`~/.local/bin`** wrappers (**`arcadia`**, **`arcadia-gui`**, etc.) are on **`PATH`**.

---

## Environment variables

| Variable | Where | Purpose |
|----------|-------|---------|
| **`ARCADIA_NET_AS`** | Desktop GUI, iOS | Bootstrap **`net_as`** route (e.g. **`lan:192.168.1.5`**). Overrides **`thin-client.toml`** preference on startup. |
| **`ARCADIA_IOS_DEVICE_NAME`** | iOS deploy scripts | Pin install target by device name |
| **`ARCADIA_IOS_FORCE_UNINSTALL`** | iOS deploy scripts | Uninstall existing app before install |

---

## Adding features (short pointers)

- **New module:** **`MODULE_REGISTRY`** + **`modules/<name>.rs`** + **`modules/mod.rs`** — see **`CLAUDE.md`**.
- **New navigation page:** **`PAGE_DEFINITIONS`** / **`GROUP_DEFINITIONS`** + surface panel + routing derived from module state — not hardcoded visibility tables.
- **New thin-client payload:** extend **`SurfaceSnapshot.extra`** or **`SurfacePatch`** enum in **`modules/surface.rs`**; keep surfaces consuming **`surface.snapshot`** / **`surface.patch`** rather than one-off remote-session verbs.

---

## CI

**`.github/workflows/`** — builds desktop targets and iOS-related configurations on selected branches (see workflow files for triggers).

---

## Contributing

- Follow **`AGENTS.md`**: registry-driven modules/pages, **`is_module_enabled(name)`**, theme tokens in theme modules only.
- After FFI changes: run **`build-ios-framework.sh`** and commit **`Generated/`** + **`xcframework`** artifacts per team practice.
