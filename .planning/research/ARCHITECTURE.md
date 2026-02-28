# Architecture Patterns: LDB-Engine

**Project:** LDB-Engine (Linux Database Engine)
**Researched:** 2026-02-27
**Domain:** Desktop application with Docker integration
**Confidence:** HIGH (based on official Tauri v2 documentation and verified sources)

---

## Recommended Architecture

LDB-Engine follows a **layered command pattern** typical of Tauri v2 applications, with clear component boundaries separating UI concerns from Docker operations and state persistence.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        FRONTEND (Svelte)                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐ │
│  │ InstanceList│  │ CreateForm │  │ LogViewer   │  │ Settings   │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └─────┬──────┘ │
└─────────┼────────────────┼────────────────┼───────────────┼─────────┘
          │                │                │               │
          └────────────────┴────────┬───────┴───────────────┘
                                     │ invoke()
                                     ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    TAURI IPC LAYER (Commands)                      │
│         ┌──────────────────────────────────────────────┐          │
│         │  Command Layer (src-tauri/src/commands/)    │          │
│         │  - instance_commands.rs                      │          │
│         │  - docker_commands.rs                        │          │
│         │  - system_commands.rs                        │          │
│         └──────────────────────┬───────────────────────┘          │
└─────────────────────────────────┼───────────────────────────────────┘
                                  │
┌─────────────────────────────────┼───────────────────────────────────┐
│                    BUSINESS LOGIC LAYER                            │
│  ┌─────────────────┐  ┌─────────┴─────────┐  ┌─────────────────┐   │
│  │ Docker Service  │  │ Instance Service  │  │ Config Service  │   │
│  │ (bollard)       │  │ (lifecycle mgr)   │  │ (app settings)  │   │
│  └────────┬────────┘  └─────────┬─────────┘  └────────┬────────┘   │
└───────────┼─────────────────────┼─────────────────────┼─────────────┘
            │                     │                     │
            ▼                     ▼                     ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    DATA LAYER                                       │
│  ┌─────────────────────┐              ┌────────────────────────┐   │
│  │ Docker Engine API   │              │ SQLite (rusqlite)      │   │
│  │ /var/run/docker.sock│              │ Instance metadata      │   │
│  └─────────────────────┘              │ App configuration      │   │
│                                        └────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Component Boundaries

### 1. Frontend Layer (Svelte)

**Responsibility:** User interface, form handling, real-time display updates

**Boundaries:**
- Cannot access Docker socket directly
- Communicates ONLY via Tauri `invoke()` commands
- Receives real-time updates via Tauri events
- No direct database access (all data flows through Rust backend)

**Key Components:**
| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `InstanceList.svelte` | Display all database instances, status | `invoke('list_instances')` |
| `CreateInstance.svelte` | Form for new instance creation | `invoke('create_instance')` |
| `LogViewer.svelte` | Real-time container log display | `invoke('get_logs')`, events |
| `ConnectionString.svelte` | Generate/copy connection strings | `invoke('get_connection_string')` |

### 2. Command Layer (Rust)

**Responsibility:** IPC bridge between frontend and business logic, input validation

**Boundaries:**
- Validates all input from frontend before passing to services
- Returns serialized JSON responses
- Handles errors and converts to frontend-friendly messages
- Commands are thin wrappers - business logic lives in services

**File Structure:**
```
src-tauri/src/
├── commands/
│   ├── mod.rs           # Command exports
│   ├── instance.rs      # CRUD operations for instances
│   ├── docker.rs        # Docker-specific operations
│   └── system.rs        # App configuration, port scanning
├── lib.rs               # Command registration
```

**Command Pattern Example:**
```rust
// commands/instance.rs
#[tauri::command]
pub async fn create_instance(
    state: State<'_, AppState>,
    config: InstanceConfig
) -> Result<Instance, String> {
    // 1. Validate input
    validate_config(&config)?;
    
    // 2. Delegate to service
    let instance = state.instance_service.create(config).await?;
    
    // 3. Return result
    Ok(instance)
}
```

### 3. Service Layer (Rust)

**Responsibility:** Business logic, Docker API interaction, state management

**Boundaries:**
- Services own business logic
- Services interact with external systems (Docker, SQLite)
- Services are injected into command layer via State
- Async operations use tokio runtime

**Key Services:**
| Service | Responsibility | External Dependencies |
|---------|---------------|----------------------|
| `DockerService` | Container lifecycle, image management | bollard + Docker socket |
| `InstanceService` | Instance CRUD, state tracking | SQLite + DockerService |
| `ConfigService` | App settings, port management | SQLite |

### 4. Data Layer

**Docker Engine API:**
- Connection: Unix socket `/var/run/docker.sock` (Linux)
- Client: `bollard` crate (async, Tokio-based)
- Operations: pull images, create/start/stop containers, stream logs

**SQLite (Local State):**
- Location: `$APPCONFIG/ldb-engine.db`
- Tables: instances, settings, connection_history
- Access: rusqlite with synchronous queries (sufficient for app state)

---

## Data Flow

### Flow 1: Create Database Instance

```
User fills form
       │
       ▼
Svelte: onSubmit() ──invoke('create_instance', config)──►
       │
       ▼
Tauri IPC: routes to commands::create_instance()
       │
       ▼
Command: validate_config(config)?
       │
       ├─── Invalid ──► Return Error
       │
       ▼ Valid
Service: InstanceService.create(config)
       │
       ├─── 1. Check port availability
       ├─── 2. Pull Docker image (if needed)
       ├─── 3. Create container with volume mounts
       ├─── 4. Start container
       └─── 5. Save metadata to SQLite
       │
       ▼
Response: { id, name, status: "running", port }
       │
       ▼
Svelte: Update instance list, show success toast
```

### Flow 2: Real-time Log Streaming

```
User clicks "View Logs"
       │
       ▼
Svelte: subscribe to event 'container-log-${id}'
       │
       ▼
Tauri: Command returns stream handle
       │
       ▼
DockerService: ContainerLogs { follow: true, stdout: true, stderr: true }
       │
       ▼
Stream ──emit('container-log-${id}', line)──► Frontend event listener
       │
       ▼
Svelte: xterm.js writes to terminal
```

### Flow 3: Instance Lifecycle (Start/Stop/Restart)

```
User clicks "Stop"
       │
       ▼
invoke('stop_instance', { id })
       │
       ▼
Command: State lookup, call DockerService.stop_container(id)
       │
       ▼
DockerService: docker.stop_container(id)
       │
       ▼
Docker API: POST /containers/{id}/stop
       │
       ▼
Update SQLite status, emit event
       │
       ▼
Frontend: Update UI to show "Stopped" status
```

---

## Build Order and Dependencies

### Phase 1: Foundation (Weeks 1-2)

**Goal:** Establish minimal working Tauri app with Docker connectivity

**Build Sequence:**

1. **Setup Tauri v2 + Svelte project**
   ```
   npm create tauri-app@latest
   # Select: Svelte, TypeScript, Tailwind
   ```

2. **Add Rust dependencies**
   ```toml
   # Cargo.toml
   [dependencies]
   tauri = { version = "2", features = [] }
   bollard = "0.19"          # Docker API client
   tokio = { version = "1", features = ["full"] }
   rusqlite = { version = "0.32", features = ["bundled"] }
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   ```

3. **Verify Docker connectivity**
   - Create minimal `DockerService` that connects to socket
   - Test `docker ping` via bollard

4. **Setup logging**
   - Add `tracing` crate for Rust logging
   - Configure frontend logging

### Phase 2: Core Docker Operations (Weeks 3-4)

**Goal:** Implement Docker Hub integration and image management

**Build Sequence:**

1. **Docker Service - Image Operations**
   - Implement `list_images()`
   - Implement `pull_image(repo, tag)` with progress events
   - Implement `get_image_manifest()`

2. **Command Layer**
   - `commands/docker.rs`: image listing, pull progress
   - Add Tauri event emission for pull progress

3. **Frontend**
   - Image browser UI
   - Pull progress indicator

### Phase 3: Instance Management (Weeks 5-6)

**Goal:** CRUD operations for database instances

**Build Sequence:**

1. **SQLite Schema**
   ```sql
   CREATE TABLE instances (
     id TEXT PRIMARY KEY,
     name TEXT NOT NULL,
     db_type TEXT NOT NULL,      -- postgres, mysql, redis, mongodb
     version TEXT NOT NULL,      -- e.g., "16-alpine"
     port INTEGER NOT NULL,
     container_id TEXT,
     volume_path TEXT,
     status TEXT DEFAULT 'stopped',  -- running, stopped, error
     created_at TEXT NOT NULL,
     config_json TEXT            -- serialized instance config
   );
   ```

2. **Instance Service**
   - `create_instance(config)` - creates container + saves metadata
   - `list_instances()` - returns all from SQLite
   - `get_instance(id)` - returns single instance
   - `delete_instance(id)` - stops container, removes volume (optional)

3. **Docker Service - Container Operations**
   - `create_container(config)`
   - `start_container(id)`
   - `stop_container(id)`
   - `remove_container(id)`
   - `get_container_status(id)`

4. **Frontend**
   - Instance list view
   - Create instance form with validation
   - Start/Stop/Delete buttons

### Phase 4: Connection Utilities (Week 7)

**Goal:** Generate connection strings and connection info

**Build Sequence:**

1. **Connection String Generation**
   - Build connection strings per database type
   - PostgreSQL: `postgresql://user:pass@host:port/db`
   - MySQL: `mysql://user:pass@host:port/db`
   - Redis: `redis://:pass@host:port`
   - MongoDB: `mongodb://user:pass@host:port/db`

2. **Frontend**
   - Copy-to-clipboard button
   - Connection details panel

### Phase 5: Log Viewer (Week 8)

**Goal:** Real-time container log streaming

**Build Sequence:**

1. **Log Streaming**
   - Implement `stream_logs(container_id)` returning `Receiver<String>`
   - Use Tauri events to push log lines to frontend
   - Handle stdout and stderr separately

2. **Frontend**
   - Integrate xterm.js
   - Handle terminal resize events
   - Implement scrollback buffer

---

## Critical Design Decisions

### Decision 1: Service-Oriented Backend

**Choice:** Create dedicated service modules instead of putting logic in commands

**Rationale:**
- Commands remain thin and focused on validation + dispatch
- Services are reusable and testable
- Easier to add new functionality without bloating command handlers

### Decision 2: SQLite as Source of Truth

**Choice:** SQLite stores instance metadata, Docker API provides runtime status

**Rationale:**
- Fast startup - app doesn't need to poll Docker on launch
- Offline-capable - can show last known state
- Simple - no external database needed

**Sync Strategy:**
```rust
// On app start: reconcile SQLite state with Docker
for instance in instances {
    let docker_status = docker.inspect_container(&instance.container_id);
    if docker_status.missing {
        // Container was deleted outside app
        update_status(&instance.id, Status::Error);
    }
}
```

### Decision 3: Async Command Handlers

**Choice:** All Tauri commands are async (using tokio)

**Rationale:**
- Docker API calls are inherently async
- Prevents blocking the event loop
- Better resource utilization

### Decision 4: Event-Driven Updates

**Choice:** Frontend subscribes to events for real-time updates

**Rationale:**
- More efficient than polling
- Works for log streaming
- Status changes propagate automatically

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Blocking Commands

**What:** Using synchronous code in Tauri commands

**Why Bad:** Blocks the frontend UI, app becomes unresponsive

**Instead:** Use `async fn` with tokio, offload blocking work to spawn_blocking

```rust
// BAD
#[tauri::command]
fn slow_operation() -> Result<String, String> {
    std::thread::sleep(std::time::Duration::from_secs(5));  // Blocks!
    Ok("done")
}

// GOOD
#[tauri::command]
async fn slow_operation() -> Result<String, String> {
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    Ok("done")
}
```

### Anti-Pattern 2: Frontend Direct Database Access

**What:** Using tauri-plugin-sql to let frontend directly query instance data

**Why Bad:** Bypasses business logic validation, security concerns

**Instead:** Frontend calls commands, backend manages all data access

### Anti-Pattern 3: Global Mutable State

**What:** Using global static variables for services

**Why Bad:** Race conditions, hard to test, lifetime issues

**Instead:** Use Tauri State management with proper lifetimes

```rust
// GOOD - Using Tauri State
struct AppState {
    docker: DockerService,
    instances: InstanceService,
}

Builder::default()
    .manage(AppState { docker, instances })
    .invoke_handler(...)
```

### Anti-Pattern 4: Large Payloads in Commands

**What:** Passing entire container configs or large JSON blobs

**Why Bad:** IPC overhead, potential serialization issues

**Instead:** Use IDs for lookups, stream large data via events

---

## Scalability Considerations

| Concern | Current Approach | At Scale (100+ instances) |
|---------|-----------------|--------------------------|
| Startup time | Load all instances from SQLite | Paginate, lazy load |
| Status polling | On-demand via commands | Background task with throttling |
| Log streaming | One stream per container | Multiplex or limit concurrent streams |
| Memory | Single Docker client | Connection pooling |

---

## File Structure Recommendation

```
ldb-engine-tauri/
├── src/                          # Svelte frontend
│   ├── lib/
│   │   ├── components/
│   │   │   ├── InstanceList.svelte
│   │   │   ├── CreateInstance.svelte
│   │   │   ├── InstanceCard.svelte
│   │   │   ├── LogViewer.svelte
│   │   │   └── ConnectionString.svelte
│   │   ├── stores/
│   │   │   └── instances.ts      # Svelte stores for state
│   │   └── api/
│   │       └── tauri.ts          # Wrapper around invoke()
│   ├── routes/
│   │   └── +page.svelte          # Main page
│   └── app.html
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs                # App setup, command registration
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── instance.rs
│   │   │   ├── docker.rs
│   │   │   └── system.rs
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── docker.rs         # DockerService
│   │   │   ├── instance.rs       # InstanceService
│   │   │   └── config.rs         # ConfigService
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── instance.rs       # Instance struct
│   │   │   └── config.rs         # Config types
│   │   └── db/
│   │       ├── mod.rs
│   │       └── schema.rs         # SQLite schema & migrations
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

---

## Summary

| Aspect | Recommendation |
|--------|----------------|
| Architecture | Layered command pattern with services |
| State | SQLite (rusqlite) for metadata, Docker API for runtime |
| Docker Client | bollard crate (async) |
| IPC | Tauri commands + events |
| Build Order | Foundation → Docker ops → Instance CRUD → Utilities → Logs |

**Key Insight:** The architecture treats the Tauri backend as the "source of truth" coordinator, with SQLite persisting what should be remembered and Docker providing ephemeral runtime state. This separation allows the app to start fast and work offline while still leveraging Docker's full capabilities.
