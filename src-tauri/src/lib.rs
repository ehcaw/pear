// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
//
mod ai;
mod audio;
mod file;
mod supabase;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::Match;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf}; // Import Match type

use ai::transcribe_generate_play;
use audio::{is_audio_playing, pause_audio, play_audio_file, resume_audio, stop_audio};
use file::{embed_codebase, read_directory_structure, read_file};
use supabase::query_document;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            transcribe_generate_play,
            is_audio_playing,
            pause_audio,
            play_audio_file,
            resume_audio,
            stop_audio,
            read_file,
            read_directory_structure,
            embed_codebase,
            query_document
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
