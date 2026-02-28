# Technology Stack: LDB-Engine

**Project:** LDB-Engine (Linux Database Engine)
**Researched:** February 27, 2026
**Confidence:** HIGH

## Overview

This document recommends the technology stack for building a Tauri v2 desktop application with Docker integration. The stack prioritizes: active maintenance, Rust ecosystem maturity, Wayland/Linux compatibility, and minimal dependencies.

---

## Recommended Stack

### Core Desktop Framework

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **Tauri** | 2.10.x | Desktop framework | Native WebView (WebKit2GTK-4.1 on Linux), 95% smaller binaries than Electron, Rust backend |
| **Svelte** | 5.x (latest) | Frontend framework | Reactive runes, minimal JS bundle, excellent Tauri integration |
| **SvelteKit** | 2.x | App framework | Use with `@sveltejs/adapter-static` for SPA mode |
| **Tailwind CSS** | 4.x | Styling | Zero-config with Vite, atomic CSS, excellent shadcn-svelte support |

**Installation:**

```bash
# Create Tauri app with Svelte template
npm create tauri-app@latest ldb-engine -- --template sveltekit --manager npm

# Install Tailwind CSS v4
npx svelte-add@latest tailwindcss

# Install shadcn-svelte
npx shadcn-svelte@latest init
```

### Docker Integration (Rust Backend)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **bollard** | 0.20.x | Docker API client | Only actively maintained Rust Docker SDK; supports Docker API 1.52; async with Tokio |
| **tokio** | 1.x | Async runtime | Required by bollard; de facto standard (async-std discontinued) |
| **serde** | 1.x | Serialization | Built-in bollard support; JSON for SQLite storage |

**Cargo.toml:**

```toml
[dependencies]
bollard = "0.20"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }

# Optional: for container exec/terminal
tokio-util = "0.7"
```

**NOTE:** Do NOT use `shiplift` (abandoned since 2021) or `docker-crates` (less maintained).

### Terminal Emulation (Frontend)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **@xterm/xterm** | 5.5.x or 6.0.x | Terminal emulator | Used by VS Code, Hyper; actively maintained; zero dependencies |
| **@xterm/addon-fit** | Match xterm | Auto-resize | Essential for responsive terminal |
| **@xterm/addon-web-links** | Match xterm | Clickable links | UX improvement |

**Installation:**

```bash
# CORRECT - Use @xterm namespace (NOT deprecated 'xterm' package)
npm install @xterm/xterm @xterm/addon-fit @xterm/addon-web-links
```

**What NOT to use:**

- ❌ `xterm` (deprecated npm package, renamed to `@xterm/xterm`)
- ❌ `xtermjs` (incorrect package name)
- ⚠️ `node-pty` (requires native compilation, complex for Tauri; use bollard exec API instead)

**Alternative:** Consider **Ghostty Web** (WASM-based, xterm.js API compatible) for future-proofing, but xterm.js is currently more stable.

### UI Components

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **shadcn-svelte** | latest | UI component library | Accessible, customizable, excellent Svelte 5 support |
| **lucide-svelte** | latest | Icons | Default with shadcn-svelte |
| **tailwind-variants** | latest | Component variants | Required by shadcn-svelte |

### Tauri Plugins

| Plugin | Version | Purpose |
|--------|---------|---------|
| **tauri-plugin-shell** | 2.x | Execute commands in containers |
| **tauri-plugin-store** | 0.10.x | Persistent key-value storage |
| **tauri-plugin-svelte** | 2.x | Svelte stores persistence |

---

## Platform-Specific Configuration

### Linux (Wayland/X11)

Tauri v2 uses **WebKit2GTK-4.1** which requires:

```bash
# Arch/Manjaro
sudo pacman -S webkit2gtk-4.1

# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel
```

**Wayland Compatibility:** WebKit2GTK-4.1 works on Wayland via GTK's Wayland backend. No special configuration needed for most distros.

### SQLite (Local Storage)

```bash
# Rust dependency
cargo add rusqlite --features bundled
```

---

## Version Compatibility Matrix

| Component | Minimum | Recommended | Notes |
|-----------|---------|------------|-------|
| Tauri | 2.0 | 2.10.x | WebKit2GTK-4.1 requirement since alpha.3 |
| Svelte | 5.0 | 5.x | Use runes ($state, $derived) |
| SvelteKit | 2.0 | 2.20.x | Use adapter-static |
| Tailwind CSS | 4.0 | 4.x | Requires @tailwindcss/vite |
| bollard | 0.19 | 0.20.x | Docker API 1.52 |
| Tokio | 1.0 | 1.x | Required by bollard |
| Node.js | 18+ | 20.x LTS | For build tooling |

---

## Architecture Pattern

### Recommended: IPC Command Pattern

```
┌─────────────────────────────────────────────────────┐
│                   Svelte Frontend                    │
│  (UI Components + xterm.js terminal)               │
└──────────────────┬──────────────────────────────────┘
                   │ Tauri invoke()
                   ▼
┌─────────────────────────────────────────────────────┐
│              Rust Backend (Tauri)                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │   Commands  │  │   bollard   │  │   SQLite    │  │
│  │  (Docker)   │  │   (Docker)  │  │  (Storage)  │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────┘
```

**Example Command:**

```rust
// src-tauri/src/lib.rs
use bollard::Docker;
use tauri::command;

#[command]
async fn list_containers() -> Result<Vec<Container>, String> {
    let docker = Docker::connect_with_socket_defaults()
        .map_err(|e| e.to_string())?;
    
    let containers = docker.list_containers(None)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(containers)
}
```

---

## What NOT to Use and Why

| Technology | Why Avoid | Alternative |
|------------|-----------|-------------|
| **shiplift** | Abandoned since 2021 | bollard |
| **docker-crates** | Less maintained, fewer features | bollard |
| **xterm** (npm) | Deprecated, renamed | @xterm/xterm |
| **electron** | 95% larger binaries, heavier | Tauri v2 |
| **async-std** | Discontinued/inactive | tokio |
| **React/Vue** | Heavier than Svelte | Svelte 5 |
| **Tailwind v3** | v4 is current with better performance | Tailwind v4 |

---

## Dependencies for Package.json

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "@tauri-apps/plugin-store": "^2.0.0",
    "@xterm/xterm": "^5.5.0",
    "@xterm/addon-fit": "^0.10.0",
    "@xterm/addon-web-links": "^0.11.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.5.0"
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.0",
    "@tailwindcss/vite": "^4.0.0",
    "tailwindcss": "^4.0.0",
    "vite": "^6.0.0"
  }
}
```

---

## Dependencies for Cargo.toml

```toml
[dependencies]
tauri = { version = "2", features = ["devtools"] }
tauri-plugin-shell = "2"
tauri-plugin-store = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Docker
bollard = "0.20"
tokio = { version = "1", features = ["full"] }

# Local Storage
rusqlite = { version = "0.32", features = ["bundled"] }

[build-dependencies]
tauri-build = "2"
```

---

## Sources

- **bollard:** https://docs.rs/bollard/latest/bollard/ (Verified: HIGH)
- **Tauri v2:** https://v2.tauri.app/ (Verified: HIGH)
- **@xterm/xterm:** https://www.npmjs.com/package/@xterm/xterm (Verified: HIGH)
- **shadcn-svelte:** https://www.shadcn-svelte.com/docs/installation/manual (Verified: HIGH)
- **Tailwind v4:** https://tailwindcss.com/docs/upgrade-guide (Verified: HIGH)
- **Tokio vs async-std:** https://infobytes.guru/articles/rust-async-runtime-comparison.html (Verified: MEDIUM)
- **WebKit2GTK-4.1:** https://v2.tauri.app/ja/blog/tauri-2-0-0-alpha-3 (Verified: HIGH)

---

## Confidence Assessment

| Area | Confidence | Reason |
|------|------------|--------|
| Core Framework | HIGH | Official Tauri v2 docs, verified current |
| Docker SDK | HIGH | bollard actively maintained, version 0.20.x |
| Terminal | HIGH | @xterm/xterm official npm, verified package |
| UI Stack | HIGH | shadcn-svelte docs, Tailwind v4 released |
| Wayland | MEDIUM | WebKit2GTK-4.1 supports Wayland, but testing recommended |
| Async Runtime | HIGH | Tokio is de facto standard, async-std discontinued |
