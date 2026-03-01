# LDB-Engine (Linux Database Engine)

## What This Is

A lightweight, GUI-driven local database manager for Linux developers, providing a "1-click" setup experience powered by Docker under the hood. Built with Tauri v2 + Svelte + Rust.

## Core Value

Enable developers to spin up local database instances (PostgreSQL, Redis, MySQL, MongoDB) with a single click, without manual Docker configuration.

## Requirements

### Validated

- ✅ FR1: Docker Hub Integration & Image Discovery — v1.0
- ✅ FR2: Instance Creation & Configuration — v1.0
- ✅ FR3: Instance Lifecycle Management — v1.0
- ✅ FR4: Persistent Data Management — v1.0
- ✅ FR5: Connection Utilities — v1.0
- ✅ FR6: Real-time Log Viewer — v1.0

### Active

(None yet — start next milestone with `/gsd-new-milestone`)

### Out of Scope

- Multi-node clustering — single instance only
- Cloud deployment — Linux local only
- User management/auth — single user local app
- Built-in SQL query editor — beyond core value
- Remote management — local-first design

## Context

- **Current milestone:** v1.0 MVP shipped 2026-03-01
- **Tech stack:** Rust backend, Tauri v2, Svelte 5, Tailwind CSS v4, Shadcn/ui, bollard 0.20 (Docker SDK)
- **Stats:** ~15,846 LOC, 53 files, 12 plans, 2 days to ship
- **Target users:** Linux developers who need quick local databases

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri v2 | Modern, lightweight, Rust-native | ✅ Shipped v1.0 |
| Shadcn/ui | Developer-focused aesthetic, accessible | ✅ Shipped v1.0 |
| SQLite for state | Simple, Rust-native, sufficient for metadata | ✅ Shipped v1.0 |
| bollard 0.20 | Docker API integration | ✅ Shipped v1.0 |
| Svelte 5 runes | Modern reactivity | ✅ Shipped v1.0 |
| Container naming "ldb-" prefix | Identify our containers | ✅ Shipped v1.0 |
| Modal overlay for log viewing | Clean UI pattern | ✅ Shipped v1.0 |

---

*Last updated: 2026-03-01 after v1.0 milestone*
