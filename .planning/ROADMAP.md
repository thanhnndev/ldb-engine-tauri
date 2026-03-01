# Roadmap: LDB-Engine

## Overview

A lightweight, GUI-driven local database manager for Linux developers that provides 1-click database instance setup (PostgreSQL, Redis, MySQL, MongoDB) powered by Docker. The journey progresses from core Docker connectivity through instance management to connection utilities and log viewing.

## Phases

- [x] **Phase 1: Docker Hub Integration** - Core foundation for discovering and pulling database images
- [x] **Phase 2: Instance Management** - Full CRUD lifecycle with persistent storage
- [x] **Phase 3: Connection Utilities** - Easy connection string generation and copying
- [x] **Phase 4: Log Viewer** - Real-time container log streaming
- [x] **Phase 5: Fix Delete Volume Bug** - Critical bug fix for PERS-04
- [ ] **Phase 6: Clean Up Orphaned Components** - Remove unused Phase 1 components

## Phase Details

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

---

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Docker Hub Integration | 3/3 | ✓ Complete | 2026-02-28 |
| 2. Instance Management | 3/3 | ✓ Complete | 2026-02-28 |
| 3. Connection Utilities | 2/2 | ✓ Complete | 2026-02-28 |
| 4. Log Viewer | 2/2 | ✓ Complete | 2026-02-28 |
| 5. Fix Delete Volume Bug | 1/1 | ✓ Complete | 2026-03-01 |
| 6. Clean Up Orphaned Components | 0/1 | ⏳ Pending | — |
