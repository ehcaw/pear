use crate::error::{AppError, Result};
use crate::neo4j::NeoDB;
use crate::parser::Parser;
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::DialogExt;

/// Parse a codebase and ingest it into Neo4j
#[tauri::command]
pub async fn parse_and_ingest_codebase(app_handle: AppHandle, directory: String) -> Result<String> {
    // Emit event that parsing has started
    app_handle
        .emit("parse_progress", "Starting codebase analysis")
        .unwrap();

    // Create Neo4j connection
    let neo_db = NeoDB::new().await?;

    // Create parser
    let mut parser = Parser::new();

    // Parse and ingest
    let file_count = parser
        .parse_and_ingest_directory(&app_handle, &neo_db, &directory)
        .await?;

    // Emit event that parsing is complete
    app_handle
        .emit(
            "parse_complete",
            format!("Analysis complete. Processed {} files.", file_count),
        )
        .unwrap();

    Ok(format!("Successfully processed {} files.", file_count))
}
