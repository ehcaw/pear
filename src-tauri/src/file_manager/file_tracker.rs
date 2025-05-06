use glob::Pattern;
use std::collections::{HashMap, HashSet};

use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;
use tokio::fs;
use walkdir::{DirEntry, WalkDir};

#[derive(Error, Debug)]
pub enum FileTrackerError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Hash error: {0}")]
    HashError(String),

    #[error("Path error: {0}")]
    PathError(String),
}

pub struct FileMetadata {
    path: PathBuf,
    hash: String,
    last_modified: SystemTime,
    size: u64,
}

pub struct FileTracker {
    pub repository_path: PathBuf,
    pub repository_id: String,
    pub owner_id: String,
    file_metadata_map: HashMap<PathBuf, FileMetadata>,
    ignored_patterns: Vec<Pattern>,
}

impl FileTracker {
    pub fn new(repository_path: PathBuf, repository_id: String, owner_id: String) -> Self {
        let default_ignores = vec![
            glob::Pattern::new("**/node_modules/**").unwrap(),
            glob::Pattern::new("**/.git/**").unwrap(),
            glob::Pattern::new("**/target/**").unwrap(),
            glob::Pattern::new("**/dist/**").unwrap(),
            glob::Pattern::new("**/build/**").unwrap(),
        ];

        FileTracker {
            repository_path,
            repository_id,
            owner_id,
            file_metadata_map: HashMap::new(),
            ignored_patterns: default_ignores,
        }
    }
    pub async fn scan_repository(&mut self) -> Result<Vec<PathBuf>, FileTrackerError> {
        let mut changed_files = Vec::new();
        let mut current_files = HashSet::new();

        // First collect all paths (immutable borrow of self)
        let all_paths: Vec<_> = WalkDir::new(&self.repository_path)
            .into_iter()
            .filter_entry(|e| !self.should_ignore(e))
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect();

        // Then process them (now using mutable borrow)
        for path in all_paths {
            current_files.insert(path.clone());
            if self.has_file_changed(&path).await? {
                changed_files.push(path);
            }
        }

        Ok(changed_files)
    }

    fn should_ignore(&self, entry: &DirEntry) -> bool {
        let path = entry.path().to_string_lossy();

        for pattern in &self.ignored_patterns {
            if pattern.matches(&path) {
                return true;
            }
        }

        false
    }

    async fn has_file_changed(&mut self, path: &Path) -> Result<bool, FileTrackerError> {
        let metadata = fs::metadata(path).await?;
        let last_modified = metadata.modified()?;
        let size = metadata.len();

        if let Some(stored_metadata) = self.file_metadata_map.get(path) {
            // Quick check based on size and modification time
            if stored_metadata.size == size && stored_metadata.last_modified == last_modified {
                return Ok(false);
            }
        }

        // Calculate hash and compare
        let current_hash = self.calculate_file_hash(path).await?;
        if let Some(stored_metadata) = self.file_metadata_map.get(path) {
            if stored_metadata.hash == current_hash {
                return Ok(false);
            }
        }

        // Update metadata
        self.file_metadata_map.insert(
            path.to_path_buf(),
            FileMetadata {
                path: path.to_path_buf(),
                hash: current_hash,
                last_modified,
                size,
            },
        );

        Ok(true)
    }

    async fn calculate_file_hash(&self, path: &Path) -> Result<String, FileTrackerError> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut file, &mut buffer).await?;

        let hash = blake3::hash(&buffer);
        Ok(hash.to_hex().to_string())
    }

    // Method to reset tracking (useful for initial scan)
    pub async fn reset_tracking(&mut self) {
        self.file_metadata_map.clear();
    }
}
