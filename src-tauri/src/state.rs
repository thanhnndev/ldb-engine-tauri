use std::fs;
use std::path::PathBuf;

use crate::models::instance::Instance;

/// State manager for persisting instance metadata
pub struct StateManager {
    /// Path to the instances JSON file
    instances_file: PathBuf,
}

impl StateManager {
    /// Create a new StateManager with the default data directory
    pub fn new() -> Result<Self, String> {
        let data_dir = Self::get_data_dir()?;
        
        // Ensure data directory exists
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)
                .map_err(|e| format!("Failed to create data directory: {}", e))?;
        }
        
        let instances_file = data_dir.join("instances.json");
        
        Ok(Self { instances_file })
    }
    
    /// Get the data directory path (~/.ldb-engine/)
    fn get_data_dir() -> Result<PathBuf, String> {
        let home = dirs::home_dir()
            .ok_or("Could not find home directory")?;
        
        Ok(home.join(".ldb-engine"))
    }
    
    /// Get the volume directory path (~/.ldb-engine/volumes/)
    pub fn get_volume_dir() -> Result<PathBuf, String> {
        let data_dir = Self::get_data_dir()?;
        let volume_dir = data_dir.join("volumes");
        
        if !volume_dir.exists() {
            fs::create_dir_all(&volume_dir)
                .map_err(|e| format!("Failed to create volume directory: {}", e))?;
        }
        
        Ok(volume_dir)
    }
    
    /// Get the volume path for a specific instance
    pub fn get_instance_volume_path(instance_id: &str) -> Result<PathBuf, String> {
        let volume_dir = Self::get_volume_dir()?;
        let instance_volume_dir = volume_dir.join(instance_id);
        
        if !instance_volume_dir.exists() {
            fs::create_dir_all(&instance_volume_dir)
                .map_err(|e| format!("Failed to create instance volume directory: {}", e))?;
        }
        
        Ok(instance_volume_dir)
    }
    
    /// Load all instances from the JSON file
    /// Returns an empty Vec if the file doesn't exist yet
    pub fn load_instances(&self) -> Result<Vec<Instance>, String> {
        if !self.instances_file.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&self.instances_file)
            .map_err(|e| format!("Failed to read instances file: {}", e))?;
        
        let instances: Vec<Instance> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse instances file: {}", e))?;
        
        Ok(instances)
    }
    
    /// Save all instances to the JSON file
    pub fn save_instances(&self, instances: &[Instance]) -> Result<(), String> {
        let content = serde_json::to_string_pretty(instances)
            .map_err(|e| format!("Failed to serialize instances: {}", e))?;
        
        fs::write(&self.instances_file, content)
            .map_err(|e| format!("Failed to write instances file: {}", e))?;
        
        Ok(())
    }
    
    /// Add a new instance and save
    pub fn add_instance(&self, instance: Instance) -> Result<(), String> {
        let mut instances = self.load_instances()?;
        instances.push(instance);
        self.save_instances(&instances)
    }
    
    /// Update an existing instance and save
    pub fn update_instance(&self, instance: Instance) -> Result<(), String> {
        let mut instances = self.load_instances()?;
        
        if let Some(pos) = instances.iter().position(|i| i.id == instance.id) {
            instances[pos] = instance;
            self.save_instances(&instances)
        } else {
            Err(format!("Instance not found: {}", instance.id))
        }
    }
    
    /// Remove an instance by ID and save
    pub fn remove_instance(&self, id: &str) -> Result<(), String> {
        let mut instances = self.load_instances()?;
        
        let initial_len = instances.len();
        instances.retain(|i| i.id.to_string() != id);
        
        if instances.len() == initial_len {
            return Err(format!("Instance not found: {}", id));
        }
        
        self.save_instances(&instances)
    }
    
    /// Get an instance by ID
    pub fn get_instance(&self, id: &str) -> Result<Option<Instance>, String> {
        let instances = self.load_instances()?;
        Ok(instances.into_iter().find(|i| i.id.to_string() == id))
    }
    
    /// Get the path to the instances file
    pub fn get_instances_file_path(&self) -> &PathBuf {
        &self.instances_file
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create StateManager")
    }
}

/// Convenience function to load all instances
pub fn load_instances() -> Result<Vec<Instance>, String> {
    let manager = StateManager::new()?;
    manager.load_instances()
}

/// Convenience function to save all instances
pub fn save_instances(instances: &[Instance]) -> Result<(), String> {
    let manager = StateManager::new()?;
    manager.save_instances(instances)
}

/// Remove the volume directory for an instance
pub fn remove_volume_dir(instance_id: &str) -> Result<(), String> {
    let volume_dir = StateManager::get_instance_volume_path(instance_id)?;
    
    if volume_dir.exists() {
        fs::remove_dir_all(&volume_dir)
            .map_err(|e| format!("Failed to remove volume directory: {}", e))?;
    }
    
    Ok(())
}
