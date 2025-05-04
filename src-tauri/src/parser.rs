use std::path::Path;
use tauri::{AppHandle, Emitter};
use tree_sitter::{Language, Node, Parser as TSParser};
use walkdir::WalkDir;

use crate::error::{AppError, Result};
use crate::neo4j::NeoDB;

// Define supported languages
#[derive(Debug)]
pub enum CodeLanguage {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Unknown,
}

impl CodeLanguage {
    fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => CodeLanguage::Rust,
            "js" => CodeLanguage::JavaScript,
            "jsx" => CodeLanguage::JavaScript,
            "ts" => CodeLanguage::TypeScript,
            "tsx" => CodeLanguage::TypeScript,
            "py" => CodeLanguage::Python,
            _ => CodeLanguage::Unknown,
        }
    }

    fn get_language(&self) -> Result<Language> {
        match self {
            CodeLanguage::Rust => Ok(tree_sitter_rust::language()),
            CodeLanguage::JavaScript => Ok(tree_sitter_javascript::language()),
            CodeLanguage::TypeScript => Ok(tree_sitter_typescript::language_typescript()),
            CodeLanguage::Python => Ok(tree_sitter_python::language()),
            CodeLanguage::Unknown => Err(AppError::UnsupportedLanguage(
                "Unknown language".to_string(),
            )),
        }
    }
}

// Code entity types
#[derive(Debug)]
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

// Code entity representation
#[derive(Debug)]
pub struct CodeEntity {
    pub entity_type: EntityType,
    pub name: String,
    pub path: String,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub properties: std::collections::HashMap<String, String>,
}

// Main Parser struct
pub struct Parser {
    ts_parser: TSParser,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            ts_parser: TSParser::new(),
        }
    }

    // Parse and ingest a directory
    pub async fn parse_and_ingest_directory(
        &mut self,
        app_handle: &AppHandle,
        neo_db: &NeoDB,
        directory: &str,
    ) -> Result<usize> {
        let mut file_count = 0;

        // Create directory entity for the root directory
        let dir_path = Path::new(directory);
        let dir_name = dir_path
            .file_name()
            .ok_or_else(|| AppError::Parse("Invalid directory name".to_string()))?;

        let dir_entity = CodeEntity {
            entity_type: EntityType::Directory,
            name: dir_name.to_string_lossy().to_string(),
            path: directory.to_string(),
            start_line: None,
            end_line: None,
            properties: std::collections::HashMap::new(),
        };

        // Ingest directory entity
        neo_db.ingest_entity(&dir_entity).await?;

        // Collect directories to ignore
        let ignore_dirs = [
            "node_modules",
            ".git",
            "target",
            "dist",
            "build",
            ".idea",
            ".vscode",
            "__pycache__",
            ".next",
            ".nuxt",
        ];

        // Walk directory recursively
        for entry in WalkDir::new(directory)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| {
                let file_name = e.file_name().to_string_lossy();
                !ignore_dirs.iter().any(|d| &file_name == d)
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip non-file entries
            if !path.is_file() {
                if path.is_dir() {
                    // Process directory entity
                    if let Some(parent_path) = path.parent() {
                        let dir_entity = CodeEntity {
                            entity_type: EntityType::Directory,
                            name: path.file_name().unwrap().to_string_lossy().to_string(),
                            path: path.to_string_lossy().to_string(),
                            start_line: None,
                            end_line: None,
                            properties: std::collections::HashMap::new(),
                        };

                        // Ingest directory entity
                        neo_db.ingest_entity(&dir_entity).await?;

                        // Create CONTAINS relationship
                        neo_db
                            .create_relationship(
                                &parent_path.to_string_lossy().to_string(),
                                &path.to_string_lossy().to_string(),
                                "CONTAINS",
                            )
                            .await?;
                    }
                }
                continue;
            }

            // Skip files without extension
            let extension = match path.extension() {
                Some(ext) => ext.to_string_lossy().to_string(),
                None => continue,
            };

            // Determine language
            let language = CodeLanguage::from_extension(&extension);

            // Skip unsupported languages
            if matches!(language, CodeLanguage::Unknown) {
                continue;
            }

            // Emit progress event
            let relative_path = path.strip_prefix(directory).unwrap_or(path);
            app_handle
                .emit(
                    "parse_progress",
                    format!("Processing {}", relative_path.display()),
                )
                .unwrap();

            // Parse file
            match self.parse_file(path, &language).await {
                Ok(entities) => {
                    // Create entities in Neo4j
                    for entity in entities {
                        neo_db.ingest_entity(&entity).await?;
                    }
                    file_count += 1;
                }
                Err(e) => {
                    // Emit error event
                    app_handle
                        .emit(
                            "parse_error",
                            format!("Error parsing {}: {}", path.display(), e),
                        )
                        .unwrap();
                }
            }
        }

        Ok(file_count)
    }

    // Parse a single file
    async fn parse_file(
        &mut self,
        path: &Path,
        language: &CodeLanguage,
    ) -> Result<Vec<CodeEntity>> {
        // Read file content
        let content = std::fs::read_to_string(path).map_err(|e| AppError::Io(e))?;

        // Set language for parser
        let lang = language.get_language()?;
        self.ts_parser
            .set_language(lang)
            .map_err(|e| AppError::TreeSitter(e.to_string()))?;

        // Parse the file
        let tree = self
            .ts_parser
            .parse(&content, None)
            .ok_or_else(|| AppError::Parse("Failed to parse file".to_string()))?;

        // Start with the file entity
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let file_path = path.to_string_lossy().to_string();
        let parent_path = path.parent().unwrap().to_string_lossy().to_string();

        let mut entities = vec![CodeEntity {
            entity_type: EntityType::File,
            name: file_name.clone(),
            path: file_path.clone(),
            start_line: None,
            end_line: None,
            properties: {
                let mut props = std::collections::HashMap::new();
                props.insert("language".to_string(), format!("{:?}", language));
                props
            },
        }];

        // Create CONTAINS relationship between parent directory and file
        // (This will be handled in the Neo4j module)

        // Parse the syntax tree to extract entities
        self.extract_entities(tree.root_node(), &content, &file_path, &mut entities)?;

        Ok(entities)
    }

    // Extract entities from the AST
    fn extract_entities(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        entities: &mut Vec<CodeEntity>,
    ) -> Result<()> {
        // This is a simplified implementation. In a complete implementation, you'd need to:
        // 1. Handle different node types based on the language
        // 2. Extract functions, classes, variables, etc.
        // 3. Track relationships between entities

        // For now, let's implement a basic version that just identifies functions
        if node.kind() == "function_definition" || node.kind() == "function_declaration" {
            // Extract function name
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" || child.kind() == "name" {
                    let name = child
                        .utf8_text(source.as_bytes())
                        .map_err(|e| AppError::Parse(e.to_string()))?;

                    let start_position = node.start_position();
                    let end_position = node.end_position();

                    entities.push(CodeEntity {
                        entity_type: EntityType::Function,
                        name: name.to_string(),
                        path: file_path.to_string(),
                        start_line: Some(start_position.row + 1),
                        end_line: Some(end_position.row + 1),
                        properties: std::collections::HashMap::new(),
                    });

                    break;
                }
            }
        } else if node.kind() == "class_declaration" || node.kind() == "struct_definition" {
            // Extract class/struct name
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" || child.kind() == "name" {
                    let name = child
                        .utf8_text(source.as_bytes())
                        .map_err(|e| AppError::Parse(e.to_string()))?;

                    let start_position = node.start_position();
                    let end_position = node.end_position();

                    let entity_type = if node.kind() == "class_declaration" {
                        EntityType::Class
                    } else {
                        EntityType::Struct
                    };

                    entities.push(CodeEntity {
                        entity_type,
                        name: name.to_string(),
                        path: file_path.to_string(),
                        start_line: Some(start_position.row + 1),
                        end_line: Some(end_position.row + 1),
                        properties: std::collections::HashMap::new(),
                    });

                    break;
                }
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_entities(child, source, file_path, entities)?;
        }

        Ok(())
    }
}
