# LDB-Engine (Linux Database Engine)

## What This Is

A lightweight, GUI-driven local database manager for Linux developers, providing a "1-click" setup experience powered by Docker under the hood. Built with Tauri v2 + Svelte + Rust.

## Core Value

Enable developers to spin up local database instances (PostgreSQL, Redis, MySQL, MongoDB) with a single click, without manual Docker configuration.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] FR1: Docker Hub Integration & Image Discovery
- [ ] FR2: Instance Creation & Configuration
- [ ] FR3: Instance Lifecycle Management
- [ ] FR4: Persistent Data Management
- [ ] FR5: Connection Utilities
- [ ] FR6: Real-time Log Viewer

### Out of Scope

- Multi-node clustering — single instance only
- Cloud deployment — Linux local only
- User management/auth — single user local app

## Context

- **Starting point:** Fresh Tauri v2 starter kit with Svelte frontend
- **Tech stack:** Rust backend, Tauri v2, Svelte, Tailwind CSS v4, Shadcn/ui, Docker SDK
- **Target users:** Linux developers who need quick local databases

## Constraints

- **Wayland/Niri:** Must work natively on Wayland without XWayland
- **Performance:** Under 50MB binary, under 2s startup, minimal memory footprint
- **Distribution:** AUR package primary, Flatpak secondary

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri v2 | Modern, lightweight, Rust-native | — Pending |
| Shadcn/ui | Developer-focused aesthetic, accessible | — Pending |
| SQLite for state | Simple, Rust-native, sufficient for metadata | — Pending |

---
*Last updated: 2025-02-27 after initialization*
