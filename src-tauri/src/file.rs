use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::Match;
use ignore::WalkBuilder;
use serde::Serialize;
use serde_json::{from_str, Value};
use std::fs;
use std::path::{Path, PathBuf};

use futures::future::join_all;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::{mpsc as std_mpsc, Arc};
use std::time::Duration;
use tokio::sync::mpsc as tokio_mpsc;

use crate::cache;
use crate::supabase;
// Keep TreeCursor import if needed elsewhere, otherwise remove. Removing for now.

#[derive(Serialize, Debug, Clone)] // Add Clone if needed elsewhere
pub struct FileNodeRust {
    id: String, // Use path as id
    name: String,
    path: String,
    #[serde(rename = "type")]
    node_type: String, // "file" or "directory"
    children: Option<Vec<FileNodeRust>>,
}

#[tauri::command]
pub fn read_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
}

// Recursive helper function to build the file tree respecting .gitignore
fn build_tree_recursive(
    dir_path: &Path,
    gitignore: &Gitignore,
) -> Result<Vec<FileNodeRust>, String> {
    let mut nodes = Vec::new();

    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory {}: {}", dir_path.display(), e))?;

    for entry_result in entries {
        let entry = entry_result.map_err(|e| format!("Error reading directory entry: {}", e))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Check if the path should be ignored BEFORE processing further
        let is_dir = path.is_dir();
        match gitignore.matched(&path, is_dir) {
            Match::Ignore(_) => continue, // Skip ignored files/dirs
            _ => {}                       // Process if Whitelisted or None
        }

        let path_str = path.to_string_lossy().to_string();
        let id = path_str.clone(); // Use path as ID

        if is_dir {
            // Recursively build children for directories
            match build_tree_recursive(&path, gitignore) {
                Ok(children) => {
                    // Only add directory if it (or its subdirectories) contain non-ignored files
                    if !children.is_empty() {
                        nodes.push(FileNodeRust {
                            id,
                            name,
                            path: path_str,
                            node_type: "directory".to_string(),
                            children: Some(children),
                        });
                    } else {
                        // Optionally, include empty directories if needed by changing the logic here
                        // For now, we skip adding fully ignored/empty directories
                    }
                }
                Err(e) => {
                    // Log error for subdirectory and continue with others
                    eprintln!("Error building tree for {}: {}", path.display(), e);
                    // Optionally collect these errors to return them
                }
            }
        } else {
            // It's a file, add it
            nodes.push(FileNodeRust {
                id,
                name,
                path: path_str,
                node_type: "file".to_string(),
                children: None,
            });
        }
    }

    // Optional: Sort entries (folders first, then by name)
    nodes.sort_by(|a, b| match (a.node_type.as_str(), b.node_type.as_str()) {
        ("directory", "file") => std::cmp::Ordering::Less,
        ("file", "directory") => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(nodes)
}

// Tauri command to read directory structure respecting .gitignore
#[tauri::command]
pub fn read_directory_structure(dir_path_str: String) -> Result<Vec<FileNodeRust>, String> {
    let dir_path = PathBuf::from(dir_path_str);
    if !dir_path.is_dir() {
        return Err(format!(
            "Path is not a valid directory: {}",
            dir_path.display()
        ));
    }

    // --- .gitignore Handling ---
    let mut builder = GitignoreBuilder::new(&dir_path);
    // Add standard .gitignore file if present in the root
    let gitignore_path = dir_path.join(".gitignore");
    if gitignore_path.exists() {
        builder.add(gitignore_path);
    }
    // You could add other standard ignore files like .git/info/exclude if needed
    // builder.add(dir_path.join(".git/info/exclude"));

    // Build the ignore matcher
    let gitignore = builder
        .build()
        .map_err(|e| format!("Failed to build gitignore rules: {}", e))?;

    // Start the recursive build
    build_tree_recursive(&dir_path, &gitignore)
}

#[derive(Debug, Clone)] // Added Clone
pub struct CodeChunk {
    file_path: String,
    name: String,       // function/class/module name etc.
    chunk_type: String, // "class", "function", "method", "interface", "enum", "type", "variable", "module", "file" etc.
    content: String,    // The actual code content
    start_line: usize,
    end_line: usize,
}

// Make this function async
async fn embed_file_data(file_data: &CodeChunk) {
    match supabase::call_embedding_function_async(&file_data.content).await {
        Ok(response_body) => {
            // --- Parse the JSON response and extract the embedding array ---
            let extracted_embedding_string: Result<String, String> =
                match from_str::<Value>(&response_body) {
                    // Parse the response string into a generic JSON Value
                    Ok(json_value) => {
                        // Attempt to get the "embedding" field and check if it's an array
                        if let Some(embedding_value) = json_value.get("embedding") {
                            if embedding_value.is_array() {
                                // Convert the embedding Value back into a JSON string (just the array part)
                                serde_json::to_string(embedding_value).map_err(|e| {
                                    format!("Failed to re-serialize embedding array: {}", e)
                                })
                            } else {
                                Err(format!("'embedding' field in response is not a JSON array"))
                            }
                        } else {
                            Err(format!("Response JSON does not contain 'embedding' field"))
                        }
                    }
                    Err(e) => Err(format!(
                        "Failed to parse edge function response as JSON: {}",
                        e
                    )),
                };

            // --- Proceed only if extraction was successful ---
            match extracted_embedding_string {
                Ok(embedding_array_string) => {
                    // Now pass the extracted array string to insert_embedding
                    match supabase::insert_embedding(
                        file_data.file_path.clone(),
                        file_data.content.clone(),
                        embedding_array_string, // Pass the extracted array string
                    )
                    .await
                    {
                        Ok(_) => println!("DB insert successful for {}", file_data.file_path),
                        Err(e) => eprintln!(
                            "Error inserting embedding into DB for {}: {}",
                            file_data.file_path, e
                        ),
                    }
                }
                Err(e) => {
                    // Log the extraction error
                    eprintln!(
                        "Failed to extract embedding for {}: {}",
                        file_data.file_path, e
                    );
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Error calling embedding function for {}: {}",
                file_data.file_path, e
            );
        }
    }
}

pub async fn embed_single_file(file_path: &str) -> Result<(), String> {
    let path = Path::new(file_path);
    if path.is_file() {
        match fs::read_to_string(path) {
            Ok(content) => {
                if content.chars().any(|c| c == '\0') {
                    return Err("Likely binary file".to_string());
                }
                if content.len() > 5_000_000 {
                    return Err("File contents too long to embed".to_string());
                }
                if content.is_empty() {
                    return Err("Empty file".to_string());
                }
                let file_name = path.file_name().map_or_else(
                    || "unknown_file".to_string(),
                    |n| n.to_str().unwrap_or("invalid_filename").to_string(),
                );
                let file_chunk = CodeChunk {
                    file_path: path.to_string_lossy().to_string(),
                    name: file_name,
                    chunk_type: "file".to_string(),
                    content: content.clone(),
                    start_line: 1,
                    end_line: content.lines().count().max(1),
                };
                embed_file_data(&file_chunk);
            }
            Err(e) => {
                (eprintln!("Error reading file {}: {}", path.display(), e));
            }
        }
    }
    Ok(())
}

// --- embed_codebase remains async and calls embed_file_data.await ---
#[tauri::command]
pub async fn embed_codebase(dir_path_str: String) -> Result<String, String> {
    // ... (file walking logic remains the same) ...

    let mut files_to_embed: Vec<CodeChunk> = /* ... collect files ... */ Vec::new(); // Placeholder
    let mut processed_files = 0;
    let mut errors: Vec<String> = Vec::new();

    // --- File Collection (Synchronous) ---
    let base_path = PathBuf::from(dir_path_str); // Added definition
    if !base_path.is_dir() {
        // Added check
        return Err(format!("Invalid directory path: {}", base_path.display()));
    }
    for result in WalkBuilder::new(&base_path).build() {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    match fs::read_to_string(path) {
                        // blocking read
                        Ok(content) => {
                            if content.chars().any(|c| c == '\0') {
                                eprintln!("Skipping likely binary file: {}", path.display());
                                continue;
                            }
                            if content.len() > 5_000_000 {
                                eprintln!(
                                    "Skipping large file ({} bytes): {}",
                                    content.len(),
                                    path.display()
                                );
                                continue;
                            }
                            if content.is_empty() {
                                continue;
                            }

                            let file_name = path.file_name().map_or_else(
                                || "unknown_file".to_string(),
                                |n| n.to_str().unwrap_or("invalid_filename").to_string(),
                            );

                            let file_chunk = CodeChunk {
                                file_path: path.to_string_lossy().to_string(),
                                name: file_name,
                                chunk_type: "file".to_string(),
                                content: content.clone(),
                                start_line: 1,
                                end_line: content.lines().count().max(1),
                            };

                            files_to_embed.push(file_chunk);
                            processed_files += 1;
                        }
                        Err(e) => {
                            let error_msg = format!("Error reading file {}: {}", path.display(), e);
                            eprintln!("{}", error_msg);
                            errors.push(error_msg);
                        }
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Error walking directory: {}", e);
                eprintln!("{}", error_msg);
                errors.push(error_msg);
            }
        }
    }

    let total_files = files_to_embed.len();
    println!(
        "Finished walking directory. Found {} files to embed.",
        total_files
    );

    if !errors.is_empty() {
        eprintln!(
            "Encountered {} errors during file reading:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }

    // --- Embedding Step (Asynchronous) ---
    println!("Starting embedding of {} files...", total_files);

    let (sender, mut receiver) = tokio_mpsc::channel(100); // Buffer size of 100
    let receiver = Arc::new(tokio::sync::Mutex::new(receiver));

    for file_data in files_to_embed {
        embed_file_data(&file_data).await; // Calls the async embed_file_data
    }
    drop(sender);

    let num_workers = 10;
    let mut handles = Vec::new();
    for _ in 0..num_workers {
        let receiver_clone = receiver.clone();
        let handle = tokio::spawn(async move {
            loop {
                let file = {
                    let mut receiver = receiver_clone.lock().await;
                    match receiver.recv().await {
                        Some(file) => file,
                        None => break,
                    }
                };
                embed_file_data(&file).await;
            }
        });
        handles.push(handle);
    }

    join_all(handles).await;

    println!("Embedding process completed.");

    Ok(format!(
        "Processed {} files. Attempted embedding for {}. {} read errors encountered.",
        processed_files,
        total_files,
        errors.len()
    ))
}

pub async fn watch_directory(dir_path: &str) -> Result<(), String> {
    let path = Path::new(dir_path);
    let (tx, rx) = std_mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                tx.send(event)
                    .unwrap_or_else(|e| eprintln!("Error sending event : {}", e))
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to create watcher : {}", e))?;
    watcher
        .watch(path, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch path: {}", e))?;
    let content_cache = cache::get_db_content_cache();
    for event in rx {
        let paths = event.paths;
        match event.kind {
            EventKind::Create(_) => {
                for path in paths {
                    embed_single_file(path.to_str().unwrap()).await;
                }
            }
            EventKind::Modify(_) => {
                // Handle file/directory modification
                println!("Modified: {:?}", paths);
            }
            EventKind::Remove(_) => {
                // Handle file/directory deletion
                println!("Removed: {:?}", paths);
            }
            _ => {}
        }
    }
    Ok(())
}
