use crate::models::instance::DatabaseType;
use crate::state::StateManager;

/// Generate a standard connection string for a database instance
///
/// Returns a properly formatted connection string based on the database type:
/// - PostgreSQL: postgresql://postgres:{password}@127.0.0.1:{port}/{name}
/// - Redis: redis://:{password}@127.0.0.1:{port}
/// - MySQL: mysql://root:{password}@127.0.0.1:{port}/{name}
/// - MongoDB: mongodb://root:{password}@127.0.0.1:{port}/{name}?authSource=admin
#[tauri::command]
pub async fn get_connection_string(instance_id: String) -> Result<String, String> {
    // Look up the instance from StateManager
    let state_manager = StateManager::new()?;
    let instance = state_manager
        .get_instance(&instance_id)?
        .ok_or_else(|| "Instance not found".to_string())?;

    // Transform instance name to database name (lowercase, replace spaces with underscores)
    let db_name = instance.name.to_lowercase().replace(' ', "_");

    // Generate connection string based on database type
    let connection_string = match instance.database_type {
        DatabaseType::PostgreSQL => format!(
            "postgresql://postgres:{}@127.0.0.1:{}/{}",
            instance.root_password, instance.port, db_name
        ),
        DatabaseType::Redis => format!(
            "redis://:{}@127.0.0.1:{}",
            instance.root_password, instance.port
        ),
        DatabaseType::MySQL => format!(
            "mysql://root:{}@127.0.0.1:{}/{}",
            instance.root_password, instance.port, db_name
        ),
        DatabaseType::MongoDB => format!(
            "mongodb://root:{}@127.0.0.1:{}/{}?authSource=admin",
            instance.root_password, instance.port, db_name
        ),
    };

    Ok(connection_string)
}
