use bollard::container::{Config, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions};
use bollard::Docker;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::commands::ports::{get_next_port_for_type, get_occupied_ports};
use crate::models::instance::{CreateInstanceRequest, DatabaseType, Instance, InstanceStatus};
use crate::state::StateManager;

/// Get the default volume mount path for a database type
fn get_volume_path(database_type: &DatabaseType) -> &str {
    match database_type {
        DatabaseType::PostgreSQL => "/data",
        DatabaseType::MySQL => "/var/lib/mysql",
        DatabaseType::MongoDB => "/data",
        DatabaseType::Redis => "/data",
    }
}

/// Get environment variables for a database type
fn get_env_vars(database_type: &DatabaseType, password: &str) -> Vec<(String, String)> {
    match database_type {
        DatabaseType::PostgreSQL => vec![("POSTGRES_PASSWORD".to_string(), password.to_string())],
        DatabaseType::MySQL => vec![("MYSQL_ROOT_PASSWORD".to_string(), password.to_string())],
        DatabaseType::MongoDB => vec![
            ("MONGO_INITDB_ROOT_USERNAME".to_string(), "root".to_string()),
            ("MONGO_INITDB_ROOT_PASSWORD".to_string(), password.to_string()),
        ],
        DatabaseType::Redis => vec![],
    }
}

/// Get the command for a database type (used for Redis which uses CMD instead of ENV)
fn get_database_command(database_type: &DatabaseType, password: &str) -> Option<Vec<String>> {
    match database_type {
        DatabaseType::Redis => Some(vec![
            "redis-server".to_string(),
            "--requirepass".to_string(),
            password.to_string(),
        ]),
        _ => None,
    }
}

/// Convert bollard container state to InstanceStatus
fn get_instance_status(state: &bollard::models::ContainerState) -> InstanceStatus {
    if state.running == Some(true) {
        InstanceStatus::Running
    } else if state.paused == Some(true) {
        InstanceStatus::Stopped
    } else if state.restarting == Some(true) {
        InstanceStatus::Creating
    } else {
        // Check if there's an error
        if state.error.is_some() && !state.error.as_ref().unwrap().is_empty() {
            InstanceStatus::Error
        } else {
            InstanceStatus::Stopped
        }
    }
}

/// Create a new database container (without starting it)
/// Note: The image should be pulled separately via pull_docker_image before calling this
#[tauri::command]
pub async fn create_instance(request: CreateInstanceRequest) -> Result<Instance, String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    // Get port - either from request or auto-detect
    let port = match request.port {
        Some(p) => {
            // Check if port is already occupied
            let occupied = get_occupied_ports().await?;
            if occupied.contains(&p) {
                return Err(format!("Port {} is already in use", p));
            }
            p
        }
        None => {
            // Auto-detect next available port for this database type
            get_next_port_for_type(request.database_type).await?
        }
    };

    let container_name = format!("ldb-{}", request.name.replace(' ', "-").to_lowercase());
    let full_image = format!("{}:{}", request.image, request.tag);

    // Get environment variables based on database type
    let env_vars = get_env_vars(&request.database_type, &request.password);
    let env: Vec<String> = env_vars
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();

    // Create volume directory for this instance
    let instance_id = Uuid::new_v4().to_string();
    let volume_path = StateManager::get_instance_volume_path(&instance_id)?;
    let volume_path_str = volume_path.to_string_lossy().to_string();
    
    // Get the container's internal volume path
    let container_volume_path = get_volume_path(&request.database_type);
    let volume_bind = format!("{}:{}", volume_path_str, container_volume_path);

    // Get command if needed (Redis)
    let cmd = get_database_command(&request.database_type, &request.password);

    // Create port bindings
    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        format!("{}/tcp", port),
        Some(vec![bollard::models::PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(port.to_string()),
        }])
    );

    // Create exposed ports
    let mut exposed_ports = HashMap::new();
    exposed_ports.insert(format!("{}/tcp", port), HashMap::new());

    // Create container config
    let config = Config {
        image: Some(full_image.clone()),
        env: if env.is_empty() { None } else { Some(env) },
        cmd: cmd.clone(),
        host_config: Some(bollard::models::HostConfig {
            port_bindings: Some(port_bindings),
            binds: Some(vec![volume_bind]),
            ..Default::default()
        }),
        exposed_ports: Some(exposed_ports),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: container_name.clone(),
        platform: None,
    };

    // Note: Image should already be pulled via pull_docker_image before this call
    // We don't pull here to allow the frontend to show progress

    // Create the container
    let response = docker
        .create_container(Some(options), config)
        .await
        .map_err(|e| format!("Failed to create container: {}", e))?;

    let mut instance = Instance::new(
        request.name,
        request.database_type,
        request.image,
        request.tag,
        port,
        request.password,
    );
    
    // Set the instance ID to match what we created for the volume
    instance.id = Uuid::parse_str(&instance_id).map_err(|e| format!("Failed to parse instance ID: {}", e))?;
    instance.volume_path = Some(volume_path_str);

    // Persist instance state
    let state_manager = StateManager::new()?;
    state_manager.add_instance(instance.clone())?;

    println!(
        "Created container {} with ID {}",
        container_name, response.id
    );

    Ok(instance)
}

/// Start a container
#[tauri::command]
pub async fn start_instance(container_id: String) -> Result<Instance, String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    docker
        .start_container(&container_id, None::<StartContainerOptions<String>>)
        .await
        .map_err(|e| format!("Failed to start container: {}", e))?;

    // Inspect to get container details
    let info = docker
        .inspect_container(&container_id, None)
        .await
        .map_err(|e| format!("Failed to inspect container: {}", e))?;

    let config = info.config.ok_or("No config found")?;
    let state = info.state.ok_or("No state found")?;

    // Parse database type from image name
    let image = config.image.unwrap_or_default();
    let database_type = if image.contains("postgres") {
        DatabaseType::PostgreSQL
    } else if image.contains("redis") {
        DatabaseType::Redis
    } else if image.contains("mysql") {
        DatabaseType::MySQL
    } else if image.contains("mongo") {
        DatabaseType::MongoDB
    } else {
        DatabaseType::PostgreSQL
    };

    let name = info
        .name
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string();

    let created_at = info.created.map(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
        .flatten()
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    // Try to get volume path from state
    let state_manager = StateManager::new().ok();
    let volume_path = state_manager
        .and_then(|sm| {
            let instances = sm.load_instances().ok()?;
            instances.into_iter().find(|i| {
                let container_name = format!("ldb-{}", i.name.replace(' ', "-").to_lowercase());
                container_name == name
            })
        })
        .and_then(|i| i.volume_path);

    // Get port from network settings
    let port = info.network_settings
        .as_ref()
        .and_then(|ns| ns.ports.as_ref())
        .and_then(|ports| {
            ports.iter().next().and_then(|(_port_key, bindings)| {
                bindings.as_ref().and_then(|bindings| {
                    bindings.first().and_then(|b| {
                        b.host_port.as_ref().and_then(|p| p.parse().ok())
                    })
                })
            })
        })
        .unwrap_or(0);

    let instance = Instance {
        id: Uuid::new_v4(),
        name,
        database_type,
        image: image.clone(),
        tag: image.split(':').nth(1).unwrap_or("latest").to_string(),
        port,
        root_password: String::new(), // Not retrievable from inspect
        status: get_instance_status(&state),
        created_at,
        volume_path,
    };

    Ok(instance)
}

/// Stop a container
#[tauri::command]
pub async fn stop_instance(container_id: String) -> Result<Instance, String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let options = StopContainerOptions {
        t: 10, // 10 second timeout
    };

    docker
        .stop_container(&container_id, Some(options))
        .await
        .map_err(|e| format!("Failed to stop container: {}", e))?;

    // Inspect to get container details
    let info = docker
        .inspect_container(&container_id, None)
        .await
        .map_err(|e| format!("Failed to inspect container: {}", e))?;

    let config = info.config.ok_or("No config found")?;

    // Parse database type from image name
    let image = config.image.unwrap_or_default();
    let database_type = if image.contains("postgres") {
        DatabaseType::PostgreSQL
    } else if image.contains("redis") {
        DatabaseType::Redis
    } else if image.contains("mysql") {
        DatabaseType::MySQL
    } else if image.contains("mongo") {
        DatabaseType::MongoDB
    } else {
        DatabaseType::PostgreSQL
    };

    let name = info
        .name
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string();

    let created_at = info.created.map(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
        .flatten()
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    // Try to get volume path from state
    let state_manager = StateManager::new().ok();
    let volume_path = state_manager
        .and_then(|sm| {
            let instances = sm.load_instances().ok()?;
            instances.into_iter().find(|i| {
                let container_name = format!("ldb-{}", i.name.replace(' ', "-").to_lowercase());
                container_name == name
            })
        })
        .and_then(|i| i.volume_path);

    // Get port from network settings
    let port = info.network_settings
        .as_ref()
        .and_then(|ns| ns.ports.as_ref())
        .and_then(|ports| {
            ports.iter().next().and_then(|(_port_key, bindings)| {
                bindings.as_ref().and_then(|bindings| {
                    bindings.first().and_then(|b| {
                        b.host_port.as_ref().and_then(|p| p.parse().ok())
                    })
                })
            })
        })
        .unwrap_or(0);

    let instance = Instance {
        id: Uuid::new_v4(),
        name,
        database_type,
        image: image.clone(),
        tag: image.split(':').nth(1).unwrap_or("latest").to_string(),
        port,
        root_password: String::new(),
        status: InstanceStatus::Stopped,
        created_at,
        volume_path,
    };

    Ok(instance)
}

/// Restart a container
#[tauri::command]
pub async fn restart_instance(container_id: String) -> Result<Instance, String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let options = bollard::container::RestartContainerOptions {
        t: 10, // 10 second timeout
    };

    docker
        .restart_container(&container_id, Some(options))
        .await
        .map_err(|e| format!("Failed to restart container: {}", e))?;

    // Wait a moment for container to be fully running
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Inspect to get container details
    let info = docker
        .inspect_container(&container_id, None)
        .await
        .map_err(|e| format!("Failed to inspect container: {}", e))?;

    let config = info.config.ok_or("No config found")?;
    let state = info.state.ok_or("No state found")?;

    // Parse database type from image name
    let image = config.image.unwrap_or_default();
    let database_type = if image.contains("postgres") {
        DatabaseType::PostgreSQL
    } else if image.contains("redis") {
        DatabaseType::Redis
    } else if image.contains("mysql") {
        DatabaseType::MySQL
    } else if image.contains("mongo") {
        DatabaseType::MongoDB
    } else {
        DatabaseType::PostgreSQL
    };

    let name = info
        .name
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string();

    let created_at = info.created.map(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
        .flatten()
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    // Try to get volume path from state
    let state_manager = StateManager::new().ok();
    let volume_path = state_manager
        .and_then(|sm| {
            let instances = sm.load_instances().ok()?;
            instances.into_iter().find(|i| {
                let container_name = format!("ldb-{}", i.name.replace(' ', "-").to_lowercase());
                container_name == name
            })
        })
        .and_then(|i| i.volume_path);

    // Get port from network settings
    let port = info.network_settings
        .as_ref()
        .and_then(|ns| ns.ports.as_ref())
        .and_then(|ports| {
            ports.iter().next().and_then(|(_port_key, bindings)| {
                bindings.as_ref().and_then(|bindings| {
                    bindings.first().and_then(|b| {
                        b.host_port.as_ref().and_then(|p| p.parse().ok())
                    })
                })
            })
        })
        .unwrap_or(0);

    let instance = Instance {
        id: Uuid::new_v4(),
        name,
        database_type,
        image: image.clone(),
        tag: image.split(':').nth(1).unwrap_or("latest").to_string(),
        port,
        root_password: String::new(),
        status: get_instance_status(&state),
        created_at,
        volume_path,
    };

    Ok(instance)
}

/// List all containers (including stopped)
#[tauri::command]
pub async fn list_instances() -> Result<Vec<Instance>, String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let options = Some(ListContainersOptions::<String> {
        all: true, // Include stopped containers
        ..Default::default()
    });

    let containers = docker
        .list_containers(options)
        .await
        .map_err(|e| format!("Failed to list containers: {}", e))?;

    let mut instances = Vec::new();

    for container in containers {
        // Filter to only include our database containers
        let name = container.names.as_ref()
            .and_then(|names| names.first())
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_default();

        // Only include containers we created (prefixed with ldb-)
        if !name.starts_with("ldb-") {
            continue;
        }

        let container_id = container.id.unwrap_or_default();

        // Inspect each container to get more details
        let info = docker
            .inspect_container(&container_id, None)
            .await
            .map_err(|e| format!("Failed to inspect container {}: {}", container_id, e))?;

        let config = info.config.ok_or("No config found")?;
        let state = info.state.ok_or("No state found")?;

        // Parse database type from image name
        let image = config.image.unwrap_or_default();
        let database_type = if image.contains("postgres") {
            DatabaseType::PostgreSQL
        } else if image.contains("redis") {
            DatabaseType::Redis
        } else if image.contains("mysql") {
            DatabaseType::MySQL
        } else if image.contains("mongo") {
            DatabaseType::MongoDB
        } else {
            DatabaseType::PostgreSQL
        };

        let created_at = info.created.map(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        // Try to get volume path from state
        let state_manager = StateManager::new().ok();
        let volume_path = state_manager
            .and_then(|sm| {
                let instances = sm.load_instances().ok()?;
                instances.into_iter().find(|i| {
                    let container_name = format!("ldb-{}", i.name.replace(' ', "-").to_lowercase());
                    container_name == name
                })
            })
            .and_then(|i| i.volume_path);

        let instance = Instance {
            id: Uuid::new_v4(),
            name,
            database_type,
            image: image.clone(),
            tag: image.split(':').nth(1).unwrap_or("latest").to_string(),
            port: container.ports.as_ref()
                .and_then(|ports| ports.first())
                .and_then(|p| p.public_port)
                .unwrap_or(0),
            root_password: String::new(),
            status: get_instance_status(&state),
            created_at,
            volume_path,
        };

        instances.push(instance);
    }

    Ok(instances)
}

/// Get the current status of a container
#[tauri::command]
pub async fn get_container_status_string(container_id: String) -> Result<String, String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let info = docker
        .inspect_container(&container_id, None)
        .await
        .map_err(|e| format!("Failed to inspect container: {}", e))?;

    let state = info.state.ok_or("No state found")?;

    if state.running == Some(true) {
        Ok("running".to_string())
    } else if state.paused == Some(true) {
        Ok("paused".to_string())
    } else if state.restarting == Some(true) {
        Ok("restarting".to_string())
    } else if state.oom_killed == Some(true) {
        Ok("oom_killed".to_string())
    } else if state.dead == Some(true) {
        Ok("dead".to_string())
    } else {
        // Check status field which is a string in newer bollard
        if let Some(status) = &state.status {
            return Ok(status.to_string());
        }
        Ok("stopped".to_string())
    }
}

/// Delete a container
#[tauri::command]
pub async fn delete_instance(container_id: String, delete_volume: Option<bool>) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    // First, try to get the instance from state to find volume path
    let state_manager = StateManager::new()?;
    let instances = state_manager.load_instances()?;
    
    // Find instance by container_id (name starts with ldb-)
    let instance_to_delete = instances.iter().find(|i| {
        let container_name = format!("ldb-{}", i.name.replace(' ', "-").to_lowercase());
        container_name == container_id
    });

    let volume_path = instance_to_delete.and_then(|i| i.volume_path.clone());

    let options = RemoveContainerOptions {
        force: true,
        ..Default::default()
    };

    docker
        .remove_container(&container_id, Some(options))
        .await
        .map_err(|e| format!("Failed to delete container: {}", e))?;

    // Remove from state
    if let Some(instance) = instance_to_delete {
        let _ = state_manager.remove_instance(&instance.id.to_string());
    }

    // Delete volume directory if requested
    if delete_volume.unwrap_or(false) {
        if let Some(vp) = volume_path {
            if std::path::Path::new(&vp).exists() {
                std::fs::remove_dir_all(&vp)
                    .map_err(|e| format!("Failed to delete volume directory: {}", e))?;
            }
        } else if let Some(instance) = instance_to_delete {
            // Fallback: try to delete by instance ID
            let _ = crate::state::remove_volume_dir(&instance.id.to_string());
        }
    }

    Ok(())
}

/// Get the volume path for a specific instance
#[tauri::command]
pub fn get_instance_volume_path(instance_id: String) -> Result<String, String> {
    let path = StateManager::get_instance_volume_path(&instance_id)?;
    Ok(path.to_string_lossy().to_string())
}
