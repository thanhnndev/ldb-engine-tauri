# Milestone v1.0: MVP

**Status:** ✅ SHIPPED 2026-03-01
**Phases:** 1-6
**Total Plans:** 12

## Overview

A lightweight, GUI-driven local database manager for Linux developers that provides 1-click database instance setup (PostgreSQL, Redis, MySQL, MongoDB) powered by Docker. The journey progresses from core Docker connectivity through instance management to connection utilities and log viewing.

## Phases

### Phase 1: Docker Hub Integration

**Goal**: Users can discover official database images from Docker Hub and pull them to local system
**Depends on**: Nothing (first phase)
**Requirements**: DOCK-01, DOCK-02, DOCK-03, DOCK-04, DOCK-05, DOCK-06
**Success Criteria** (what must be TRUE):
  1. User can view available tags for PostgreSQL from Docker Hub
  2. User can view available tags for Redis from Docker Hub
  3. User can view available tags for MySQL from Docker Hub
  4. User can view available tags for MongoDB from Docker Hub
  5. User can select a specific image version (e.g., postgres:16-alpine)
  6. User can see real-time download progress when pulling images

**Plans**: 3 plans in 3 waves
- [x] 01-01-PLAN.md — Dependencies Setup
- [x] 01-02-PLAN.md — Docker Hub API + Pull Backend
- [x] 01-03-PLAN.md — Frontend UI Components

**Details:**
- Added bollard 0.20, tokio 1.x, reqwest 0.12, futures 0.3
- Created docker module structure with hub.rs and client.rs
- Created ImageCard, TagList, PullProgress components (later removed as orphaned)

---

### Phase 2: Instance Management

**Goal**: Users can create, configure, start, stop, restart, and delete database instances with persistent storage
**Depends on**: Phase 1
**Requirements**: INST-01, INST-02, INST-03, INST-04, INST-05, INST-06, LIFE-01, LIFE-02, LIFE-03, LIFE-04, LIFE-05, LIFE-06, PERS-01, PERS-02, PERS-03, PERS-04
**Success Criteria** (what must be TRUE):
  1. User can create a new database instance with a custom name
  2. User can select database type (PostgreSQL, Redis, MySQL, MongoDB)
  3. User can select image version from available tags
  4. User can set root password (required for PostgreSQL/MySQL, optional for Redis)
  5. System auto-detects occupied ports and suggests next available
  6. System correctly maps password to environment variables for each database type
  7. User can start a database instance
  8. User can stop a database instance
  9. User can restart a database instance
  10. User can delete a database instance
  11. System displays current state (Running, Stopped, Error) in real-time
  12. System creates local directories for volume mounts automatically
  13. Data persists across container restarts
  14. Upon deletion, system prompts with checkbox to delete associated volume data
  15. User can choose to delete or retain volume data on instance removal

**Plans**: 3 plans in 3 waves
- [x] 02-01-PLAN.md — Instance model + lifecycle commands
- [x] 02-02-PLAN.md — State persistence + port detection + volumes
- [x] 02-03-PLAN.md — Frontend UI for instance management

**Details:**
- Implemented Instance model with uuid v4 IDs
- Used chrono for timestamp tracking
- Container naming prefix "ldb-" to identify our containers
- Used dirs crate for cross-platform home directory resolution
- Base ports: PostgreSQL=5432, Redis=6379, MySQL=3306, MongoDB=27017
- Volume cleanup defaults to false (preserve data on delete)
- Used Svelte 5 runes for frontend reactivity
- 5-second polling interval for status updates

---

### Phase 3: Connection Utilities

**Goal**: Users can easily obtain connection strings to connect to their running databases
**Depends on**: Phase 2
**Requirements**: CONN-01, CONN-02, CONN-03
**Success Criteria** (what must be TRUE):
  1. System generates standard connection string for running instance
  2. User can copy connection string to clipboard with one click
  3. Connection string follows standard format (e.g., postgresql://user:password@127.0.0.1:5432/db)

**Plans**: 2 plans in 2 waves
- [x] 03-01-PLAN.md — Backend connection string command
- [x] 03-02-PLAN.md — Frontend component + clipboard integration

**Details:**
- Standard connection string formats for PostgreSQL, Redis, MySQL, MongoDB
- Instance name transformed to lowercase with underscores for database name
- MongoDB includes authSource=admin query parameter
- Show connection string only for running instances
- Used @tauri-apps/plugin-clipboard-manager for clipboard operations
- 2-second feedback reset on copy button

---

### Phase 4: Log Viewer

**Goal**: Users can view real-time logs from their database containers
**Depends on**: Phase 3
**Requirements**: LOGS-01, LOGS-02, LOGS-03, LOGS-04
**Success Criteria** (what must be TRUE):
  1. User can view logs from selected container in embedded terminal view
  2. System streams stdout from container in real-time
  3. System streams stderr from container in real-time
  4. User can watch logs update as container produces output

**Plans**: 2 plans in 2 waves
- [x] 04-01-PLAN.md — Backend log streaming command
- [x] 04-02-PLAN.md — Frontend LogViewer + integration

**Details:**
- LogEvent enum with StdOut, StdErr, Error, Eof variants for structured streaming
- Used bollard::container::LogsOptions struct for bollard 0.16 API
- LogEvent interface typed with nested data.message structure
- 5000 line log limit to prevent memory growth
- Modal overlay pattern for log viewing

---

### Phase 5: Fix Delete Volume Bug

**Goal**: Fix critical bug where deleteVolume parameter is ignored when deleting instances
**Depends on**: Phase 4
**Gap Closure**: Closes GAP-01 from v1-MILESTONE-AUDIT.md
**Success Criteria** (what must be TRUE):
  1. InstanceCard.svelte passes deleteVolume parameter correctly to deleteInstance
  2. Users who check "Delete volume data" will have their volume deleted
  3. "Delete Instance with Volume" E2E flow completes successfully

**Plans**: 1 plan in 1 wave
- [x] 05-01-PLAN.md — Fix deleteVolume parameter propagation

**Details:**
- Fixed GAP-01 - deleteVolume parameter now passed from InstanceControls through InstanceList to deleteInstance function

---

### Phase 6: Clean Up Orphaned Components

**Goal**: Remove unused components from Phase 1 to reduce codebase confusion
**Depends on**: Phase 5
**Gap Closure**: Tech debt cleanup from v1-MILESTONE-AUDIT.md
**Success Criteria** (what must be TRUE):
  1. ImageCard.svelte removed (or integrated if still needed)
  2. TagList.svelte removed (or integrated if still needed)
  3. PullProgress.svelte removed (or integrated if still needed)

**Plans**: 1 plan in 1 wave
- [x] 06-01-PLAN.md — Remove orphaned components

**Details:**
- Deleted 3 orphaned components (ImageCard, TagList, PullProgress) from Phase 1

---

## Milestone Summary

**Key Decisions:**

- Tauri v2 chosen for modern, lightweight, Rust-native approach
- Shadcn/ui chosen for developer-focused aesthetic and accessibility
- SQLite chosen for state (simple, Rust-native, sufficient for metadata)
- Docker module separation: hub for Docker Hub API, client for local Docker daemon
- Container naming prefix "ldb-" to identify our containers
- Svelte 5 runes for frontend reactivity
- Modal overlay pattern for log viewing
- bollard 0.20 for Docker API integration

**Issues Resolved:**

- GAP-01: Delete Volume Option Ignored - Fixed deleteVolume parameter propagation
- Tech Debt: Orphaned Components - Removed unused ImageCard, TagList, PullProgress components
- Context overflow prevention - Created milestone archive system

**Issues Deferred:**

- None - all identified issues resolved in this milestone

**Technical Debt Incurred:**

- None - Phase 6 cleaned up all identified tech debt

---

_For current project status, see .planning/ROADMAP.md_

---

## Milestone Stats

- **Phases:** 6
- **Plans:** 12
- **Files changed:** 53
- **Lines added:** ~15,846
- **Timeline:** 2026-02-27 to 2026-03-01 (~2 days)

---

*Archived: 2026-03-01 as part of v1.0 milestone completion*
