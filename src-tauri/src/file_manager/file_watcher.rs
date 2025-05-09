use crate::error::{AppError, Result as AppResult};
use crate::file_manager::file_tracker::FileTracker;
use crate::file_manager::neo4j::NeoDB;
use crate::parser::Parser;

use log::{error, info};
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, Receiver as MpscReceiver};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Error;

const DEBOUNCE_TIME: Duration = Duration::from_millis(300);

pub struct FileWatcherSystem {
    watcher: RecommendedWatcher,
    // rx: std::sync::mpsc::Receiver<std::result::Result<notify::Event, notify::Error>>, // Remove this
    neo_db: Arc<NeoDB>,
    parser: Arc<Mutex<Parser>>,
    file_tracker: Arc<Mutex<FileTracker>>,
    repository_path: PathBuf,
    ignore_patterns: Vec<String>,
    pending_changes: Arc<Mutex<HashMap<PathBuf, Instant>>>,
    // Add a flag to signal if watching is active
    is_watching: Arc<AtomicBool>,
}

impl FileWatcherSystem {
    pub fn new(
        repository_path: PathBuf,
        neo_db: Arc<NeoDB>,
        parser: Arc<Mutex<Parser>>,
        file_tracker: Arc<Mutex<FileTracker>>,
    ) -> Result<Self, Error> {
        // Create a channel to receive the events
        let (tx, rx) = std::sync::mpsc::channel();

        let pending_changes = Arc::new(Mutex::new(HashMap::new()));
        let is_watching = Arc::new(AtomicBool::new(false));

        // Create a watcher with default configuration
        let watcher = RecommendedWatcher::new(
            move |res| {
                tx.send(res).unwrap_or_else(|e| {
                    error!("Failed to send event through channel: {}", e);
                });
            },
            Config::default(),
        )
        .map_err(|e| AppError::Config(format!("Failed to create file watcher: {}", e)))
        .unwrap();

        // Default ignore patterns
        let ignore_patterns = vec![
            "node_modules".to_string(),
            ".git".to_string(),
            "target".to_string(),
            "dist".to_string(),
            "build".to_string(),
            "__pycache__".to_string(),
            ".next".to_string(),
            ".nuxt".to_string(),
        ];

        let neo_db_clone = neo_db.clone();
        let parser_clone = parser.clone();
        let file_tracker_clone = file_tracker.clone();
        let repository_path_clone = repository_path.clone();
        let ignore_patterns_clone = ignore_patterns.clone();

        let system = FileWatcherSystem {
            watcher,
            neo_db: neo_db_clone,
            parser: parser_clone,
            file_tracker: file_tracker_clone,
            repository_path: repository_path_clone,
            ignore_patterns: ignore_patterns_clone,
            pending_changes: pending_changes.clone(),
            is_watching: is_watching.clone(),
        };

        tokio::spawn(async move {
            // Process events while the watcher is active
            while is_watching.load(std::sync::atomic::Ordering::SeqCst) {
                match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(Ok(event)) => {
                        // Process the event (implementation details...)
                        Self::process_event(
                            &event,
                            &neo_db,
                            &parser,
                            &file_tracker,
                            &repository_path,
                            &ignore_patterns,
                            &pending_changes,
                        )
                        .await
                        .unwrap_or_else(|e| {
                            error!("Error processing event: {}", e);
                        });
                    }
                    Ok(Err(e)) => {
                        error!("Watch error: {:?}", e);
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // No events, check for debounced files
                        let neoclone = neo_db.clone();
                        let parserclone = parser.clone();
                        let filetrackerclone = file_tracker.clone();
                        Self::process_debounced_files(
                            &neoclone,
                            &parserclone,
                            &filetrackerclone,
                            &pending_changes,
                        )
                        .await
                        .unwrap_or_else(|e| {
                            error!("Error processing debounced files: {}", e);
                        });

                        // Sleep a bit to avoid busy waiting
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        // Channel closed, exit loop
                        break;
                    }
                }
            }
        });

        Ok(system)
    }

    // Start watching the directory
    pub fn start(&mut self) -> Result<(), String> {
        self.watcher
            .watch(&self.repository_path, RecursiveMode::Recursive)
            .map_err(|e| AppError::Config(format!("Failed to watch directory: {}", e)));

        // Set the watching flag to true
        self.is_watching
            .store(true, std::sync::atomic::Ordering::SeqCst);

        info!(
            "Started watching directory: {}",
            self.repository_path.display()
        );

        Ok(())
    }

    // Stop watching the directory
    pub fn stop(&mut self) -> Result<(), String> {
        self.watcher
            .unwatch(&self.repository_path)
            .map_err(|e| AppError::Config(format!("Failed to unwatch directory: {}", e)));

        // Set the watching flag to false
        self.is_watching
            .store(false, std::sync::atomic::Ordering::SeqCst);

        info!(
            "Stopped watching directory: {}",
            self.repository_path.display()
        );

        Ok(())
    }

    // Process a single event (static method)
    async fn process_event(
        event: &Event,
        neo_db: &Arc<NeoDB>,
        parser: &Arc<Mutex<Parser>>,
        file_tracker: &Arc<Mutex<FileTracker>>,
        repository_path: &PathBuf,
        ignore_patterns: &Vec<String>,
        pending_changes: &Arc<Mutex<HashMap<PathBuf, Instant>>>,
    ) -> Result<(), String> {
        // Extract the paths affected by this event
        let filtered_paths: Vec<PathBuf> = event
            .paths
            .iter()
            .filter(|path| !Self::should_ignore(path, ignore_patterns))
            .cloned()
            .collect();

        if filtered_paths.is_empty() {
            return Ok(());
        }

        // Handle the event based on its kind
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in filtered_paths {
                    if path.is_file() {
                        // Update the pending changes with the current time
                        let mut changes = pending_changes.lock().await;
                        changes.insert(path.clone(), Instant::now());
                        info!("Detected change in file: {}", path.display());
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in filtered_paths {
                    info!("Detected removal of: {}", path.display());
                    // Handle file removal by updating the database
                    Self::handle_file_removal(&path, neo_db, file_tracker).await?;
                }
            }
            EventKind::Modify(ModifyKind::Name(_)) => {
                // Handle rename events specially
                if event.paths.len() >= 2 {
                    let from_path = &event.paths[0];
                    let to_path = &event.paths[1];

                    if !Self::should_ignore(to_path, ignore_patterns) {
                        info!(
                            "Detected rename from: {} to: {}",
                            from_path.display(),
                            to_path.display()
                        );
                        Self::handle_file_rename(from_path, to_path, neo_db, file_tracker).await?;
                    }
                }
            }
            _ => {} // Ignore other event kinds
        }

        Ok(())
    }

    // Process debounced files (static method)
    async fn process_debounced_files(
        neo_db: &Arc<NeoDB>,
        parser: &Arc<Mutex<Parser>>,
        file_tracker: &Arc<Mutex<FileTracker>>,
        pending_changes: &Arc<Mutex<HashMap<PathBuf, Instant>>>,
    ) -> Result<(), String> {
        let now = Instant::now();
        let mut files_to_process = Vec::new();

        // Collect files that have been stable for the debounce period
        {
            let mut changes = pending_changes.lock().await;
            changes.retain(|path, last_changed| {
                if now.duration_since(*last_changed) >= Duration::from_millis(300) {
                    files_to_process.push(path.clone());
                    false // Remove from pending
                } else {
                    true // Keep in pending
                }
            });
        }

        // Process the stable files
        if !files_to_process.is_empty() {
            info!("Processing {} debounced files", files_to_process.len());

            // Update file tracker
            let mut tracker = file_tracker.lock().await;
            for path in &files_to_process {
                tracker.update_file(path);
            }

            // Parse files
            let mut parser_guard = parser.lock().await;
            for path in &files_to_process {
                if path.is_file() {
                    // Get the file extension
                    let extension = match path.extension() {
                        Some(ext) => ext.to_string_lossy().to_string(),
                        None => continue,
                    };

                    // Parse the file
                    match parser_guard.parse_single_file(path, &extension).await {
                        Ok(entities) => {
                            for entity in entities.items {
                                neo_db.ingest_entity(&entity).await;
                            }
                            info!("Updated file in graph: {}", path.display());
                        }
                        Err(e) => {
                            error!("Failed to parse file {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // Static helper methods
    fn should_ignore(path: &Path, ignore_patterns: &Vec<String>) -> bool {
        // Check if the path contains any ignored pattern
        let path_str = path.to_string_lossy();
        for pattern in ignore_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        // Check if it's a hidden file or directory
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            if file_name_str.starts_with('.') {
                return true;
            }
        }

        false
    }

    async fn handle_file_removal(
        path: &Path,
        neo_db: &Arc<NeoDB>,
        file_tracker: &Arc<Mutex<FileTracker>>,
    ) -> Result<(), String> {
        // Remove file and its entities from the database
        info!("Removing file from graph: {}", path.display());
        neo_db.remove_file(path).await;

        // Update file tracker
        let mut tracker = file_tracker.lock().await;
        tracker.remove_file(path);

        Ok(())
    }

    async fn handle_file_rename(
        from_path: &Path,
        to_path: &Path,
        neo_db: &Arc<NeoDB>,
        file_tracker: &Arc<Mutex<FileTracker>>,
    ) -> Result<(), String> {
        // Update the path in the database
        info!(
            "Updating file path in graph from: {} to: {}",
            from_path.display(),
            to_path.display()
        );
        neo_db.update_file_path(from_path, to_path).await;

        // Update file tracker
        let mut tracker = file_tracker.lock().await;
        tracker.rename_file(from_path, to_path);

        Ok(())
    }
    pub async fn process_events(&mut self) -> Result<(), String> {
        // Events are already being processed in the background task
        // This method exists just to provide compatibility with the AppState code
        Ok(())
    }
}
