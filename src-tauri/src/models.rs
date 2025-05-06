use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub path: String,
    pub owner_id: String,
    pub name: Option<String>,
}

#[derive(Clone)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub hash: String,
    pub last_modified: SystemTime,
    pub size: u64,
}

#[derive(Clone, Serialize)]
pub struct FileChangePayload {
    pub repository_id: String,
    pub changed_files: Vec<String>,
    pub event_type: String,
}

#[derive(Clone, Debug)]
pub enum EntityType {
    File,
    Directory,
    Function,
    Method,
    Class,
    Struct,
    Interface,
    Trait,
    Enum,
    Variable,
    Parameter,
    CallSite,
    Import,
}

#[derive(Clone, Debug)]
pub struct CodeEntity {
    pub name: String,
    pub path: String,
    pub entity_type: EntityType,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub properties: HashMap<String, String>,
}
