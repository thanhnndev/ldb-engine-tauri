use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::exec::CreateExecOptions;
use bollard::image::CreateImageOptions;
use bollard::Docker;
use chrono::Utc;
use futures::StreamExt;
use uuid::Uuid;

use crate::docker::client::DockerClient;
use crate::models::instance::{CreateInstanceRequest, DatabaseType, Instance, InstanceStatus};

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
fn get_instance_status(state: &bollard::container::State) -> InstanceStatus {
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
#[tauri::command]
pub async fn create_instance(request: CreateInstanceRequest) -> Result<Instance, String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let container_name = format!("ldb-{}", request.name.replace(' ', "-").to_lowercase());
    let full_image = format!("{}:{}", request.image, request.tag);

    // Get environment variables based on database type
    let env_vars = get_env_vars(&request.database_type, &request.password);
    let env: Vec<String> = env_vars
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();

    // Get volume mount path
    let volume_path = get_volume_path(&request.database_type);
    let volume_bind = format!("{}:{}", volume_path, volume_path);

    // Get command if needed (Redis)
    let cmd = get_database_command(&request.database_type, &request.password);

    // Create container config
    let config = Config {
        image: Some(full_image.clone()),
        env: if env.is_empty() { None } else { Some(env) },
        cmd: cmd.clone(),
        host_config: Some(bollard::service::HostConfig {
            port_bindings: Some(bollard::service::PortBindings {
                entry: Some(vec![bollard::service::PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(request.port.to_string()),
                })]),
                ..Default::default()
            }),
            binds: Some(vec![volume_bind]),
            ..Default::default()
        }),
        exposed_ports: Some(
            [(request.port.to_string(), bollard::service::ExposedPort::tcp(request.port))].into(),
        ),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: container_name.clone(),
        platform: None,
    };

    // Pull image first if not present
    let mut stream = docker.create_image(
        Some(CreateImageOptions {
            from_image: full_image.clone(),
            ..Default::default()
        }),
        None,
        None,
    );

    while let Some(result) = stream.next().await {
        match result {
            Ok(info) => {
                if let Some(status) = info.status {
                    println!("Pull status: {}", status);
                }
            }
            Err(e) => {
                return Err(format!("Failed to pull image: {}", e));
            }
        }
    }

    // Create the container
    let response = docker
        .create_container(Some(options), config)
        .await
        .map_err(|e| format!("Failed to create container: {}", e))?;

    let instance = Instance::new(
        request.name,
        request.database_type,
        request.image,
        request.tag,
        request.port,
        request.password,
    );

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
        .map(|dt| dt.with_timezone(&Utc::))
        .unwrap_or_else(Utc::now);

    let instance = Instance {
        id: Uuid::new_v4(),
        name,
        database_type,
        image: image.clone(),
        tag: image.split(':').nth(1).unwrap_or("latest").to_string(),
        port: info.network_settings
            .as_ref()
            .and_then(|ns| ns.ports.as_ref())
            .and_then(|ports| ports.values().next())
            .and_then(|bindings| bindings.first())
            .and_then(|b| b.host_port.as_ref())
            .and_then(|p| p.parse().ok())
            .unwrap_or(0),
        root_password: String::new(), // Not retrievable from inspect
        status: get_instance_status(&state),
        created_at,
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
        .map(|dt| dt.with_timezone(&Utc::))
        .unwrap_or_else(Utc::now);

    let instance = Instance {
        id: Uuid::new_v4(),
        name,
        database_type,
        image: image.clone(),
        tag: image.split(':').nth(1).unwrap_or("latest").to_string(),
        port: info.network_settings
            .as_ref()
            .and_then(|ns| ns.ports.as_ref())
            .and_then(|ports| ports.values().next())
            .and_then(|bindings| bindings.first())
            .and_then(|b| b.host_port.as_ref())
            .and_then(|p| p.parse().ok())
            .unwrap_or(0),
        root_password: String::new(),
        status: InstanceStatus::Stopped,
        created_at,
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
        .map(|dt| dt.with_timezone(&Utc::))
        .unwrap_or_else(Utc::now);

    let instance = Instance {
        id: Uuid::new_v4(),
        name,
        database_type,
        image: image.clone(),
        tag: image.split(':').nth(1).unwrap_or("latest").to_string(),
        port: info.network_settings
            .as_ref()
            .and_then(|ns| ns.ports.as_ref())
            .and_then(|ports| ports.values().next())
            .and_then(|bindings| bindings.first())
            .and_then(|b| b.host_port.as_ref())
            .and_then(|p| p.parse().ok())
            .unwrap_or(0),
        root_password: String::new(),
        status: get_instance_status(&state),
        created_at,
    };

    Ok(instance)
}

/// Delete a container
#[tauri::command]
pub async fn delete_instance(container_id: String) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let options = RemoveContainerOptions {
        force: true,
        ..Default::default()
    };

    docker
        .remove_container(&container_id, Some(options))
        .await
        .map_err(|e| format!("Failed to delete container: {}", e))?;

    Ok(())
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
            .map(|dt| dt.with_timezone(&Utc::))
            .unwrap_or_else(Utc::now);

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
        };

        instances.push(instance);
    }

    Ok(instances)
}

/// Get the current status of a container
#[tauri::command]
pub async fn get_instance_status(container_id: String) -> Result<String, String> {
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
    } else if state.removing == Some(true) {
        Ok("removing".to_string())
    } else if state.exited.is_some() {
        Ok("exited".to_string())
    } else if state.dead == Some(true) {
        Ok("dead".to_string())
    } else {
        Ok("created".to_string())
    }
}
