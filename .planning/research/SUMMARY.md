# Research Summary: LDB-Engine Stack

**Project:** LDB-Engine (Linux Database Engine) - A lightweight, GUI-driven local database manager for Linux developers with 1-click Docker setup.

**Domain:** Desktop application with Docker integration
**Researched:** February 27, 2026
**Overall confidence:** HIGH

## Executive Summary

This stack research confirms the originally proposed technology choices are sound and provides specific version recommendations with rationale. The Tauri v2 + Svelte 5 + bollard combination is well-supported and actively maintained. Key findings:

1. **bollard** is the only actively maintained Rust Docker SDK (version 0.20.x), replacing the abandoned `shiplift`
2. **@xterm/xterm** (not the deprecated `xterm` package) is the terminal emulator choice
3. **Tauri v2** uses WebKit2GTK-4.1 with good Wayland support on Linux
4. **Tokio** is the de facto async runtime (async-std is discontinued)
5. **Tailwind CSS v4** + **shadcn-svelte** is the recommended UI stack

## Key Findings

### Stack
- Tauri v2 (2.10.x) + Svelte 5 + SvelteKit 2.x + Tailwind CSS v4
- bollard 0.20.x for Docker API, tokio for async runtime
- @xterm/xterm 5.5.x for terminal emulation

### Architecture
- IPC command pattern with Tauri invoke()
- Frontend: Svelte + xterm.js terminal
- Backend: Rust with bollard for Docker, SQLite for local storage

### Critical Pitfall
- Avoid deprecated `xterm` npm package (use `@xterm/xterm`)
- Avoid abandoned `shiplift` crate (use `bollard`)

## Implications for stack is Roadmap

The well-established with:

1. **Phase 1:** Core Tauri + Svelte setup with SQLite storage
2. **Phase 2:** Docker integration via bollard (containers, images)
3. **Phase 3:** Terminal emulation with xterm.js for container exec

**Phase ordering rationale:**
- Frontend shell first (faster iteration)
- Database layer before Docker (foundation)
- Docker integration next (core feature)
- Terminal last (differentiator)

**Research flags for phases:**
- Phase 1-2: Standard patterns, unlikely to need deep research
- Phase 3 (Terminal): May need research on WebSocket streaming from bollard exec

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Verified via Context7 and official docs |
| Docker SDK | HIGH | bollard 0.20.x actively maintained |
| Terminal | HIGH | @xterm/xterm is current npm package |
| Wayland | MEDIUM | WebKit2GTK-4.1 supports Wayland, testing recommended |
| Architecture | HIGH | Standard Tauri IPC pattern |

## Gaps to Address

- **WebSocket streaming:** For container exec with real-time terminal output, may need to verify bollard's streaming API handles xterm.js fitAddon resizing
- **Wayland testing:** While WebKit2GTK-4.1 supports Wayland, specific display manager configurations may need verification

## Files Created

| File | Purpose |
|------|---------|
| .planning/research/STACK.md | Complete technology stack with versions, rationale, and installation |

## Ready for Roadmap

Research complete. Stack recommendations are current and verified.
