use ignore::{
    gitignore::{Gitignore, GitignoreBuilder},
    Match,
};
use serde::Serialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Serialize, Debug, Clone)] // Add Clone if needed elsewhere
pub struct FileNodeRust {
    id: String, // Use path as id
    name: String,
    path: String,
    #[serde(rename = "type")]
    node_type: String, // "file" or "directory"
    children: Option<Vec<FileNodeRust>>,
}

/// Reads the entire content of a file into a string.
///
/// # Arguments
///
/// * `file_path` - A string slice (`&str`) representing the path to the file.
///
/// # Returns
///
/// * `Ok(String)` containing the file's content if the file is read successfully and is valid UTF-8.
/// * `Err(io::Error)` if the file cannot be found, cannot be opened due to permissions,
///   or if the content is not valid UTF-8.
#[tauri::command]
pub fn read_file_content(file_path: String) -> Result<String, String> {
    // <-- Return Result<String, String>
    let path = Path::new(&file_path); // Borrow the String to create a Path

    // Read the file to a string and map the error
    fs::read_to_string(path).map_err(|e| format!("Failed to read file '{}': {}", file_path, e))
    // <-- Map io::Error to String
}

// You might want to add some tests below (optional)
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_read_existing_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, Rust!").unwrap();

        let content = read_file_content(file_path.to_str().unwrap().to_string()).unwrap(); // Pass String
        assert_eq!(content.trim(), "Hello, Rust!");
    }

    #[test]
    fn test_read_non_existent_file() {
        let result = read_file_content("path/to/non/existent/file.txt".to_string()); // Pass String
        assert!(result.is_err());
        let err_msg = result.err().unwrap();
        assert!(err_msg.contains("Failed to read file"));
        assert!(err_msg.contains("path/to/non/existent/file.txt"));
    }
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
    return build_tree_recursive(&dir_path, &gitignore);
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
