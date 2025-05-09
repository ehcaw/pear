use crate::error::Result;
use blake3::Hash;
use log::{info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Structure to track files and their metadata
pub struct FileTracker {
    repository_path: PathBuf,
    repository_id: String,
    owner_id: String,
    file_hashes: HashMap<PathBuf, String>,
    file_timestamps: HashMap<PathBuf, SystemTime>,
}

impl FileTracker {
    /// Creates a new file tracker for a repository
    pub fn new(repository_path: PathBuf, repository_id: String, owner_id: String) -> Self {
        FileTracker {
            repository_path,
            repository_id,
            owner_id,
            file_hashes: HashMap::new(),
            file_timestamps: HashMap::new(),
        }
    }

    /// Initializes the tracker by scanning the repository
    pub fn initialize(&mut self) -> Result<()> {
        info!(
            "Initializing file tracker for repository: {}",
            self.repository_id
        );
        self.scan_repository()?;
        Ok(())
    }

    /// Scans the entire repository and builds the initial file index
    fn scan_repository(&mut self) -> Result<()> {
        // Implementation would walk through the repository and
        // compute hashes for all relevant files
        info!("Scanning repository: {}", self.repository_path.display());

        // This would be implemented with walkdir or similar
        // For now, we're just stubbing it

        Ok(())
    }

    /// Updates a file's metadata when it has changed
    pub fn update_file(&mut self, path: &Path) -> bool {
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified_time) = metadata.modified() {
                let was_updated = match self.file_timestamps.get(path) {
                    Some(old_time) => modified_time > *old_time,
                    None => true, // New file
                };

                if was_updated {
                    info!("Updating file: {}", path.display());
                    // Calculate hash
                    if let Ok(hash) = self.calculate_file_hash(path) {
                        self.file_hashes.insert(path.to_path_buf(), hash);
                        self.file_timestamps
                            .insert(path.to_path_buf(), modified_time);
                        return true;
                    }
                }
            }
        } else {
            warn!("Failed to get metadata for: {}", path.display());
        }
        false
    }

    /// Removes a file from tracking when it's deleted
    pub fn remove_file(&mut self, path: &Path) {
        info!("Removing file from tracking: {}", path.display());
        self.file_hashes.remove(path);
        self.file_timestamps.remove(path);
    }

    /// Updates tracking information when a file is renamed
    pub fn rename_file(&mut self, from_path: &Path, to_path: &Path) {
        info!(
            "Renaming file in tracking from {} to {}",
            from_path.display(),
            to_path.display()
        );

        if let Some(hash) = self.file_hashes.remove(from_path) {
            self.file_hashes.insert(to_path.to_path_buf(), hash);
        }

        if let Some(timestamp) = self.file_timestamps.remove(from_path) {
            self.file_timestamps
                .insert(to_path.to_path_buf(), timestamp);
        }
    }

    /// Gets the current hash for a file
    pub fn get_file_hash(&self, path: &Path) -> Option<&String> {
        self.file_hashes.get(path)
    }

    /// Checks if a file has changed since last update
    pub fn has_file_changed(&self, path: &Path) -> Result<bool> {
        if let Some(old_hash) = self.file_hashes.get(path) {
            let new_hash = self.calculate_file_hash(path)?;
            return Ok(old_hash != &new_hash);
        }

        // File not previously tracked
        Ok(true)
    }

    /// Gets all tracked files
    pub fn get_tracked_files(&self) -> Vec<PathBuf> {
        self.file_hashes.keys().cloned().collect()
    }

    /// Calculate hash for a file
    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        let content = fs::read(path)?;
        let hash = blake3::hash(&content);
        Ok(hash.to_hex().to_string())
    }
}
