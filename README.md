# Arcadia

**A multi-platform runtime and shell: one Rust core, thin native surfaces, and LAN-aware control—built to be extended, not rented.**

Arcadia is what happens when you take the same instincts behind **[Holos](https://github.com/stack-node/holos)**—*utility over monetization, modules over lock-in, ownership over subscriptions*—and carry them further: **Rust everywhere it matters**, **macOS + iOS**, **CLI + GPUI**, **optional thin-client routing over the LAN**, and **zero tolerance for duplicated truth** between surfaces.

---

## Table of contents

- [Why Arcadia exists](#why-arcadia-exists)
- [What Arcadia is](#what-arcadia-is)
- [What you can do with it](#what-you-can-do-with-it)
- [Development status](#development-status)
- [Philosophy](#philosophy)
- [Architecture (technical)](#architecture-technical)
- [Repository layout](#repository-layout)
- [Configuration](#configuration)
- [Prerequisites](#prerequisites)
- [Build and run](#build-and-run)
- [Environment variables](#environment-variables)
- [Adding features](#adding-features)
- [Thin clients, snapshots, and remote control](#thin-clients-snapshots-and-remote-control)
- [Known gaps & roadmap](#known-gaps--roadmap)
- [CI](#ci)
- [Contributing](#contributing)
- [Lineage](#lineage)
- [About the creator](#about-the-creator)
- [Donations](#donations)
- [Final note](#final-note)

---

## Why Arcadia exists

Small-tool ecosystems trend the same way: **paywalls**, **subscriptions**, **feature flags**, and **“AI-generated app of the week”** churn. Even good ideas get trapped in silos—one app for the menu bar, one for the terminal, one for “sync,” each with its own settings schema and no escape hatch.

**Holos** pushed back on that on macOS: modular, free, yours to extend.

**Arcadia** pushes back harder:

- **One core** (`arcadia-core`) owns modules, commands, config, navigation metadata, and LAN plumbing—surfaces are **render + dispatch**, not second implementations.
- **Multiple surfaces** from the same logic: terminal REPL, desktop GUI, pocket UI—without forking behavior per platform.
- **Optional “headless host + GUI client”** patterns over the LAN so your MacBook can drive your phone—or vice versa—without inventing a new protocol per feature.

If something’s missing, you **add a module** or **extend `surface.snapshot` / `surface.patch`**, not buy another app.

---

## What Arcadia is

- **Always intended to stay open and hackable** — no artificial paywalls in the architecture; the repo is the product.
- **Actually structured for real use** — registry-driven modules, typed navigation, thin-client snapshots—not a demo scaffold.
- **Modular by design** — enable dependencies from **`MODULE_REGISTRY`**; pages declare **`required_module`** instead of surfaces hardcoding visibility.
- **Multi-platform by default** — Desktop (**Rust + GPUI / CLI**) and iOS (**SwiftUI + UniFFI**) consume the **same** core.
- **Built to be extended, not monetized** — new capability → register in core → every surface picks it up.

It starts **minimal**: you turn modules on, you route commands how you want (local or **`lan:…`**). Nothing requires a vendor dashboard.

---

## What you can do with it

- **Run a native shell / terminal workflow** on desktop (**`shell.execute`**, PTY/TUI paths where enabled).
- **Manage modules and configuration** from CLI or GUI; same **`modules.toml`** semantics everywhere.
- **Discover and pair LAN peers** (**`lan.scan`**, **`lan.node`**, nodes UI on desktop / mobile).
- **Route commands to another machine** on your LAN (**`ExecutionContext.net_as`**, session chip on desktop, route picker on iOS).
- **Mirror host UI state** across peers via **`surface.snapshot`** (modules + navigation JSON + revision) and **`surface.patch`** (tagged operations—today **`modules_set`**, designed for more).
- **Run headless** (`Desktop` without **`gui`**) as a long-lived **host** while another GUI acts as a **thin client**.
- **Rebuild iOS** after FFI changes using **`Shared/Scripts/build-ios-framework.sh`** so Swift stays in lockstep with Rust.

---

## Development status

This project **moves fast** and **breaks occasionally**.

- Features land **continuously** on branches like **`development`**.
- APIs (**especially FFI** and **`surface.*`**) may evolve—see **`gaps.md`** for intentional limitations (revision coverage, multi-writer semantics, transport depth).
- **Building from source** is the surest way to stay current.
- **Stable / tagged builds** will appear as the project matures; CI exercises desktop + iOS simulator paths (see **`.github/workflows/`**).

If something’s marked rough, it probably is—the difference is we track known gaps in-repo instead of pretending shipping equals finished.

---

## Philosophy

**Fat core, thin shells.**  
Business logic lives in **`Shared/ArcadiaCore`**. Desktop and iOS **read registries**, **render**, and **`execute_command`**—they do not re-implement module graphs or navigation trees.

**Single sources of truth.**

| What | Where |
|------|--------|
| Module manifests + deps | **`MODULE_REGISTRY`** · `Shared/ArcadiaCore/src/config/modules.rs` |
| Navigation pages + groups | **`PAGE_DEFINITIONS` / `GROUP_DEFINITIONS`** · `navigation.rs` |
| Serializable nav for snapshots | **`NavigationRegistryOwned`** · embedded in **`surface.snapshot`** |
| Theme tokens | Desktop **`gui/theme/`** · iOS **`AppTheme.swift`** |

**Extend the registry, not scatter `if pageId == …`.**  
See **`AGENTS.md`** for anti-patterns we refuse (named module booleans, magic page IDs in visibility, hex colors in views).

**Personal tool energy, public repo.**  
If Arcadia helps others, great—that’s bonus. The goal is **a system you own**, **can fork**, and **can route across machines you trust**.

*(Holos wore **95% spite / 5% usefulness** proudly. Arcadia keeps the spirit—**pushback on rent-seeking tooling**—but trades the ratio for **engineering spite**: fewer duplicated definitions, fewer lies between CLI and GUI.)*

---

## Architecture (technical)

### Command model

- Tokens: **`module.command`** — e.g. **`shell.execute`**, **`lan.scan`**, **`surface.snapshot`**, **`surface.patch`**.
- **`execute_command`** (`modules/mod.rs`) handles local dispatch **or** LAN forward when **`ExecutionContext.net_as`** is set (e.g. **`lan:192.168.1.10`**).
- Forwarding requires **local** **`remote-session`**, **`lan`**, and **`net`** enabled; the **peer** enforces module rules for the token.

### Modules of note

| Module | Role |
|--------|------|
| **`surface`** | **`surface.snapshot`** / **`surface.patch`** — generic host UI mirror channel (**`modules`**, **`revision`**, **`extra.navigation_registry`**). |
| **`remote-session`** | **Routing gate only** — no standalone mirror verbs; pairing + approval live under **`lan`**. |
| **`lan` / `net`** | Discovery, UDP **`NODE_EXEC`**, **`lan.session_targets`** for picker JSON. |
| **`shell`**, **`shell-motd`**, … | Feature modules registered like any other. |

### Desktop (`Desktop/`)

- Cargo features: default **`headless`**; **`gui`** enables GPUI (`main` selects GUI when **`gui`** is on).
- **`gui/app/`** — lifecycle, sidebar, shell/TUI, modules, LAN nodes, session route chip.
- Theme — centralized; **no raw RGB** in arbitrary view files (**`theme/`**).

### iOS (`Mobile/iOS/`)

- **`set_config_root_path`** early (sandbox).
- **`navigation_registry_json()`** for local nav; remote **`surface.snapshot`** can replace **`NavigationRegistry`** from **`extra.navigation_registry`**.
- UniFFI exports include **`execute_command`**, navigation JSON, **`thin_client_*`**, **`drain_remote_mirror_batch`**, etc.

### Remote mirror (host)

When this machine executes inbound **`NODE_EXEC`** for a peer, **`remote_mirror`** can enqueue transcript lines + “resync local UI” hints so surfaces showing **local** state stay coherent—see **`modules/remote_mirror.rs`** and FFI **`RemoteMirrorDrain`**.

---

## Repository layout

```
Shared/ArcadiaCore/                 # Crate: arcadia-core (staticlib / cdylib / lib)
  src/
    lib.rs
    ffi.rs                          # UniFFI → Swift
    navigation.rs                   # Static defs + NavigationRegistryOwned JSON
    config/
      modules.rs                    # MODULE_REGISTRY, ModulesConfig
      thin_client.rs                # ThinClientConfig → thin-client.toml
      …
    modules/
      shell.rs, net.rs, lan/, …
      surface.rs                    # snapshot / patch / revision
      remote_session.rs             # routing-only manifest entry
      remote_mirror.rs              # host mirror queue + FFI drain

Desktop/                            # Binary: arcadia
  src/main.rs, cli/, gui/

Mobile/iOS/
  ArcadiaApp/                       # SwiftUI
  ArcadiaCore/                      # Generated Swift + ArcadiaCore.xcframework

Shared/Scripts/
  build-ios-framework.sh
  Launcher.sh / Launcher.ps1
  install-global-commands-macos.sh

Resources/                          # Icons, wallpapers, sounds
Configuration/                      # Layout reference (runtime: ~/Arcadia/Configuration on Desktop)
Launchers/Development/OSX/        # Optional dev launcher (SwiftPM .gitignored under .build/)
.github/workflows/

gaps.md                             # Deliberate limitations & “ultimate” follow-ups
CLAUDE.md / AGENTS.md               # Contributor & agent rules
```

---

## Configuration

Default config root: **`~/Arcadia/Configuration/`** (see **`Shared/ArcadiaCore/src/config/mod.rs`**).  
iOS sets root via **`set_config_root_path`** (app container).

| File | Purpose |
|------|---------|
| **`modules.toml`** | **`ModulesConfig`** — per-module on/off |
| **`commandline.toml`** | CLI preferences |
| **`thin-client.toml`** | **`preferred_remote_route`** (`lan:…`), **`surface_client_id`** (UUID for **`surface.patch`**) |

LAN pairing / approval flows live under **`modules/lan/`** and related config.

---

## Prerequisites

| Tool | Used for |
|------|-----------|
| Rust (`rustup`, `cargo`) | Core + Desktop |
| Xcode + CLI tools | iOS app + xcframework |
| **`rustup target`** `aarch64-apple-ios`, `aarch64-apple-ios-sim` | **`build-ios-framework.sh`** |

---

## Build and run

### Desktop GUI

```sh
cd Desktop && cargo build --features gui && cargo run --features gui
```

### Desktop CLI (headless)

Default features are **`headless`**:

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

### iOS framework + Swift bindings (**do this after `ffi.rs` or exported API changes**)

```sh
bash Shared/Scripts/build-ios-framework.sh
```

Refreshes **`Mobile/iOS/ArcadiaCore/Generated/`** and **`ArcadiaCore.xcframework`**. Then build **`ArcadiaApp`** in Xcode.

### Launcher menus

```sh
bash Shared/Scripts/Launcher.sh
pwsh Shared/Scripts/Launcher.ps1
```

### Global wrappers (macOS)

```sh
bash Shared/Scripts/install-global-commands-macos.sh
```

Installs helpers into **`~/.local/bin`** — ensure it’s on **`PATH`**.

### macOS dev launcher app

See **`Launchers/Development/OSX/README.md`**.

---

## Environment variables

| Variable | Where | Purpose |
|----------|-------|---------|
| **`ARCADIA_NET_AS`** | Desktop GUI, iOS | Bootstrap **`net_as`** (e.g. **`lan:192.168.1.5`**). Overrides **`thin-client.toml`** on startup. |
| **`ARCADIA_IOS_DEVICE_NAME`** | iOS deploy scripts | Pin device by name |
| **`ARCADIA_IOS_FORCE_UNINSTALL`** | iOS deploy scripts | Uninstall before install |

---

## Adding features

- **New module:** **`MODULE_REGISTRY`** + **`modules/<name>.rs`** + **`modules/mod.rs`** — details in **`CLAUDE.md`**.
- **New navigation page:** **`PAGE_DEFINITIONS` / `GROUP_DEFINITIONS`** + surface panel; visibility from **`required_module`**, not surface **`match` spam**.
- **New mirrored UI state:** extend **`SurfaceSnapshot.extra`** and **`SurfacePatch`** in **`modules/surface.rs`** — avoid one-off **`remote-session.*`** verbs.

---

## Thin clients, snapshots, and remote control

- **`surface.snapshot`** — JSON: **`modules`**, **`revision`**, **`extra.navigation_registry`** (host nav for clients).
- **`surface.patch`** — JSON array of tagged ops (**`modules_set`** today); optional **`client_id`** (**`thin-client.toml`**).
- **`lan.session_targets`** — picker data for approved connected peers.
- **Multi-client caveat:** host **`modules.toml`** is shared — concurrent edits are **last writer wins**; **`revision`** is a partial freshness signal (see **`gaps.md`**).

Desktop persists **`preferred_remote_route`** when you pick **Local** vs a peer; iOS uses **`thinClientPreferredRouteSet`**.

---

## Known gaps & roadmap

Deliberate limitations and next-tier work (**revision vs CLI saves**, stale UI detection, transport/session depth, authz, testing discipline) live in **`gaps.md`**. Read that before assuming “production-grade sync” is solved.

---

## CI

**`.github/workflows/`** — builds desktop targets and iOS simulator-related configs on selected branches (see individual workflows for triggers).

---

## Contributing

- Read **`AGENTS.md`** — registry-driven discipline beats shortcuts.
- Extend **`MODULE_REGISTRY`** / **`PAGE_DEFINITIONS`** instead of hardcoding module/page IDs in GUI/Swift.
- After FFI changes: run **`build-ios-framework.sh`** and commit **`Generated/`** + **`xcframework`** per team/release practice.

If something’s missing: **open a PR**, **draft a module**, or **file an issue with a concrete repro**.

---

## Lineage

**[Holos](https://github.com/stack-node/holos)** — macOS-first, modular, “built out of utility and spite” against rent-seeking micro-apps.

**Arcadia** — same **DNA** (free, open, yours), **different chassis**: **Rust core**, **cross-platform surfaces**, **explicit LAN routing**, **`surface.*` mirror channel**, and **agent-enforced registry patterns** so the codebase stays honest as it grows.

---

## About the creator

I’m a twenty-something British developer.

Moved to the US in 2016 chasing family—it didn’t pan out how you’d hope. Along the way I fell hard into **electricity**, then **hardware**, then **software**. Spent years in demanding jobs (including **Disney** and **government** work): solid craft, solid burnout, and a growing dislike of systems that optimize **rent** over **agency**.

Eventually I hit a wall, stepped back, and landed back in the **UK** to rebuild—**tired**, **broke**, and dealing with **chronic insomnia**.

Turns out insomnia leaves a lot of hours for **building**.

**[Holos](https://github.com/stack-node/holos)** was one outlet for that—macOS-first, modular, angry at menu-bar subscriptions.

**Arcadia** is the next chapter: **Rust**, **multi-platform**, **one honest core**, **LAN-aware surfaces**, and the same underlying attitude—**tools you own**, not dashboards that invoice you.

---

## Donations

There **is** a donation link (when I’ve remembered to wire it somewhere sensible—check the **GitHub profile**, **repo Sponsors**, or **releases** if it’s live).

You probably **shouldn’t** use it.

Any money would realistically help with boring friction—**Apple Developer** fees, hardware for iOS builds, that sort of thing—which sits in tension with the **“don’t feed the rent-seekers”** ethos of these projects. It would still help Arcadia and Holos reach their **technical** potential.

If you donate anyway and you’d rather that money **not** go toward licenses or anything in that vein, say so—I’d **rather** put it toward something **human**. I’m saving toward a **cat**; until that’s sorted, that’s the soft default. **After that**—or if you explicitly ask that I **not** keep any of it—donations marked **“don’t support the system”** (or the same in a note) can go to my **local animal shelter**.

No obligation. **Code** and **issues** beat **coffee money** every time.

---

## Final note

Arcadia is meant to be **yours**: fork it, break it, fix it, route it across **your** LAN, disable half the modules, wire something weird into **`surface.patch`**.

If it helps you **replace a pile of tiny apps** or **own your automation stack**, feed that back as **code or docs**—not hype.

Make something useful. Make something weird. Make something only you care about.

That’s still the point—just with **one Rust core** keeping the story straight.
