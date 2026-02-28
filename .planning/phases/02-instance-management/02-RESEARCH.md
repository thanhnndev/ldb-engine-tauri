# Phase 2: Instance Management - Research

**Researched:** 2026-02-28
**Domain:** Docker container lifecycle management, volume management, port allocation
**Confidence:** HIGH

## Summary

This phase implements database instance management using bollard 0.20 Docker SDK. Users can create, configure, start, stop, restart, and delete database containers with persistent storage. The implementation uses host directory bind mounts for persistence, with automatic port allocation and database-specific environment variable mapping.

**Primary recommendation:** Use bollard's container APIs with HostConfig for port bindings and volume mounts. Store instance metadata in a local JSON file. Use ContainerInspectResponse.state to track running status.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| bollard | 0.20 | Docker API client | Primary async Docker SDK for Rust |
| tokio | 1.x | Async runtime | Required by bollard |
| serde | 1.x | Serialization | Instance metadata storage |

### Supporting
| Library | Purpose | When to Use |
|---------|---------|-------------|
| bollard-stubs | Docker API types | Auto-included with bollard |
| futures | Async streams | Container log streaming |

**Installation:**
```bash
# Already in Cargo.toml
bollard = "0.20"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/
├── commands/
│   ├── mod.rs           # Command exports
│   ├── instances.rs     # Instance CRUD commands
│   └── containers.rs    # Container lifecycle commands
├── docker/
│   ├── mod.rs           # Module exports
│   ├── client.rs        # DockerClient (existing)
│   ├── instances.rs     # Instance management logic
│   └── volume.rs        # Volume helpers
├── models/
│   ├── mod.rs           # Model exports
│   └── instance.rs      # Instance metadata struct
└── state.rs             # App state management

src/lib/
├── types.ts             # Instance types (existing)
├── stores/
│   └── instances.ts     # Svelte stores for instances
└── components/
    └── InstanceCard.svelte
```

### Pattern 1: Instance Metadata Storage
**What:** Store instance configuration in local JSON file
**When to use:** Persisting user-created database instances
**Example:**
```rust
// Instance metadata structure
#[derive(Serialize, Deserialize, Clone)]
pub struct DatabaseInstance {
    pub id: String,           // UUID
    pub name: String,          // User-provided name
    pub db_type: DbType,      // postgres, mysql, redis, mongo
    pub image: String,        // Full image name with tag
    pub port: u16,            // Host port
    pub password: String,     // Root password (encrypted in production)
    pub volume_path: String,  // Host directory for persistence
    pub created_at: DateTime,
}

// Storage location
// Linux: ~/.local/share/ldb-engine/instances.json
// Or app data directory from Tauri
```

### Pattern 2: Container Creation with Environment Variables
**What:** Map user input to Docker container configuration
**When to use:** Creating new database containers
**Example:**
```rust
use bollard::models::{ContainerCreateBody, HostConfig, PortBinding};
use std::collections::HashMap;

fn build_container_config(
    db_type: &DbType,
    password: &str,
    port: u16,
    volume_path: &str,
) -> ContainerCreateBody {
    let mut env = vec![];
    
    match db_type {
        DbType::Postgres => {
            env.push(format!("POSTGRES_PASSWORD={}", password));
            // Optional: POSTGRES_USER, POSTGRES_DB
        }
        DbType::MySQL => {
            env.push(format!("MYSQL_ROOT_PASSWORD={}", password));
            // Optional: MYSQL_DATABASE
        }
        DbType::MongoDB => {
            env.push("MONGO_INITDB_ROOT_USERNAME=root".to_string());
            env.push(format!("MONGO_INITDB_ROOT_PASSWORD={}", password));
        }
        DbType::Redis => {
            // Redis doesn't use env vars for password
            // Password must be set via command in CMD
        }
    }
    
    // Port bindings
    let mut port_bindings = HashMap::new();
    let default_port = get_default_port(db_type);
    port_bindings.insert(
        format!("{}/tcp", default_port),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(port.to_string()),
        }]),
    );
    
    // Volume mounts
    let binds = vec![format!("{}:/data", volume_path)];
    
    ContainerCreateBody {
        image: Some(image_name),
        env: Some(env),
        exposed_ports: Some(vec![format!("{}/tcp", default_port)]),
        host_config: Some(HostConfig {
            port_bindings: Some(port_bindings),
            binds: Some(binds),
            restart_policy: Some(RestartPolicy {
                name: Some("unless-stopped".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}
```

### Pattern 3: Container Status Detection
**What:** Query container state from Docker daemon
**When to use:** Displaying real-time status, polling
**Example:**
```rust
use bollard::container::InspectContainerOptions;

async fn get_instance_status(docker: &Docker, name: &str) -> InstanceStatus {
    let options = InspectContainerOptions::default();
    let info = docker.inspect_container(name, Some(options)).await?;
    
    let state = info.state.unwrap();
    let status = state.status.unwrap(); // "running", "exited", "paused", etc.
    
    match status.as_str() {
        "running" => InstanceStatus::Running,
        "exited" => InstanceStatus::Stopped,
        "restarting" => InstanceStatus::Starting,
        "dead" => InstanceStatus::Error(state.error.unwrap_or_default()),
        _ => InstanceStatus::Unknown,
    }
}
```

### Pattern 4: Port Allocation Strategy
**What:** Find available port and prevent conflicts
**When to use:** When creating new instances
**Example:**
```rust
async fn find_available_port(docker: &Docker, base_port: u16) -> Result<u16> {
    let containers = docker.list_containers(Some(ListContainersOptions {
        all: true,
        ..Default::default()
    })).await?;
    
    let mut occupied: HashSet<u16> = HashMap::new();
    
    for container in containers {
        if let Some(config) = container.host_config {
            if let Some(port_bindings) = config.port_bindings {
                for (_container_port, bindings) in port_bindings {
                    if let Some(bindings) = bindings {
                        for binding in bindings {
                            if let Some(host_port) = &binding.host_port {
                                if let Ok(port) = host_port.parse::<u16>() {
                                    occupied.insert(port);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Find first available port starting from base_port
    for port in base_port..(base_port + 1000) {
        if !occupied.contains(&port) {
            return Ok(port);
        }
    }
    
    Err("No available ports".into())
}
```

### Pattern 5: Volume Directory Management
**What:** Create and manage host directories for persistence
**When to use:** Before container creation
**Example:**
```rust
use std::fs;
use dirs;

fn get_volume_path(instance_name: &str) -> PathBuf {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ldb-engine")
        .join("volumes")
        .join(instance_name);
    
    fs::create_dir_all(&data_dir).ok();
    data_dir
}

fn cleanup_volume(instance_name: &str, delete_data: bool) -> Result<()> {
    let volume_path = get_volume_path(instance_name);
    if delete_data {
        fs::remove_dir_all(&volume_path)?;
    }
    Ok(())
}
```

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Docker connection | Custom HTTP client | bollard | Handles auth, serialization, reconnection |
| Container lifecycle | Raw API calls | bollard container API | Proper error handling, idempotency |
| Port conflict detection | netstat parsing | Docker list_containers API | Accurate, atomic with container state |
| Volume management | Custom file operations | Docker host bind mounts | Standard, portable, cleaner cleanup |
| Instance status polling | setInterval in frontend | Tauri events + backend polling | Better architecture separation |

**Key insight:** Bollard provides async container management that integrates with tokio. Don't reinvent Docker communication.

## Common Pitfalls

### Pitfall 1: Redis Password Not Working
**What goes wrong:** Setting password via environment variable doesn't work for Redis
**Why it happens:** Redis image doesn't support REDIS_PASSWORD env var (only some community images do)
**How to avoid:** Use command in ContainerCreateBody to pass --requirepass
**Warning signs:** Redis connects without password despite setting env var

### Pitfall 2: Volume Data Not Persisting
**What goes wrong:** Data disappears after container restart
**Why it happens:** Using Docker volumes instead of host bind mounts, or wrong mount path
**How to avoid:** Use host bind mounts with correct container path (e.g., /data for PostgreSQL, Redis, MongoDB; /var/lib/mysql for MySQL)
**Warning signs:** Database empty after restart, new container shows fresh install

### Pitfall 3: Port Conflicts on Container Start
**What goes wrong:** Container fails to start with "port is already allocated"
**Why it happens:** Not checking existing port bindings, or stale data in instance storage
**How to avoid:** Query Docker for occupied ports before allocating, validate on each operation
**Warning signs:** Error message about port binding failure

### Pitfall 4: Container Remove Fails
**What goes wrong:** Cannot delete container, "container is running"
**Why it happens:** Trying to remove without stopping first
**How to avoid:** Always stop container before removing, or use force remove
**Warning signs:** Error about container in running state

### Pitfall 5: Instance Status Stuck
**What goes wrong:** UI shows "Running" but container actually stopped
**Why it happens:** Not polling or using cached state without refresh
**How to avoid:** Poll Docker on interval, listen to container events
**Warning signs:** Stale status after external changes

## Code Examples

### Creating a Database Container
```rust
// Source: bollard documentation + Docker Hub official images
use bollard::container::{CreateContainerOptions, StartContainerOptions};
use bollard::models::{ContainerCreateBody, HostConfig, PortBinding, RestartPolicy};
use std::collections::HashMap;

pub async fn create_database_instance(
    docker: &Docker,
    name: &str,
    db_type: DbType,
    image: &str,
    password: &str,
    port: u16,
    volume_path: &str,
) -> Result<String> {
    // Build environment variables
    let env = build_env_vars(db_type, password);
    
    // Build port bindings
    let default_port = db_type.default_port();
    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        format!("{}/tcp", default_port),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(port.to_string()),
        }]),
    );
    
    // Build volume binds
    let binds = vec![format!("{}:{}", volume_path, db_type.data_path())];
    
    // Create container config
    let config = ContainerCreateBody {
        image: Some(image.to_string()),
        env: Some(env),
        exposed![format!("{}/tcp", default_ports: Some(vec_port)]),
        host_config: Some(HostConfig {
            port_bindings: Some(port_bindings),
            binds: Some(binds),
            restart_policy: Some(RestartPolicy {
                name: Some("unless-stopped".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        // For Redis: add cmd with --requirepass
        cmd: if db_type == DbType::Redis {
            Some(vec!["redis-server".to_string(), format!("--requirepass {}", password)])
        } else {
            None
        },
        ..Default::default()
    };
    
    // Create container
    let options = CreateContainerOptions {
        name,
        platform: None,
    };
    
    let response = docker.create_container(Some(options), config).await?;
    Ok(response.id)
}
```

### Starting/Stopping/Restarting
```rust
// Source: bollard container API
pub async fn start_instance(docker: &Docker, name: &str) -> Result<()> {
    docker.start_container(name, None::<StartContainerOptions<String>>).await?;
    Ok(())
}

pub async fn stop_instance(docker: &Docker, name: &str) -> Result<()> {
    use bollard::container::StopContainerOptions;
    let options = StopContainerOptions { t: 10 };
    docker.stop_container(name, Some(options)).await?;
    Ok(())
}

pub async fn restart_instance(docker: &Docker, name: &str) -> Result<()> {
    use bollard::container::RestartContainerOptions;
    let options = RestartContainerOptions { t: 10 };
    docker.restart_container(name, Some(options)).await?;
    Ok(())
}
```

### Removing Container with Volume Option
```rust
// Source: bollard container API
pub async fn delete_instance(
    docker: &Docker, 
    name: &str, 
    delete_volume: bool,
) -> Result<()> {
    // Stop container first if running
    if let Ok(info) = docker.inspect_container(name, None).await {
        if info.state.as_ref().map(|s| s.running).flatten() == Some(true) {
            docker.stop_container(name, None::<StopContainerOptions<String>>).await?;
        }
    }
    
    // Remove container
    use bollard::container::RemoveContainerOptions;
    let options = RemoveContainerOptions {
        force: true,  // Force remove even if running
        ..Default::default()
    };
    docker.remove_container(name, Some(options)).await?;
    
    // Delete volume if requested
    if delete_volume {
        let volume_path = get_volume_path(name);
        std::fs::remove_dir_all(volume_path).ok();
    }
    
    Ok(())
}
```

### Inspecting Container Status
```rust
// Source: bollard container API
use bollard::container::InspectContainerOptions;

#[derive(Debug, Clone, Serialize)]
pub enum InstanceState {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error(String),
    Unknown,
}

pub async fn get_instance_state(docker: &Docker, name: &str) -> Result<InstanceState> {
    let options = InspectContainerOptions::default();
    let info = docker.inspect_container(name, Some(options)).await?;
    
    let state = info.state.ok_or("No state information")?;
    let status = state.status.unwrap_or_default();
    
    match status.as_str() {
        "running" => Ok(InstanceState::Running),
        "exited" => Ok(InstanceState::Stopped),
        "paused" => Ok(InstanceState::Stopped),  // Treat paused as stopped for UI
        "restarting" => Ok(InstanceState::Starting),
        "removing" => Ok(InstanceState::Stopping),
        "dead" => Ok(InstanceState::Error(state.error.unwrap_or_default())),
        _ => Ok(InstanceState::Unknown),
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Docker CLI parsing | bollard async API | bollard 0.14+ | Type-safe, proper async |
| Anonymous volumes | Host bind mounts | Always | Data persists reliably |
| Manual port tracking | Docker API port inspection | Modern approach | Accurate conflict detection |
| Polling with sleep | Event-based + interval | Advanced | Lower latency status updates |

**Deprecated/outdated:**
- `bollard::container::Config` - Use `bollard::models::ContainerCreateBody` instead (renamed in 0.16)
- Direct docker CLI commands - Use bollard API for consistency

## Open Questions

1. **Redis Password Handling**
   - What we know: Redis doesn't support password via environment variable, needs command-line `--requirepass`
   - What's unclear: Whether this approach persists correctly across Redis restarts
   - Recommendation: Test thoroughly; may need custom config file mount

2. **Instance Naming Collision**
   - What we know: Docker container names must be unique
   - What's unclear: How to handle duplicate user-provided names
   - Recommendation: Generate internal unique ID, allow user-friendly display name

3. **Data Migration on Version Upgrade**
   - What we know: PostgreSQL data directory path changed in version 18
   - What's unclear: How to handle user upgrading PostgreSQL version
   - Recommendation: Document that major version upgrades require fresh instance

## Sources

### Primary (HIGH confidence)
- https://docs.rs/bollard/latest/bollard/ - Container APIs, volume APIs, HostConfig
- https://hub.docker.com/_/postgres - PostgreSQL environment variables, volume paths
- https://hub.docker.com/_/mysql - MySQL environment variables, volume paths
- https://hub.docker.com/_/mongo - MongoDB environment variables
- https://hub.docker.com/_/redis - Redis configuration

### Secondary (MEDIUM confidence)
- https://docs.docker.com/engine/api/v1.49/ - Docker Engine API specification
- https://docs.docker.com/compose/environment-variables/ - Environment variable precedence

### Tertiary (LOW confidence)
- Community discussions on Redis password via Docker - needs validation

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH - bollard 0.20 is current stable version
- Architecture: HIGH - Patterns based on bollard API and Docker best practices
- Database Env Vars: HIGH - Verified against official Docker Hub images
- Volume Paths: HIGH - Verified against official Docker Hub documentation
- Pitfalls: MEDIUM - Based on community issues, some need validation

**Research date:** 2026-02-28
**Valid until:** 2026-05-28 (bollard releases ~quarterly, but API stable)
