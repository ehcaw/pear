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

#[derive(Clone, Debug, Serialize)]
pub enum EntityType {
    Project,
    Directory,
    File,
    Class,
    Interface,
    Method,
    Function,
    Import,
}

#[derive(Clone, Debug, Serialize)]
pub enum LinkType {
    Has,
    Owns,
    Uses,
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

#[derive(Clone, Debug)]
pub struct LinkEntity {
    pub from_name: String,
    pub to_name: String,
    pub link_type: LinkType,
}

#[derive(Clone, Debug, Serialize)]
pub struct CodeItem {
    #[serde(rename = "type")]
    pub id: String,
    pub path: String,
    pub entity_type: EntityType,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub properties: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<CodeItem>>,
}

#[derive(Serialize)]
pub struct FileStructure {
    #[serde(rename = "type")]
    pub structure_type: String,
    pub file_path: String,
    pub items: Vec<CodeItem>,
    pub file_hash: String,
}
