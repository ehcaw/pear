use crate::file_manager::file_watcher::FileWatcherSystem;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    file_watcher: Arc<Mutex<Option<FileWatcherSystem>>>,
    // Add other app state here
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            file_watcher: Arc::new(Mutex::new(None)),
            // Initialize other state
        }
    }

    // Method to start watching a directory
    pub async fn start_watching(&self, repo_path: std::path::PathBuf) -> crate::error::Result<()> {
        // Get Neo4j connection details from environment variables
        let uri = crate::env_utils::get_required("NEO4J_URI")?;
        let username = crate::env_utils::get_required("NEO4J_USER")?;
        let password = crate::env_utils::get_required("NEO4J_PASSWORD")?;

        // Initialize components
        let neo_db = Arc::new(
            crate::file_manager::neo4j::NeoDB::new(
                &uri,
                &username,
                &password,
                "repo-id".to_string(),
                "owner-id".to_string(),
            )
            .await?,
        );

        let parser = Arc::new(tokio::sync::Mutex::new(crate::parser::Parser::new()));
        let file_tracker = Arc::new(tokio::sync::Mutex::new(
            crate::file_manager::file_tracker::FileTracker::new(
                repo_path.clone(),
                "repo-id".to_string(),
                "owner-id".to_string(),
            ),
        ));

        // Create the file watcher
        let mut watcher = FileWatcherSystem::new(repo_path, neo_db, parser, file_tracker).unwrap();

        // Start watching
        watcher.start();

        // Store the watcher in app state
        let mut file_watcher_guard = self.file_watcher.lock().await;
        *file_watcher_guard = Some(watcher);

        // Start the background task for processing events
        let watcher_clone = self.file_watcher.clone();
        tokio::spawn(async move {
            loop {
                // Check if watcher exists and process events
                let mut guard = watcher_clone.lock().await;
                if let Some(watcher) = &mut *guard {
                    if let Err(e) = watcher.process_events().await {
                        log::error!("Error processing file events: {}", e);
                    }
                } else {
                    // Watcher has been stopped
                    break;
                }

                // Release lock before sleeping
                drop(guard);
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        });

        Ok(())
    }

    // Method to stop watching
    pub async fn stop_watching(&self) -> crate::error::Result<()> {
        let mut file_watcher_guard = self.file_watcher.lock().await;
        if let Some(watcher) = &mut *file_watcher_guard {
            watcher.stop();
            *file_watcher_guard = None;
        }
        Ok(())
    }
}
