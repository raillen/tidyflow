<p align="center">
  <img src="./icon.svg" alt="TidyFlow logo" width="88" height="88">
</p>

<h1 align="center">TidyFlow</h1>

<p align="center">
  Local-first desktop automation for organizing, moving, copying, and auditing file workflows.
</p>

<p align="center">
  <a href="./README.md">English</a>
  ·
  <a href="./README.pt-BR.md">Português</a>
  ·
  <a href="./README.ja-JP.md">日本語</a>
</p>

<p align="center">
  <img alt="Version" src="https://img.shields.io/badge/version-0.2.1--alpha-blue">
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-2-24C8DB">
  <img alt="Svelte" src="https://img.shields.io/badge/Svelte-5-FF3E00">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-stable-000000">
  <img alt="License" src="https://img.shields.io/badge/license-package--level-green">
</p>

<p align="center">
  <img src="./interface-uiv2.png" alt="TidyFlow desktop interface showing the dashboard and automation panels">
</p>

## Menu

- [Overview](#overview)
- [Project Status](#project-status)
- [What TidyFlow Does](#what-tidyflow-does)
- [Architecture](#architecture)
- [Repository Layout](#repository-layout)
- [Tech Stack](#tech-stack)
- [Getting Started](#getting-started)
- [Development Commands](#development-commands)
- [Documentation Map](#documentation-map)
- [Security and Privacy](#security-and-privacy)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Support](#support)

## Overview

TidyFlow is a desktop app for people who manage repeated file operations and need clear control over what happened, what is running, and what will run next.

It combines a Svelte interface, a Tauri desktop shell, and a Rust core. The app is designed to run locally, store operational data in SQLite, and keep file operations behind explicit Rust commands instead of giving the frontend direct filesystem access.

The current codebase is the v2 rewrite. Some internal crates and older technical documents still use the `autoflow-*` namespace. The public product name is **TidyFlow**.

## Project Status

| Item | Status |
| --- | --- |
| Release | `0.2.1-alpha` |
| Product name | TidyFlow |
| Internal Rust namespace | `autoflow-*` |
| Desktop shell | Tauri 2 |
| Frontend | Svelte 5 + TypeScript |
| Backend core | Rust workspace |
| Persistence | SQLite through `sqlx` |
| CI | GitLab pipeline for UI checks, Rust tests, and UI tests |

This is an alpha project. The main flows exist, but APIs, UI details, and packaging can still change before a stable release.

## What TidyFlow Does

| Area | Description |
| --- | --- |
| Transfer jobs | Create copy or move jobs with source, target, filters, conflict rules, and runtime options. |
| Simulation | Preview job and blueprint results before changing files on disk. |
| Watch folders | Monitor directories and enqueue work when relevant file events settle. |
| Scheduling | Run jobs on configured intervals or calendar-based schedules. |
| Blueprints | Organize files or folders with template-based naming, counters, folder plans, and variable input. |
| Audit trail | Query processed events, inspect details, summarize activity, and export logs as CSV or JSON. |
| Admin panel | Inspect local fleet data, active jobs, command queues, and signed heartbeat payloads for managed-agent workflows. |
| Settings | Configure appearance, language, performance limits, security, notifications, maintenance, and support options. |

TidyFlow is built for practical file automation, not for hidden background behavior. The UI keeps execution status, recent activity, failures, and upcoming runs visible.

## Architecture

```text
Svelte UI
  -> Tauri commands and events
    -> Rust application services
      -> Domain rules
      -> SQLite repositories
      -> Filesystem, watcher, scheduler, and notifications
```

Core boundaries:

| Layer | Location | Responsibility |
| --- | --- | --- |
| Desktop app | `apps/desktop` | Svelte routes, UI contracts, Tauri shell, app configuration. |
| Domain | `crates/autoflow-domain` | Jobs, blueprints, filters, schedules, settings, audit types, tokenizer, path rules. |
| Application | `crates/autoflow-application` | Use cases for jobs, execution, blueprints, scripts, notifications, and packages. |
| Infrastructure | `crates/autoflow-infrastructure` | SQLite stores, migrations, persisted settings, audit, secrets, and UI state. |
| Core runtime | `crates/autoflow-core` | App state, queue, scheduler, watchers, admin actions, and event emission. |
| Admin server | `crates/autoflow-admin-server` | HTTP-facing administrative primitives for remote management workflows. |

## Repository Layout

```text
.
├── apps/
│   └── desktop/                 # Tauri 2 + Svelte 5 desktop application
├── crates/
│   ├── autoflow-domain/         # Pure domain model and rules
│   ├── autoflow-application/    # Use cases and orchestration
│   ├── autoflow-infrastructure/ # SQLite, settings, audit, secrets, storage
│   ├── autoflow-core/           # Runtime wiring, queue, scheduler, watchers
│   └── autoflow-admin-server/   # Admin server primitives
├── docs/
│   ├── v2/                      # Architecture, IPC, roadmap, stack notes
│   ├── WIKI/                    # User-facing wiki pages
│   ├── adr/                     # Architecture decision records
│   ├── plans/                   # Planning notes and feature analysis
│   └── specs/                   # Contracts and schemas
├── tests/
│   └── e2e/                     # Playwright tests
├── Cargo.toml                   # Rust workspace
├── package.json                 # pnpm workspace scripts
├── pnpm-workspace.yaml
└── README.md
```

## Tech Stack

| Area | Tools |
| --- | --- |
| Desktop | Tauri 2 |
| UI | Svelte 5, SvelteKit, TypeScript, Vite |
| UI validation | Zod, Vitest, Playwright |
| Rust async runtime | Tokio |
| Persistence | SQLite, `sqlx`, migrations |
| File watching | `notify`, `notify-debouncer-full` |
| HTTP/admin | Axum, Reqwest |
| Packaging | Tauri bundler |
| Workspace | pnpm workspaces + Cargo workspace |

## Getting Started

### Prerequisites

- Node.js 20 LTS or newer
- pnpm through Corepack
- Rust stable toolchain
- System dependencies required by Tauri for your operating system

### Install

```bash
git clone https://github.com/raillen/tidyflow.git
cd tidyflow
corepack enable
pnpm install
```

### Run the desktop app

```bash
pnpm dev
```

The root `dev` script starts the Tauri desktop app through the `@tidyflow/desktop` workspace package.

## Development Commands

| Command | Purpose |
| --- | --- |
| `pnpm dev` | Run the Tauri + Svelte desktop app in development mode. |
| `pnpm build` | Build the desktop application bundle. |
| `pnpm check` | Run Svelte and TypeScript checks for the desktop package. |
| `pnpm test` | Run frontend unit tests. |
| `pnpm test:rust` | Run `cargo test --workspace`. |
| `pnpm --filter @tidyflow/desktop check` | Check only the desktop workspace package. |
| `cargo test --workspace` | Run all Rust tests directly. |

## Documentation Map

| Document | Use it for |
| --- | --- |
| [`docs/v2/ARCHITECTURE.md`](./docs/v2/ARCHITECTURE.md) | Runtime architecture, layers, and flows. |
| [`docs/v2/API-IPC.md`](./docs/v2/API-IPC.md) | Tauri command and event contracts. |
| [`docs/v2/DOMAIN.md`](./docs/v2/DOMAIN.md) | Domain entities, invariants, and rules. |
| [`docs/v2/ROADMAP.md`](./docs/v2/ROADMAP.md) | Planned phases and delivery criteria. |
| [`docs/v2/STACK.md`](./docs/v2/STACK.md) | Technical stack and library choices. |
| [`docs/WIKI/User-Guide.md`](./docs/WIKI/User-Guide.md) | User-facing guide. |
| [`docs/adr/`](./docs/adr/) | Architecture decisions. |

The v2 documentation is being aligned with the current alpha code. When there is a conflict, prefer the code, manifests, and this root README for the public project overview.

## Security and Privacy

TidyFlow is designed around local execution and explicit boundaries:

- Runtime data is stored locally in SQLite.
- File operations go through Rust commands.
- Path validation is centralized in domain/core rules.
- The UI does not own low-level filesystem behavior.
- Admin-agent secrets are handled through application settings and the platform keyring path.
- Destructive admin commands are represented as explicit command requests.

Alpha note: review package manifests, settings, and deployment configuration before using TidyFlow in sensitive production workflows.

## Roadmap

Near-term work is focused on stabilizing the v2 foundation:

- harden transfer execution and cancellation;
- keep job, watch, schedule, and blueprint flows consistent;
- improve translated UI text across supported locales;
- finish packaging and updater configuration;
- expand Playwright coverage for complete desktop workflows;
- align older AutoFlow documents with the current TidyFlow product name.

See [`docs/v2/ROADMAP.md`](./docs/v2/ROADMAP.md) for the detailed plan.

## Contributing

Contributions are welcome while the project is in alpha. Keep changes small and verifiable.

Before opening a pull request:

1. Run the relevant checks.
2. Update documentation when behavior, setup, commands, or contracts change.
3. Keep UI copy short, concrete, and accessible.
4. Avoid placeholder UI for incomplete features.
5. Mention known limitations clearly.

Recommended checks:

```bash
pnpm check
pnpm test
pnpm test:rust
```

## License

TidyFlow uses package-level licensing. Package manifests are the source of truth for each part of the repository.

| Area | License |
| --- | --- |
| `crates/autoflow-domain` | `MIT OR Apache-2.0` |
| `crates/autoflow-core` | `MIT OR Apache-2.0` |
| Workspace crates and desktop application that inherit workspace metadata | `GPL-3.0-only` |

See [`LICENSE`](./LICENSE), [`LICENSE-MIT`](./LICENSE-MIT), and [`LICENSE-APACHE`](./LICENSE-APACHE).

## Support

TidyFlow is developed by [Raillen Santos](https://github.com/raillen).

- Website: [raillen.site](https://raillen.site)
- GitHub: [github.com/raillen](https://github.com/raillen)
- Buy Me a Coffee: [buymeacoffee.com/raillen](https://www.buymeacoffee.com/raillen)
