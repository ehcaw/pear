use crate::error::Result;
use crate::env_utils;
use crate::file_manager::neo4j::NeoDB;
use std::path::PathBuf;
use log::info;

/// Example demonstrating how to use environment variables with dotenvy for Neo4j connection
pub async fn connect_with_env_vars() -> Result<()> {
    // Initialize environment variables from .env file
    env_utils::init()?;

    // Get Neo4j connection details from environment variables
    let neo4j_uri = env_utils::get_required("NEO4J_URI")?;
    let neo4j_user = env_utils::get_required("NEO4J_USER")?;
    let neo4j_password = env_utils::get_required("NEO4J_PASSWORD")?;
    
    // Get repository details from environment variables with defaults
    let repository_id = env_utils::get("REPOSITORY_ID")
        .unwrap_or_else(|| "default-repo".to_string());
    let owner_id = env_utils::get("OWNER_ID")
        .unwrap_or_else(|| "default-owner".to_string());

    // Optional: Get port number as integer
    let neo4j_port = env_utils::get_parsed::<u16>("NEO4J_PORT").unwrap_or(7687);
    info!("Using Neo4j port: {}", neo4j_port);

    // Connect to Neo4j
    let neo_db = NeoDB::new(
        &neo4j_uri,
        &neo4j_user,
        &neo4j_password,
        repository_id,
        owner_id,
    ).await?;

    info!("Successfully connected to Neo4j database");
    
    // Example of a complete .env file:
    //
    // NEO4J_URI=bolt://localhost:7687
    // NEO4J_USER=neo4j
    // NEO4J_PASSWORD=your_password
    // NEO4J_PORT=7687
    // REPOSITORY_ID=my-repository
    // OWNER_ID=my-owner

    Ok(())
}

/// Creates a sample .env file with Neo4j connection details
pub fn create_sample_env_file(path: Option<PathBuf>) -> Result<PathBuf> {
    let env_path = path.unwrap_or_else(|| PathBuf::from(".env"));
    
    let env_content = r#"# Neo4j Connection Details
NEO4J_URI=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=your_secure_password
NEO4J_PORT=7687

# Repository Information
REPOSITORY_ID=my-repo-id
OWNER_ID=my-owner-id
"#;

    std::fs::write(&env_path, env_content)?;
    info!("Created sample .env file at: {}", env_path.display());
    
    Ok(env_path)
}