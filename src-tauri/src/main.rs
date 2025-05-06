// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Example of how to access environment variables with dotenvy
    // This is just for demonstration - the actual env loading happens in lib.rs
    
    // Load .env file
    match dotenvy::dotenv() {
        Ok(path) => println!("Loaded environment from: {}", path.display()),
        Err(e) => println!("Could not load .env file: {}", e),
    }
    
    // Access environment variables
    if let Ok(uri) = std::env::var("NEO4J_URI") {
        println!("Neo4j URI: {}", uri);
    }
    
    // Access with default fallback
    let repo_id = std::env::var("REPOSITORY_ID").unwrap_or_else(|_| "default-repo".to_string());
    println!("Repository ID: {}", repo_id);
    
    // Run the actual application
    pear_lib::run()
}
