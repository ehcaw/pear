// Modules
mod commands;
mod env_utils;
mod error;
mod file_manager;
mod fs;
mod models;
mod parser;
mod treesitter;

// Re-exports
pub use commands::*;
pub use file_manager::AppState;
pub use fs::{read_directory_structure, read_file_content};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load environment variables
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            parse_and_ingest_codebase,
            track_repository,
            read_directory_structure,
            read_file_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
