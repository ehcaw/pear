// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
//
mod ai;
mod audio;

use ai::transcribe_generate_play;
use audio::{is_audio_playing, pause_audio, play_audio_file, resume_audio, stop_audio};

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
            transcribe_generate_play,
            is_audio_playing,
            pause_audio,
            play_audio_file,
            resume_audio,
            stop_audio
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
