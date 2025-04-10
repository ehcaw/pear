use moka::future::Cache;
use serde::Serialize;
use std::sync::OnceLock;
use std::thread;

#[derive(Debug, Serialize, Clone)]
pub struct CacheContent {
    path: String,
    content: String,
}

pub enum FileStatus {
    Active(String),
    Pending(String),
    Inactive(String),
}

static DB_CONTENT_CACHE: OnceLock<Cache<String, String>> = OnceLock::new();
pub fn get_db_content_cache() -> &'static Cache<String, String> {
    DB_CONTENT_CACHE.get_or_init(|| Cache::builder().max_capacity(100).build())
}
