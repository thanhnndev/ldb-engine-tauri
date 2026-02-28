use bollard::Docker;
use bollard::container::ListContainersOptions;

use crate::models::instance::DatabaseType;

/// Base ports for each database type
pub const BASE_PORT_POSTGRES: u16 = 5432;
pub const BASE_PORT_REDIS: u16 = 6379;
pub const BASE_PORT_MYSQL: u16 = 3306;
pub const BASE_PORT_MONGODB: u16 = 27017;

/// Get the base port for a database type
pub fn get_base_port(database_type: &DatabaseType) -> u16 {
    match database_type {
        DatabaseType::PostgreSQL => BASE_PORT_POSTGRES,
        DatabaseType::Redis => BASE_PORT_REDIS,
        DatabaseType::MySQL => BASE_PORT_MYSQL,
        DatabaseType::MongoDB => BASE_PORT_MONGODB,
    }
}

/// Get occupied ports from all running Docker containers
#[tauri::command]
pub async fn get_occupied_ports() -> Result<Vec<u16>, String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let options = Some(ListContainersOptions::<String> {
        all: false, // Only running containers
        ..Default::default()
    });

    let containers = docker
        .list_containers(options)
        .await
        .map_err(|e| format!("Failed to list containers: {}", e))?;

    let mut occupied_ports: Vec<u16> = Vec::new();

    for container in containers {
        if let Some(ports) = container.ports {
            for port in ports {
                if let Some(public_port) = port.public_port {
                    if !occupied_ports.contains(&public_port) {
                        occupied_ports.push(public_port);
                    }
                }
            }
        }
    }

    occupied_ports.sort();
    Ok(occupied_ports)
}

/// Check if a port is available (not occupied by any container)
async fn is_port_available(port: u16) -> Result<bool, String> {
    let occupied = get_occupied_ports().await?;
    Ok(!occupied.contains(&port))
}

/// Find an available port, starting from the preferred port
#[tauri::command]
pub async fn get_available_port(preferred_port: Option<u16>) -> Result<u16, String> {
    let start_port = preferred_port.unwrap_or(BASE_PORT_POSTGRES);

    // Check if preferred port is available
    if is_port_available(start_port).await? {
        return Ok(start_port);
    }

    // Search for next available port
    for port in start_port..=65535 {
        if is_port_available(port).await? {
            return Ok(port);
        }
    }

    Err("No available ports found".to_string())
}

/// Get the next available port for a specific database type
#[tauri::command]
pub async fn get_next_port_for_type(database_type: DatabaseType) -> Result<u16, String> {
    let base_port = get_base_port(&database_type);
    get_available_port(Some(base_port)).await
}
