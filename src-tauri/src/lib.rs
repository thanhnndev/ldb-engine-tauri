// Docker module for container management
pub mod docker;
pub mod commands;
pub mod models;
pub mod state;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::images::get_docker_tags,
            commands::images::get_supported_images,
            commands::images::pull_docker_image,
            commands::instances::create_instance,
            commands::instances::start_instance,
            commands::instances::stop_instance,
            commands::instances::restart_instance,
            commands::instances::delete_instance,
            commands::instances::list_instances,
            commands::instances::get_container_status_string,
            commands::instances::get_instance_volume_path,
            commands::connections::get_connection_string,
            commands::ports::get_occupied_ports,
            commands::ports::get_available_port,
            commands::ports::get_next_port_for_type,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
