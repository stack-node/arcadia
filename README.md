# Arcadia

**One Rust core. One Python SDK. An infinite extension surface. Zero rent.**

Arcadia is a multi-platform runtime, shell, and — ultimately — an **open platform for building system-integrated applications**. Today: a single `arcadia-core` crate owns every module, command, navigation structure, LAN protocol, and config schema, consumed by two native surfaces (GPUI desktop, SwiftUI iOS) plus a CLI. Tomorrow: a Python library that gives any developer full OS reach and a first-class extension SDK for building apps — menu bar tools, custom IDEs, file explorers, widgets, shells — that run everywhere and belong to no vendor.

Built on the same DNA as **[Holos](https://github.com/stack-node/holos)** — *utility over monetization, ownership over subscriptions* — but with a harder engineering mandate: **no duplicated truth between platforms, no hardcoded IDs in surface code, no growing if-else chains that break the next time a module is added.**

---

## Table of contents

- [Why Arcadia exists](#why-arcadia-exists)
- [What Arcadia is](#what-arcadia-is)
- [The vision — where this is going](#the-vision--where-this-is-going)
- [What you can do with it now](#what-you-can-do-with-it-now)
- [Development status](#development-status)
- [Philosophy](#philosophy)
- [Architecture](#architecture)
  - [Command model](#command-model)
  - [Module system](#module-system)
  - [Navigation system](#navigation-system)
  - [Thin-client and LAN routing](#thin-client-and-lan-routing)
  - [Remote mirror](#remote-mirror)
  - [Theme system](#theme-system)
  - [FFI bridge](#ffi-bridge)
- [Module reference](#module-reference)
- [Navigation reference](#navigation-reference)
- [Repository layout](#repository-layout)
- [Configuration](#configuration)
- [Prerequisites](#prerequisites)
- [Build and run](#build-and-run)
- [Environment variables](#environment-variables)
- [Adding features](#adding-features)
- [Testing](#testing)
- [Known gaps and production roadmap](#known-gaps-and-production-roadmap)
- [Security posture](#security-posture)
- [CI](#ci)
- [Contributing](#contributing)
- [Lineage](#lineage)
- [About the creator](#about-the-creator)
- [Donations](#donations)
- [Final note](#final-note)

---

## Why Arcadia exists

Small-tool ecosystems trend the same way: **paywalls, subscriptions, feature flags, AI-generated-app-of-the-week churn.** Good ideas get trapped in silos—one app for the menu bar, one for the terminal, one for "sync," each with its own incompatible settings schema and no way out.

**Holos** pushed back on that for macOS: modular, free, yours to extend.

**Arcadia** pushes harder:

- **One core** (`arcadia-core`) owns modules, commands, config, navigation metadata, and LAN plumbing. Surfaces are **render + dispatch**, not second implementations.
- **Multiple surfaces** from the same logic: terminal REPL, GPUI desktop, SwiftUI pocket — without forking behavior per platform.
- **Optional headless-host + GUI-client** patterns over LAN so your MacBook can drive your phone — or vice versa — without inventing a new protocol per feature.
- **Free. Always.** No paywalls in the architecture. The repo is the product.

If something's missing, you add a module or extend `surface.snapshot` / `surface.patch`. You don't buy another app.

There's a second reason, less technical but just as important.

A lot of people grew up in software environments where the line between user, developer, toolmaker, and creator basically dissolved — game modding scenes, jailbreak ecosystems, Emacs, early web chaos. Environments where you could bend the software, remix the system, blur the boundary between using a tool and building inside it. Those environments permanently change how you think about computing. You stop seeing apps as products and start seeing them as constrained runtimes.

Most modern software feels closed afterward.

Arcadia is an attempt to restore that feeling — but for the desktop itself, with an actual engineering foundation underneath it instead of accumulated chaos.

---

## What Arcadia is

- **A runtime and shell** — execute commands locally or route them across your LAN with the same `execute_command` API.
- **A module registry** — enable/disable capabilities (`shell`, `lan`, `net`, `surface`, `remote-session`, `shell-motd`) from any surface; the registry enforces dependencies.
- **A navigation system** — page and group definitions live in `navigation.rs`, serialized to JSON for iOS, consumed by Desktop directly. No surface hardcodes page IDs.
- **A thin-client protocol** — `surface.snapshot` mirrors host state (modules, nav registry, revision) to clients; `surface.patch` lets clients push changes back.
- **A cross-platform core** — the same Rust crate (`arcadia-core`) builds as a staticlib for iOS (via UniFFI), a native library for Desktop GPUI, and a CLI binary.

---

## The vision — where this is going

What Arcadia is *right now* is the foundation. What it's *becoming* is something more deliberate:

**A programmable personal computing substrate. A unified interaction layer above the OS where users operate *inside* the architecture — not merely use software built on top of it.**

Not an app. Not a framework. Not a toolkit.

A runtime that you inhabit and reshape.

### The real category

Arcadia sits between several things that exist and combines them in a way nothing currently does:

- An **application framework** — native surfaces, layout system, module lifecycle
- A **shell** — command routing, LAN awareness, headless-host patterns
- A **local-first app platform** — config ownership, no vendor, no cloud dependency
- A **programmable UI fabric** — extensions that render into native surfaces as first-class pages

The closest historical analogies are not other desktop frameworks. They're environments like **HyperCard**, **Smalltalk**, **Emacs**, **Garry's Mod**, **Hammerspoon**, **Quartz Composer**, **BetterTouchTool**, **KDE Plasma scripting**, and old-school **jailbreak ecosystems** — environments where the line between user, developer, toolmaker, and creator dissolved.

What those environments had in common: **the system itself was meant to be inhabited and reshaped.** Users started by doing the basic thing and ended up building systems inside systems — admin frameworks, UI toolkits, protocol adapters, entire economies — because the substrate let them.

That's the energy Arcadia is trying to restore. Not aesthetically. In *agency*.

### The tension — and why it's already solved

Arcadia's core architecture is deliberately rigid:

- centralized registries
- canonical state
- deterministic structure
- controlled capability routing
- explicit schemas

That produces coherence. But the concern with that kind of discipline is that it creates activation energy against new ideas. Every new capability has to justify itself in terms of registries, schemas, surface compatibility, state ownership. That cognitive overhead can quietly kill creativity.

The Python extension layer solves this. It separates two things that should always be separate:

| Layer | Character | Enforces |
|-------|-----------|---------|
| **Core** (`arcadia-core`, Rust) | Disciplined | Identity, state consistency, capability routing, surface sync, lifecycle, security, cross-platform |
| **Extension layer** (Python SDK) | Deliberately messy | Nothing. Be weird. Move fast. Break your own conventions. |

The core stays disciplined. The edges stay chaotic.

That balance is not a compromise — it's the architecture that successful long-lived systems always converge toward. Unix kernel / shell chaos. Browser engine / arbitrary JS. Git object model / messy workflows. Game engines / mod scripting. Emacs runtime / user mutation. Garry's Mod engine / Lua ecosystem.

The projects that become culturally important manage both simultaneously. Freedom without structure collapses. Structure without freedom stagnates. Arcadia is attempting to do both at once, at different layers, on purpose.

### The Python library

The next major layer is a Python SDK that exposes the full power of `arcadia-core` to any developer who can write a script. Not a watered-down scripting layer — full OS reach:

- **File system** — read, write, watch, index
- **Processes** — spawn, manage, pipe, monitor
- **Networking** — LAN discovery, routing, peer communication
- **Display** — render into Arcadia's native surfaces (Desktop GUI, iOS) from Python
- **Shell** — execute commands, capture output, stream PTY sessions
- **Config** — read and write module state, preferences, thin-client config
- **Events** — hook into system events, timers, window focus, LAN peer state changes

The goal is parity with what you'd get writing native Rust or Swift — but with a workflow where you open a file, write twenty lines, and have a running extension.

The Rust core stays Rust. Performance-critical paths, protocol handling, LAN networking, config I/O, FFI to native surfaces — none of that moves to Python. Python sits above it, calling into `arcadia-core` through a clean API boundary.

### Extensions: the real product

The Python SDK powers an **extension system**. Extensions are the unit of user-created capability in Arcadia. An extension can be:

| Type | Examples |
|------|---------|
| **Internal app** | A custom shell, a task manager, a log viewer — rendered inside Arcadia's native UI just like the built-in Shell page |
| **Widget** | A persistent overlay — system stats, a clock, a scratchpad, a LAN activity feed |
| **Tool** | A headless background process — file watcher, sync agent, notification hook, cron-style automator |
| **Surface extension** | A sidebar panel, a top-bar chip, a custom modal — extending the host UI without forking it |
| **Device bridge** | Cross-machine extensions that route commands to LAN peers via the existing `remote-session` + `surface.*` protocol |

Extensions register into the same `MODULE_REGISTRY` and `PAGE_DEFINITIONS` systems that built-in modules use. **There is no separate "plugin API."** Extensions are first-class modules. A menu bar tool is a module. A custom IDE panel is a navigation page. A background sync agent is a headless module with no UI.

This matters. When extensions are first-class, they inherit interoperability automatically. Navigation consistency emerges naturally. State becomes composable. Extensions can cooperate without bespoke glue. The registry system — which looks like a constraint from the outside — becomes an advantage the moment you have more than one extension running.

If the extension system ever starts to feel like "plugins bolted onto a real app," something has gone wrong. The shell, the widgets, the internal tools, the user-created apps — they are all equally real inside the same runtime.

### What this makes possible

**For individuals:** build the exact tool you want. Bartender-style menu bar manager? Thirty lines of Python registering a widget module and a tray handler. A file explorer that opens on a keyboard shortcut and talks to your NAS over LAN? Two extensions and a LAN peer config. A custom IDE with your own keybindings, your own terminal, your own sidebar? A surface extension composing built-in shell + your panels.

**For teams:** share extension bundles instead of paying for another SaaS tool. A shared monitoring dashboard, a deployment helper, a standup widget — all running locally, all owned by you, all talking to each other over the same LAN protocol Arcadia already ships.

**For the open-source community:** an ecosystem of extensions that anyone can fork, modify, and publish. No app store approval. No revenue split. No "premium tier." You write it, you run it, you share it if you want.

### Why Python for the SDK

1. **Reach** — more people can write Python than can write Rust or Swift. Lowering the barrier to extension authorship is the whole point.
2. **Iteration speed** — a Python extension reloads without a rebuild. The feedback loop for building a new tool should be seconds, not minutes.
3. **Ecosystem** — PyPI is enormous. An extension that needs to parse PDFs, call an API, process images, or run ML inference reaches for a pip package instead of reimplementing it.
4. **Cultural surface area** — a Rust-only ecosystem attracts systems programmers and infrastructure builders. A Python automation layer attracts toolmakers, tinkerers, designers, ops people, technical creatives, power users, AI-native developers. That's a much larger and more interesting group of people to build with.

### The development workflow target

```
1. arcadia ext new my-tool          # scaffold a new extension
2. edit my_tool/main.py             # write your logic
3. arcadia ext dev my-tool          # hot-reload development mode
4. arcadia ext install my-tool      # register with the local runtime
5. share my_tool/ with anyone       # they install it the same way
```

No Xcode. No Cargo. No native toolchain required to write an extension. The native layer is already compiled and shipped — extension authors build *on top of it*, not inside it.

### Cross-platform by design

Extensions written against the Python SDK run on every surface Arcadia targets:

- **macOS** — GPUI desktop, menu bar, CLI
- **iOS** — SwiftUI surface (where the extension's UI contract is met)
- **Linux** — headless or desktop
- **Windows** — headless or desktop

An extension that declares it renders a navigation page gets that page on every surface that supports pages. An extension that declares it's headless-only runs as a background service everywhere. Surface capabilities are declared, not assumed.

### The priority order

1. **Now:** bulletproof the core — registry patterns, test coverage, CI, revision semantics *(done / in progress)*
2. **Next:** Python bridge — `arcadia-core` callable from Python, initial OS API surface (file, process, shell, config)
3. **Then:** extension loader — Python extensions register as modules at runtime; dev-mode hot reload
4. **Then:** widget and surface extension contracts — render Python-driven UI into native surfaces
5. **Then:** extension registry — discover, install, and share extensions; no central gatekeeper

Each stage ships usable capability. Nothing waits for the whole roadmap to be done.

---

## What you can do with it now

| Capability | How |
|------------|-----|
| Native shell / PTY terminal | `shell.execute` (routable), `shell.internal` (REPL), full PTY/TUI on Desktop |
| Shell welcome banner | `shell-motd` module — fastfetch-style on shell open |
| Manage modules | CLI (`module enable/disable`) or GUI toggle; same `modules.toml` |
| Discover LAN peers | `lan.scan`, `lan.node`, LAN nodes UI on Desktop and iOS |
| Route commands to another machine | `ExecutionContext.net_as = "lan:IP"`, session chip on Desktop, route picker on iOS |
| Mirror host UI state to clients | `surface.snapshot` — modules + nav registry + revision |
| Push module changes from client to host | `surface.patch` with `modules_set` op |
| Run headless as a host | `cargo run` (default `headless` feature) |
| Rebuild iOS after FFI changes | `bash Shared/Scripts/build-ios-framework.sh` |
| Install global CLI wrappers | `bash Shared/Scripts/install-global-commands-macos.sh` |

---

## Development status

Moves fast. Breaks occasionally. That's intentional.

- Features land continuously on `development`.
- APIs (especially FFI and `surface.*`) may evolve — see [Known gaps and production roadmap](#known-gaps-and-production-roadmap) for deliberate limitations.
- Building from source is the surest way to stay current.
- Stable tagged builds will appear as the project matures; CI exercises desktop + iOS simulator paths.

Known gaps are tracked in-repo instead of pretending shipping equals finished.

---

## Philosophy

**Fat core, thin shells.**

`Shared/ArcadiaCore` owns everything. Desktop and iOS read registries, render what those registries say, and `execute_command` back into core. They do not re-implement module graphs or navigation trees.

**Single sources of truth — enforced, not hoped for.**

| Domain | Authority | Never duplicated in |
|--------|-----------|---------------------|
| Module manifests + deps | `MODULE_REGISTRY` · `config/modules.rs` | surface state booleans |
| Navigation pages + groups | `PAGE_DEFINITIONS` / `GROUP_DEFINITIONS` · `navigation.rs` | surface match arms |
| Serializable nav for snapshots | `NavigationRegistryOwned` · embedded in `surface.snapshot` | hardcoded Swift arrays |
| Desktop theme tokens | `gui/theme/` | inline `rgb(0x...)` in views |
| iOS theme tokens | `AppTheme.swift` | inline `Color(hex:)` in views |
| Config schema | `ModulesConfig` · `config/modules.rs` | per-platform config parsers |

**Extend the registry, not scatter `if pageId == …`.**
See `AGENTS.md` for the full list of anti-patterns we refuse to write.

**Discipline at the core. Chaos at the edges. On purpose.**

The architectural discipline of `arcadia-core` — registries, schemas, canonical state, no hardcoded IDs — exists to make the extension layer *safe to be chaotic*. Strict boundaries in the core mean extensions don't need to be strict. An extension can be messy, experimental, surface-specific, fast-moving, structurally impure, and weird. It won't corrupt the runtime underneath it.

Most software chooses: freedom without structure, or structure without freedom. Arcadia is attempting both at different layers simultaneously. The core enforces coherence. The extension layer is where experimentation, exceptions, and "this only exists here" decisions belong.

**Personal tool energy, public repo.**
If Arcadia helps others, great — that's bonus. The goal is a system you own, can fork, and can route across machines you trust.

---

## Architecture

### Command model

All execution flows through a single entry point:

```
execute_command(token: &str, args: &str, context: ExecutionContext) -> String
```

- **Tokens** follow `module.command` format: `shell.execute`, `lan.scan`, `surface.snapshot`, `surface.patch`, etc.
- **`ExecutionContext`** carries `net_as` (optional LAN routing, e.g. `lan:192.168.1.10`) and `net_timeout_ms`.
- When `net_as` is set, `execute_command` forwards the token + args over UDP to the target peer instead of dispatching locally. The peer runs the command under its own module rules.
- LAN forwarding requires local `remote-session`, `lan`, and `net` modules enabled; the peer enforces its own module requirements for the token.
- FFI exposes this identically to iOS and Desktop — same logical API, same routing semantics.

### Module system

Modules are entries in `MODULE_REGISTRY` (`config/modules.rs`). Each entry is a `ModuleManifest`:

```rust
pub struct ModuleManifest {
    pub name: &'static str,          // unique key, e.g. "shell"
    pub version: &'static str,
    pub description: &'static str,
    pub required_modules: &'static [&'static str], // dependency enforcement
}
```

`ModulesConfig` (TOML-backed) maps module names to enabled state. Key behaviors:

- `enable_with_requirements(name)` — transitively enables all deps before the target.
- `missing_requirements_for(name)` — returns unmet deps (used for UI requirement prompts).
- `merge_defaults()` — config migration entry point; handles legacy renames (e.g. `LEGACY_LAN_MODULE_NAME`).
- Changes write to `~/Arcadia/Configuration/modules.toml` (Desktop) or the app container path (iOS).

Every surface calls `list_modules()` → `Vec<ModuleStatus>` and renders whatever comes back. No surface hardcodes module names in layout logic.

### Navigation system

Navigation structure lives entirely in `navigation.rs` as two static slices:

**`PAGE_DEFINITIONS`** — 7 pages:

| ID | Title | Required Module |
|----|-------|-----------------|
| `utility.shell` | Shell | `shell` |
| `global.dashboard` | Dashboard | — |
| `global.logs` | Logs | — |
| `global.settings` | Settings | — |
| `global.modules` | Modules | — |
| `network.overview` | Network | `net` |
| `network.nodes` | Nodes | `lan` |

**`GROUP_DEFINITIONS`** — 2 groups:

| ID | Label | Pages |
|----|-------|-------|
| `utilities` | Utilities | `utility.shell` |
| `network` | Network | `network.overview`, `network.nodes` |

`NavigationPageDefinition.required_module` drives visibility — surfaces query `is_module_enabled(page.required_module)`, never hardcode per-page logic. The full registry serializes to JSON via `default_navigation_registry_json()` for:

- iOS FFI: `navigation_registry_json()` → deserializes into `NavigationRegistry` Swift struct
- Thin-client: embedded in `surface.snapshot` extra field so remote clients get host's nav without a local copy

Lookup helpers: `page_by_id(id)`, `group_by_id(id)`.

### Thin-client and LAN routing

Arcadia supports a **headless host + GUI client** pattern over LAN:

```
[iOS or Desktop GUI]  ──── surface.snapshot ───►  [headless arcadia host]
                      ◄─── surface.patch    ─────
                      ──── execute_command("lan:IP") ──► (routed command)
```

**`surface.snapshot`** — host serializes current state:
```json
{
  "modules": [{"name": "shell", "enabled": true}, ...],
  "revision": 7,
  "extra": {
    "navigation_registry": "{ ...full nav JSON... }"
  }
}
```

**`surface.patch`** — client pushes changes back:
```json
{
  "client_id": "uuid-from-thin-client.toml",
  "ops": [{"type": "modules_set", "name": "lan", "enabled": true}]
}
```

**`lan.session_targets`** — returns JSON list of approved peers for the session picker UI.

**`thin-client.toml`** persists:
- `preferred_remote_route` — remembered LAN target (e.g. `lan:192.168.1.5`)
- `surface_client_id` — UUID for patch attribution

**`ARCADIA_NET_AS`** env var bootstraps `net_as` on startup, overriding `thin-client.toml`.

**Multi-client caveat:** `modules.toml` is a single file on the host. Concurrent edits are last-writer-wins with no merge semantics. See [Known gaps](#known-gaps-and-production-roadmap).

### Remote mirror

When this machine executes an inbound `NODE_EXEC` for a remote peer, `modules/remote_mirror.rs` enqueues transcript lines plus a `sync_local_surface` flag. Surfaces drain this via `drain_remote_mirror_batch()` (FFI) on a timer (iOS: 250ms) to:

1. Display remote command output locally.
2. Trigger a `reload_modules()` when `sync_local_surface` is true (host state changed).

### Theme system

**Desktop** (`Desktop/src/gui/theme/`):
- Named color constants and helper functions — never inline `rgb(0x...)` in view files.
- `icon_path(glyph: &str) -> &str` — maps glyph keys to SVG asset paths.
- `nav_accents/` — per-accent palettes (amber, cyan, emerald, fuchsia, indigo, orange, sky, teal, violet).
- Component tokens under `modules/` — buttons, panels, rows, toggles, typography.

**iOS** (`AppTheme.swift`):
- All colors as computed properties on `AppTheme(isDark:)`.
- No `Color(hex:)` inline anywhere in view files.

### FFI bridge

`ffi.rs` is the UniFFI boundary. All iOS ↔ Rust communication goes through it. Key exports:

**Setup:**
- `set_config_root_path(path: String)` — must be called first on iOS (app sandbox path)

**Command execution:**
- `execute_command(token, args, context: ExecutionContextFfi) -> String`
- `list_commands() -> Vec<CommandInfo>`

**Module control:**
- `list_modules() -> Vec<ModuleStatus>`
- `set_module_enabled(name, enabled) -> String`
- `set_module_enabled_with_requirements(name, enabled) -> String`
- `probe_module_toggle(name, enabled) -> ModuleToggleResult` — preflight check, returns missing deps

**Navigation:**
- `navigation_registry_json() -> String`
- `platform_name() -> String`

**Thin-client:**
- `thin_client_surface_client_id() -> String`
- `thin_client_preferred_route_get() -> Option<String>`
- `thin_client_preferred_route_set(route: String) -> String`

**LAN:**
- `lan_start()`, `lan_stop()`

**Mirror:**
- `drain_remote_mirror_batch() -> RemoteMirrorDrain`

After any change to `ffi.rs` or exported types, run:
```sh
bash Shared/Scripts/build-ios-framework.sh
```
This regenerates `Mobile/iOS/ArcadiaCore/Generated/` and rebuilds `ArcadiaCore.xcframework`.

---

## Module reference

| Module | Name constant | Requires | Description |
|--------|--------------|----------|-------------|
| `net` | `NET_MODULE_NAME` | — | Networking foundation; bootstraps LAN service |
| `lan` | `LAN_MODULE_NAME` | `net` | LAN discovery via UDP; peer management; pairing |
| `surface` | `SURFACE_MODULE_NAME` | — | `surface.snapshot` and `surface.patch` host mirror channel |
| `remote-session` | `REMOTE_SESSION_MODULE_NAME` | `net`, `lan` | Routing gate for LAN command forwarding; no standalone verbs |
| `shell` | `SHELL_MODULE_NAME` | — | `shell.execute` (routable), `shell.internal` (REPL), PTY/TUI on Desktop |
| `shell-motd` | `SHELL_MOTD_MODULE_NAME` | `shell` | Fastfetch-style banner on shell open |

### LAN sub-system (`modules/lan/`)

| Component | File | Purpose |
|-----------|------|---------|
| Service entry | `mod.rs` | `start_service` / `stop_service`, command registry |
| Discovery | `discovery.rs` | Peer scan, node state tracking |
| Handlers | `handlers.rs` | `lan.scan`, `lan.node`, `lan.session_targets`, pairing approval |
| Config | `config.rs` | Approved peers persistence |
| Peers | `peers.rs` | Peer struct and list management |
| Protocol | `protocol.rs` | UDP `NODE_EXEC` and related definitions |

---

## Navigation reference

All 7 pages. Add new pages to `PAGE_DEFINITIONS` in `navigation.rs` — never to surface match arms.

| Page ID | Title | Group | Required Module | Glyph | SF Symbol |
|---------|-------|-------|-----------------|-------|-----------|
| `utility.shell` | Shell | `utilities` | `shell` | `terminal` | `terminal` |
| `global.dashboard` | Dashboard | (global) | — | `home` | `house` |
| `global.logs` | Logs | (global) | — | `logs` | `doc.text` |
| `global.settings` | Settings | (global) | — | `settings` | `gear` |
| `global.modules` | Modules | (global) | — | `modules` | `square.stack.3d.up` |
| `network.overview` | Network | `network` | `net` | `nodes` | `network` |
| `network.nodes` | Nodes | `network` | `lan` | `nodes` | `antenna.radiowaves.left.and.right` |

---

## Repository layout

```
Shared/
  ArcadiaCore/
    Cargo.toml                        # crate-type: staticlib + cdylib + lib
    src/
      lib.rs                          # root, exports + UniFFI scaffolding
      ffi.rs                          # UniFFI → Swift (iOS bridge)
      navigation.rs                   # PAGE_DEFINITIONS, GROUP_DEFINITIONS, registry JSON
      config/
        mod.rs                        # ConfigFile trait, config root path
        modules.rs                    # MODULE_REGISTRY, ModulesConfig, migrations
        commandline.rs                # CLI preferences
        thin_client.rs                # ThinClientConfig → thin-client.toml
      modules/
        mod.rs                        # execute_command dispatcher, module lifecycle
        shell.rs                      # shell.execute, shell.internal, PTY
        shell_motd.rs                 # MOTD banner
        surface.rs                    # surface.snapshot / surface.patch
        remote_session.rs             # routing manifest entry (no standalone commands)
        remote_mirror.rs              # host transcript queue + FFI drain
        net.rs                        # networking foundation
        lan/                          # LAN subsystem (see Module reference)
          mod.rs, discovery.rs, handlers.rs, config.rs, peers.rs, protocol.rs
      platform/
        mod.rs, macos.rs, ios.rs, linux.rs, windows.rs, unknown.rs
  Scripts/
    build-ios-framework.sh            # Rebuild xcframework + Swift bindings
    install-global-commands-macos.sh  # Install ~/.local/bin wrappers
    Launcher.sh / Launcher.ps1        # Shell launcher menus
  Tools/uniffi-bindgen/               # UniFFI bindgen binary (workspace member)

Desktop/
  Cargo.toml                          # features: headless (default), gui
  src/
    main.rs                           # binary entry, feature-gated GUI vs headless
    cli/
      mod.rs                          # REPL loop, startup messages
      args.rs                         # argument parsing
      completion.rs                   # shell completion
      config_cmds.rs                  # module/config CLI commands
      module_cmds.rs                  # module shortcut commands
    gui/
      mod.rs
      assets.rs                       # embedded SVG asset loading
      app/
        mod.rs                        # ArcadiaRoot state, ShellMode enum
        entry.rs                      # GPUI initialization
        lifecycle.rs                  # focus, resize, module reload
        navigation.rs                 # nav state and page routing
        root/mod.rs, render.rs        # root layout + render
        root/top_bar.rs               # title bar, session chip, shell mode toggle
        sidebar/mod.rs, layout.rs, nav_items.rs
        shell/mod.rs, panel.rs, execute.rs, keys.rs, tui_screen.rs, mirror.rs
        modules_page/mod.rs, panel.rs, row.rs, requirements_modal.rs
        lan_nodes/mod.rs, panel.rs
        splash/mod.rs, view.rs, draw_*.rs, math.rs
      theme/
        mod.rs                        # icon_path(), color constants
        chrome.rs                     # window chrome
        icons.rs                      # icon metadata
        splash_colors.rs
        modules/                      # component tokens (buttons, panel, rows, toggles, typography)
        nav_accents/                  # per-accent palettes (mod.rs, palette.rs, 9 accents)
      tui/
        mod.rs, session.rs            # PTY session lifecycle
        ansi_line.rs                  # ANSI escape parsing
        colors.rs                     # terminal color palette
        cd_builtin.rs, cwd.rs, env.rs # shell builtins + CWD tracking
        keys.rs                       # PTY keyboard events
        vt_history.rs                 # VT100 history buffer
  assets/icons/                       # SVG icons (home, terminal, logs, settings, nodes, modules, tools)

Mobile/iOS/
  ArcadiaApp/
    ArcadiaApp.swift                  # @main, config root setup
    ContentView.swift                 # top-level coordinator + Actions/Layout/NavigationState/Registry extensions
    NavigationModels.swift            # Swift structs mirroring NavigationRegistry
    AppTheme.swift                    # all iOS colors as computed properties
    SidebarView.swift                 # sidebar rendering + remote session picker
    SplashView.swift                  # animated splash
    ShellView.swift                   # shell command input + history
    ModulesView.swift                 # module toggle list
    LanNodesView.swift                # LAN peer discovery + pairing
    ModuleNames.swift                 # string constants mirroring MODULE_REGISTRY
    GlassComponents.swift             # reusable glassmorphism components
  ArcadiaCore/                        # Generated Swift + ArcadiaCore.xcframework (rebuild after ffi.rs changes)

Configuration/                        # Layout reference (runtime: ~/Arcadia/Configuration on Desktop)
  modules.toml                        # module enable/disable state
  commandline.toml                    # CLI preferences
  thin-client.toml                    # preferred_remote_route, surface_client_id

Resources/
  Wallpapers/                         # Landscape.png, Portrait.png, Landscape-Refined.png
  Sounds/                             # Notification_* (Warm, Pop, Minimal, Glass, Deep, Airy)
  Icons/                              # App icon prototypes + Final-1-appicon.png

Launchers/Development/OSX/            # SwiftPM menu bar launcher (optional, dev only)
  Package.swift
  Sources/ArcadiaDevelopmentLauncher/main.swift
  build-app.sh, README.md

.github/workflows/
  stable-build-matrix.yml             # Desktop + iOS simulator CI
  FUNDING.yml                         # GitHub Sponsors

gaps.md                               # Deliberate limitations and next-tier work
CLAUDE.md                             # Contributor guide (architecture patterns)
AGENTS.md                             # Agent rules (registry discipline, anti-patterns)
```

---

## Configuration

Runtime config root: `~/Arcadia/Configuration/` on Desktop. iOS sets root via `set_config_root_path` (app sandbox).

| File | Struct | Purpose |
|------|--------|---------|
| `modules.toml` | `ModulesConfig` | Per-module on/off state |
| `commandline.toml` | `CommandlineConfig` | CLI preferences (scaffold) |
| `thin-client.toml` | `ThinClientConfig` | `preferred_remote_route`, `surface_client_id` |

Config migrations live in `ModulesConfig::merge_defaults()`. When renaming a module, add a migration entry there — do not do ad-hoc renames at call sites.

---

## Prerequisites

| Tool | Required for |
|------|-------------|
| Rust (`rustup`, `cargo`) | Core + Desktop |
| Xcode + CLI tools | iOS app + xcframework build |
| `rustup target add aarch64-apple-ios aarch64-apple-ios-sim` | `build-ios-framework.sh` |
| Swift (via Xcode) | iOS app + dev launcher |

---

## Build and run

### Desktop GUI

```sh
cd Desktop && cargo build --features gui
cd Desktop && cargo run --features gui
```

### Desktop CLI (headless)

Default features are `headless`:

```sh
cd Desktop && cargo run
```

### Desktop release

```sh
cd Desktop && cargo build --release --features gui
```

### Core tests

```sh
cd Shared && cargo test -p arcadia-core
```

### iOS framework + Swift bindings

Run after any change to `ffi.rs` or exported types:

```sh
bash Shared/Scripts/build-ios-framework.sh
```

Regenerates `Mobile/iOS/ArcadiaCore/Generated/` and rebuilds `ArcadiaCore.xcframework`. Then open `ArcadiaApp` in Xcode and build.

### Launcher menus

```sh
bash Shared/Scripts/Launcher.sh
pwsh  Shared/Scripts/Launcher.ps1
```

### Global wrappers (macOS)

```sh
bash Shared/Scripts/install-global-commands-macos.sh
```

Installs helpers to `~/.local/bin` — ensure it's on `PATH`.

### macOS dev launcher app

```sh
cd Launchers/Development/OSX && bash build-app.sh
```

See `Launchers/Development/OSX/README.md` for details.

---

## Environment variables

| Variable | Surface | Purpose |
|----------|---------|---------|
| `ARCADIA_NET_AS` | Desktop GUI, iOS | Bootstrap `net_as` on startup (e.g. `lan:192.168.1.5`). Overrides `thin-client.toml` preferred route. |
| `ARCADIA_IOS_DEVICE_NAME` | iOS deploy scripts | Pin device by name |
| `ARCADIA_IOS_FORCE_UNINSTALL` | iOS deploy scripts | Uninstall before install |

---

## Adding features

### New module

1. Add constant + `ModuleManifest` to `MODULE_REGISTRY` in `Shared/ArcadiaCore/src/config/modules.rs`.
2. Create `Shared/ArcadiaCore/src/modules/x.rs` with a `commands()` fn returning `&[ModuleCommand]`.
3. Register in `Shared/ArcadiaCore/src/modules/mod.rs`.
4. Done. GUI, CLI, and iOS module list updates automatically — no surface edits required.

### New navigation page

1. Add `NavigationPageDefinition` to `PAGE_DEFINITIONS` in `navigation.rs`. Set `required_module` if visibility depends on a module.
2. Add the page ID to the relevant `GROUP_DEFINITIONS.pages` slice, or create a new group.
3. Implement the page panel: Desktop → `gui/app/` new panel file; iOS → new view file.
4. Route it in the surface content switch via the page ID — derive visibility from `required_module`, not a hardcoded match.

### New icon/glyph

1. Add SVG to `Desktop/assets/icons/`.
2. Add match arm to `icon_path()` in `Desktop/src/gui/theme/mod.rs`.
3. Use the key in `NavigationPageDefinition.glyph` or `NavigationGroupDefinition.glyph`.

### New theme color

- Desktop: add named constant or helper fn to `Desktop/src/gui/theme/mod.rs` or the relevant component token file under `theme/modules/`.
- iOS: add computed property to `AppTheme` in `AppTheme.swift`.
- Never inline `rgb(0x...)` or `Color(hex:)` in view files.

### New mirrored state

Extend `SurfaceSnapshot.extra` and add a `SurfacePatch` variant in `modules/surface.rs`. Wire both surfaces to consume the new extra field from snapshot. Do not create ad-hoc `remote-session.*` verbs — keep the protocol under `surface.*`.

### Renaming a module

Edit `MODULE_REGISTRY` name and constant. Add a migration to `ModulesConfig::merge_defaults()` following the `LEGACY_LAN_MODULE_NAME` pattern. Do not rename at call sites.

---

## Testing

Current test coverage is sparse. Priority areas for expansion:

```sh
# Run existing tests
cd Shared && cargo test -p arcadia-core

# What to add:
# - surface.snapshot / parse_surface_snapshot round-trips
# - NavigationRegistryOwned JSON serialization/deserialization
# - ModulesConfig migration (merge_defaults with legacy keys)
# - thin-client preference persistence (set → get → re-load)
# - LAN routing integration (execute_command with net_as)
# - Module enable/disable with dependency enforcement
```

iOS `ArcadiaCore.xcframework` rebuild after FFI changes is currently manual. Adding a CI step that fails when `Generated/` drifts from `ffi.rs` is a high-priority gap — see `gaps.md`.

---

## Known gaps and production roadmap

`gaps.md` tracks all deliberate limitations. Summary with priority ranking:

### P0 — Fix before trusting in production

| Gap | Problem | Direction |
|----|---------|-----------|
| **Revision coverage** | `surface.revision` only advances on `surface.patch`. CLI writes and FFI writes bypass it — clients can miss updates. | Bump revision from every `ModulesConfig::save`. |
| **Testing discipline** | No automated tests for snapshot round-trips, thin-client prefs, or LAN routing. | Add targeted `arcadia-core` unit + integration tests. |
| **FFI drift detection** | No CI check that `Generated/` matches `ffi.rs`. | Workflow step: rebuild and fail if diff. |

### P1 — Needed for real multi-user / multi-surface use

| Gap | Problem | Direction |
|----|---------|-----------|
| **Stale UI detection** | Desktop has `last_surface_revision` but never compares it — no "host changed under you" warning. | Compare revision on timer/focus/after routed command; optional banner + reload. |
| **Multi-writer** | Multiple GUIs on same host = last write wins, no merge, no locks. | Document as permanent constraint OR add optimistic concurrency (generation tokens on save). |
| **Transport** | Command routing is request/response UDP. No long-lived session, no ordering guarantees, no subscription for deltas. | Optional WebSocket/TCP sidecar for continuous thin-shell workflows. |

### P2 — Required before leaving trusted LAN

| Gap | Problem | Direction |
|----|---------|-----------|
| **Security posture** | No wire encryption, no auth beyond "approved node," no scoped capabilities. `shell.execute` routable to anyone approved. | Threat model doc + TLS or pairing secrets + capability tokens. |
| **Identity** | `client_id` is attribution only — no authz, no rate limits, no per-client filtering. | Host-side policy module or capability tokens if multi-tenant. |

### P3 — Polish and convergence

| Gap | Problem | Direction |
|----|---------|-----------|
| **Surface parity** | Desktop has PTY/TUI paths; iOS is shell.execute only; not all panels are execute-only. | Converge per capability class with explicit "unavailable on this surface" from core. |
| **Renderer-only client** | Surfaces still bundle compiled nav — no enforced "remote-only" profile. | Optional build flag that refuses static nav when `remote_route` is mandatory. |
| **`extra` schema** | `extra.navigation_registry` is wired; broader extra buckets and corresponding `SurfacePatch` variants are undefined. | Define schema + version fields inside `extra`; extend `SurfacePatch` incrementally. |

---

## Security posture

Current trust model: **LAN pairing + locally approved peers.** Assume trusted network.

What this means in practice:
- Any approved LAN peer can execute any command the host has enabled, including `shell.execute`.
- `surface.patch` is unauthenticated beyond `client_id` (which is just a UUID, not a secret).
- There is no encryption on the wire.

**Do not expose Arcadia to untrusted networks without addressing P2 gaps above.** This is a home-network / trusted-LAN tool today. Production-grade multi-tenant use requires TLS, capability tokens, and a real threat model document first.

---

## CI

`.github/workflows/` — `stable-build-matrix.yml` builds Desktop targets and iOS simulator configs on selected branches. See individual workflow files for triggers and matrix.

Gaps in CI coverage: FFI drift detection, core integration tests. See [Testing](#testing).

---

## Contributing

Read `AGENTS.md` — it has the registry-discipline rules and the full list of anti-patterns we refuse to write. Short version:

1. **Registry entry before surface code.** New module? `MODULE_REGISTRY` first. New page? `PAGE_DEFINITIONS` first.
2. **No per-module booleans in surface state.** One generic `is_module_enabled(name)` query.
3. **No hardcoded page ID match arms in visibility logic.** Derive from `required_module` in `PageDefinition`.
4. **No inline colors.** Theme layer only.
5. **Cross-platform logic belongs in core.** If you're writing the same thing in `app.rs` and `ContentView.swift`, it's core logic.
6. **After FFI changes:** run `build-ios-framework.sh` and commit `Generated/` + `xcframework`.

If something's missing: open a PR, draft a module, or file an issue with a concrete repro.

---

## Lineage

**[Holos](https://github.com/stack-node/holos)** — macOS-first, modular, "built out of utility and spite" against rent-seeking micro-apps.

**Arcadia** — same DNA (free, open, yours), different chassis: Rust core, cross-platform surfaces, explicit LAN routing, `surface.*` mirror channel, and agent-enforced registry patterns so the codebase stays honest as it grows.

---

## About the creator

I'm a twenty-something British developer.

Moved to the US in 2016 chasing family — it didn't pan out how you'd hope. Along the way I fell hard into **electricity**, then **hardware**, then **software**. Spent years in demanding jobs (including **Disney** and **government** work): solid craft, solid burnout, and a growing dislike of systems that optimize **rent** over **agency**.

Eventually I hit a wall, stepped back, and landed back in the **UK** to rebuild — **tired**, **broke**, and dealing with **chronic insomnia**.

Turns out insomnia leaves a lot of hours for building.

**[Holos](https://github.com/stack-node/holos)** was one outlet — macOS-first, modular, angry at menu-bar subscriptions.

**Arcadia** is the next chapter: **Rust**, **multi-platform**, **one honest core**, **LAN-aware surfaces**, and the same underlying attitude — tools you own, not dashboards that invoice you.

---

## Donations

There is a donation link (when I've remembered to wire it somewhere sensible — check the GitHub profile, repo Sponsors, or releases if it's live).

You probably shouldn't use it.

Any money would realistically help with boring friction — Apple Developer fees, hardware for iOS builds — which sits in tension with the "don't feed the rent-seekers" ethos of these projects. It would still help Arcadia and Holos reach their technical potential.

If you donate anyway and you'd rather that money not go toward licenses or anything in that vein, say so — I'd rather put it toward something human. I'm saving toward a cat; until that's sorted, that's the soft default. After that — or if you explicitly ask that I not keep any of it — donations marked "don't support the system" can go to my local animal shelter.

No obligation. **Code and issues beat coffee money every time.**

---

## Final note

Arcadia is meant to be **yours**: fork it, break it, fix it, route it across your LAN, disable half the modules, wire something weird into `surface.patch`.

If it helps you replace a pile of tiny apps or own your automation stack, feed that back as code or docs — not hype.

Make something useful. Make something weird. Make something only you care about.

That's still the point — just with one Rust core keeping the story straight.
