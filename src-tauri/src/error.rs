use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tree-sitter error: {0}")]
    TreeSitter(String),

    #[error("Neo4j error: {0}")]
    Neo4j(#[from] neo4rs::Error),

    #[error("Parsing error: {0}")]
    Parse(String),

    #[error("Tauri error: {0}")]
    Tauri(String),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),

    #[error("Failed to parse environment variable {0}: {1}")]
    EnvVarParseError(String, String),
}

// Make AppError serializable for Tauri
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Type alias for Result with AppError
pub type Result<T> = std::result::Result<T, AppError>;