use crate::env_utils;
use crate::error::Result;
use crate::file_manager::{neo4j::NeoDB, AppState};
use crate::parser::Parser;

use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

/// Parse a codebase and ingest it into Neo4j
#[tauri::command]
pub async fn parse_and_ingest_codebase(app_handle: AppHandle, directory: String) -> Result<String> {
    // Emit event that parsing has started
    app_handle
        .emit("parse_progress", "Starting codebase analysis")
        .unwrap();
    env_utils::init()?;
    let uri = env_utils::get_required("NEO4J_URI")?;
    let username = env_utils::get_required("NEO4J_USER")?;
    let password = env_utils::get_required("NEO4J_PASSWORD")?;

    // Create Neo4j connection
    let neo_db = NeoDB::new_simple(uri, username, password).await?;

    // Create parser
    let mut parser = Parser::new();

    // Parse and ingest
    let (nodes, links) = parser
        .parse_and_ingest_directory(&app_handle, &neo_db, &directory)
        .await?;

    // Emit event that parsing is complete
    app_handle
        .emit(
            "parse_complete",
            format!("Analysis complete. Processed {} files.", nodes.len()),
        )
        .unwrap();

    Ok(format!("Successfully processed {} files.", nodes.len()))
}

#[tauri::command]
pub async fn track_repository(path: String, owner_id: String) -> Result<String> {
    // Generate a unique repository ID
    let repository_id = Uuid::new_v4().to_string();

    // Initialize file tracker
    // Initialize environment
    env_utils::init()?;

    // Get Neo4j connection details
    let neo4j_updater = NeoDB::new(
        &env_utils::get_required("NEO4J_URI")?,
        &env_utils::get_required("NEO4J_USER")?,
        &env_utils::get_required("NEO4J_PASSWORD")?,
        repository_id.clone(),
        owner_id,
    )
    .await?;

    // Register repository in Neo4j
    neo4j_updater.register_repository(&path).await?;

    // Return the repository ID to the frontend
    Ok(repository_id)
}

#[tauri::command]
pub async fn start_watching_directory(
    app_handle: tauri::AppHandle,
    directory: String,
) -> Result<String> {
    // Get app state
    let state = app_handle.state::<AppState>();

    // Start watching the directory
    state
        .start_watching(std::path::PathBuf::from(directory))
        .await?;

    Ok("Started watching directory".to_string())
}

#[tauri::command]
pub async fn stop_watching_directory(app_handle: tauri::AppHandle) -> Result<String> {
    // Get app state
    let state = app_handle.state::<AppState>();

    // Stop watching
    state.stop_watching().await;

    Ok("Stopped watching directory".to_string())
}
