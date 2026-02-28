use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported database types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    PostgreSQL,
    Redis,
    MySQL,
    MongoDB,
}

impl Default for DatabaseType {
    fn default() -> Self {
        DatabaseType::PostgreSQL
    }
}

/// Instance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstanceStatus {
    Running,
    Stopped,
    Error,
    Creating,
}

impl Default for InstanceStatus {
    fn default() -> Self {
        InstanceStatus::Stopped
    }
}

/// Database instance representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: Uuid,
    pub name: String,
    pub database_type: DatabaseType,
    pub image: String,
    pub tag: String,
    pub port: u16,
    pub root_password: String,
    pub status: InstanceStatus,
    pub created_at: DateTime<Utc>,
}

impl Instance {
    pub fn new(
        name: String,
        database_type: DatabaseType,
        image: String,
        tag: String,
        port: u16,
        root_password: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            database_type,
            image,
            tag,
            port,
            root_password,
            status: InstanceStatus::Stopped,
            created_at: Utc::now(),
        }
    }
}

/// Request to create a new instance
#[derive(Debug, Clone, Deserialize)]
pub struct CreateInstanceRequest {
    pub name: String,
    pub database_type: DatabaseType,
    pub image: String,
    pub tag: String,
    pub password: String,
    pub port: u16,
}
