// Docker module for container management
pub mod docker;
pub mod commands;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::images::get_docker_tags,
            commands::images::get_supported_images,
            commands::images::pull_docker_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
