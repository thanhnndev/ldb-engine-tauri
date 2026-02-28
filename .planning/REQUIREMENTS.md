# Requirements: LDB-Engine

**Defined:** 2026-02-28
**Core Value:** Enable developers to spin up local database instances (PostgreSQL, Redis, MySQL, MongoDB) with a single click, without manual Docker configuration.

## v1 Requirements

### Docker Hub Integration

- [x] **DOCK-01**: System fetches and displays official repository tags from Docker Hub for PostgreSQL
- [x] **DOCK-02**: System fetches and displays official repository tags from Docker Hub for Redis
- [x] **DOCK-03**: System fetches and displays official repository tags from Docker Hub for MySQL
- [x] **DOCK-04**: System fetches and displays official repository tags from Docker Hub for MongoDB
- [x] **DOCK-05**: User can select specific image version (e.g., `postgres:16-alpine`, `redis:7.2`)
- [x] **DOCK-06**: System displays real-time download/pull progress when fetching images

### Instance Creation

- [x] **INST-01**: User can create new database instance with Instance Name
- [x] **INST-02**: User can select Database Type (PostgreSQL, Redis, MySQL, MongoDB)
- [x] **INST-03**: User can select Image Version from available tags
- [x] **INST-04**: User can set Root Password (required for PostgreSQL, MySQL; optional for Redis)
- [x] **INST-05**: System automatically detects occupied host ports and suggests next available
- [x] **INST-06**: System maps password to correct Docker environment variables (POSTGRES_PASSWORD, MYSQL_ROOT_PASSWORD, Redis --requirepass)

### Instance Lifecycle

- [x] **LIFE-01**: User can Start a database instance
- [x] **LIFE-02**: User can Stop a database instance
- [x] **LIFE-03**: User can Restart a database instance
- [x] **LIFE-04**: User can Delete a database instance
- [x] **LIFE-05**: System displays current execution state (Running, Stopped, Error)
- [x] **LIFE-06**: System polls Docker daemon for real-time status updates

### Persistent Data

- [x] **PERS-01**: System automatically creates local host directories for container volume mounts
- [x] **PERS-02**: System maps volumes to ensure data persists across container restarts
- [x] **PERS-03**: Upon instance deletion, system prompts with checkbox "Delete associated volume data?"
- [x] **PERS-04**: User can choose to delete or retain volume data on instance removal

### Connection Utilities

- [ ] **CONN-01**: System generates standard connection string for running instance
- [ ] **CONN-02**: User can copy connection string with one click
- [ ] **CONN-03**: Connection string uses format: `postgresql://user:password@127.0.0.1:5432/db`

### Log Viewer

- [ ] **LOGS-01**: System includes embedded terminal view for logs
- [ ] **LOGS-02**: System streams stdout from selected container
- [ ] **LOGS-03**: System streams stderr from selected container
- [ ] **LOGS-04**: User can view logs in real-time

## v2 Requirements

(None yet — to be defined after v1)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Built-in SQL query editor | Beyond core value; users can use CLI tools |
| Multi-node clustering | Single instance only, local development focus |
| Cloud deployment | Linux local only |
| User authentication | Single-user local app |
| Remote management | Local-first design |
| Team collaboration features | Out of scope for v1 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| DOCK-01 | Phase 1 | Complete |
| DOCK-02 | Phase 1 | Complete |
| DOCK-03 | Phase 1 | Complete |
| DOCK-04 | Phase 1 | Complete |
| DOCK-05 | Phase 1 | Complete |
| DOCK-06 | Phase 1 | Complete |
| INST-01 | Phase 2 | Complete |
| INST-02 | Phase 2 | Complete |
| INST-03 | Phase 2 | Complete |
| INST-04 | Phase 2 | Complete |
| INST-05 | Phase 2 | Complete |
| INST-06 | Phase 2 | Complete |
| LIFE-01 | Phase 2 | Complete |
| LIFE-02 | Phase 2 | Complete |
| LIFE-03 | Phase 2 | Complete |
| LIFE-04 | Phase 2 | Complete |
| LIFE-05 | Phase 2 | Complete |
| LIFE-06 | Phase 2 | Complete |
| PERS-01 | Phase 2 | Complete |
| PERS-02 | Phase 2 | Complete |
| PERS-03 | Phase 2 | Complete |
| PERS-04 | Phase 2 | Complete |
| CONN-01 | Phase 3 | Pending |
| CONN-02 | Phase 3 | Pending |
| CONN-03 | Phase 3 | Pending |
| LOGS-01 | Phase 4 | Pending |
| LOGS-02 | Phase 4 | Pending |
| LOGS-03 | Phase 4 | Pending |
| LOGS-04 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 28 total
- Mapped to phases: 28
- Unmapped: 0 ✓

---
*Requirements defined: 2026-02-28*
*Last updated: 2026-02-28 after initial definition*
