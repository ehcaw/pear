use std::path::Path;
use tauri::{AppHandle, Emitter};
use tree_sitter::{Language, Node, Parser as TSParser, Query, QueryCursor};
use walkdir::WalkDir;

use crate::error::{AppError, Result};
use crate::file_manager::neo4j::NeoDB;
use crate::models::{CodeEntity, EntityType};
use crate::ts_queries;

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
        println!("Starting parse_and_ingest_directory for: {}", directory);
        let mut file_count = 0;

        // Create directory entity for the root directory
        let dir_path = Path::new(directory);
        println!("Processing root directory: {:?}", dir_path);

        let dir_name = dir_path
            .file_name()
            .ok_or_else(|| AppError::Parse("Invalid directory name".to_string()))?;

        println!("Root directory name: {:?}", dir_name);

        let dir_entity = CodeEntity {
            entity_type: EntityType::Directory,
            name: dir_name.to_string_lossy().to_string(),
            path: directory.to_string(),
            start_line: None,
            end_line: None,
            properties: std::collections::HashMap::new(),
        };

        println!("Created root directory entity: {:?}", dir_entity);

        // Ingest directory entity
        match neo_db.ingest_entity(&dir_entity).await {
            Ok(_) => println!("Successfully ingested root directory entity"),
            Err(e) => println!("Error ingesting root directory: {:?}", e),
        }

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

        println!(
            "Starting directory walk with ignore patterns: {:?}",
            ignore_dirs
        );

        // Walk directory recursively
        for entry in WalkDir::new(directory)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| {
                let file_name = e.file_name().to_string_lossy();
                let should_process = !ignore_dirs.iter().any(|d| &file_name == d);
                if !should_process {
                    println!("Ignoring directory/file: {}", file_name);
                }
                should_process
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            println!("Processing entry: {}", path.display());

            // Skip non-file entries
            if !path.is_file() {
                if path.is_dir() {
                    println!("Found directory: {}", path.display());
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

                        println!("Created directory entity: {:?}", dir_entity);

                        // Ingest directory entity
                        match neo_db.ingest_entity(&dir_entity).await {
                            Ok(_) => println!("Successfully ingested directory entity"),
                            Err(e) => println!("Error ingesting directory: {:?}", e),
                        }

                        // Create CONTAINS relationship
                        println!(
                            "Creating CONTAINS relationship: {} contains {}",
                            parent_path.display(),
                            path.display()
                        );

                        match neo_db
                            .create_relationship(
                                &parent_path.to_string_lossy().to_string(),
                                &path.to_string_lossy().to_string(),
                                "CONTAINS",
                            )
                            .await
                        {
                            Ok(_) => println!("Successfully created CONTAINS relationship"),
                            Err(e) => println!("Error creating relationship: {:?}", e),
                        }
                    }
                }
                continue;
            }

            // Handle files
            let extension = match path.extension() {
                Some(ext) => {
                    let ext_str = ext.to_string_lossy().to_string();
                    println!("Found file with extension: {}", ext_str);
                    ext_str
                }
                None => {
                    println!("Skipping file without extension: {}", path.display());
                    continue;
                }
            };

            // Determine language
            let language = CodeLanguage::from_extension(&extension);
            println!("Determined language for {}: {:?}", path.display(), language);

            // Skip unsupported languages
            if matches!(language, CodeLanguage::Unknown) {
                println!("Skipping unsupported language file: {}", path.display());
                continue;
            }

            // Parse file
            println!("Starting to parse file: {}", path.display());
            match self.parse_file(path, &language).await {
                Ok(entities) => {
                    println!(
                        "Successfully parsed file. Found {} entities",
                        entities.0.len()
                    );
                    // Create entities in Neo4j
                    for (i, entity) in entities.0.iter().enumerate() {
                        println!(
                            "Ingesting entity {}/{}: {:?}",
                            i + 1,
                            entities.0.len(),
                            entity
                        );
                        match neo_db.ingest_entity(entity).await {
                            Ok(_) => println!("Successfully ingested entity"),
                            Err(e) => println!("Error ingesting entity: {:?}", e),
                        }
                    }
                    file_count += 1;
                    println!("File count now: {}", file_count);
                }
                Err(e) => {
                    println!("Error parsing file {}: {:?}", path.display(), e);
                    app_handle
                        .emit(
                            "parse_error",
                            format!("Error parsing {}: {}", path.display(), e),
                        )
                        .unwrap();
                }
            }
        }

        println!("Finished processing. Total files processed: {}", file_count);
        Ok(file_count)
    }

    // Parse a single file
    async fn parse_file(
        &mut self,
        path: &Path,
        language: &CodeLanguage,
    ) -> Result<(
        Vec<CodeEntity>,
        Vec<(
            std::string::String,
            std::string::String,
            std::string::String,
        )>,
    )> {
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

        let extractor = Extractor::new(lang);
        let (mut entities, relations) = extractor.run(&tree, &content, &file_path);

        // File node first:
        entities.insert(
            0,
            CodeEntity {
                entity_type: EntityType::File,
                name: file_name,
                path: file_path.clone(),
                start_line: None,
                end_line: None,
                properties: {
                    let mut p = std::collections::HashMap::new();
                    p.insert("language".into(), format!("{:?}", language));
                    p.insert("lines".into(), content.lines().count().to_string());
                    p
                },
            },
        );

        Ok((entities, relations))
    }

    // Parse a single file with extension
    pub async fn parse_single_file(
        &mut self,
        path: &Path,
        extension: &str,
    ) -> Result<(
        Vec<CodeEntity>,
        Vec<(
            std::string::String,
            std::string::String,
            std::string::String,
        )>,
    )> {
        // Determine language from extension
        let language = CodeLanguage::from_extension(extension);

        // Skip unsupported languages
        if matches!(language, CodeLanguage::Unknown) {
            return Err(AppError::UnsupportedLanguage(format!(
                "Unsupported file extension: {}",
                extension
            )));
        }

        // Use the existing parse_file method
        self.parse_file(path, &language).await
    }
}

pub struct Extractor {
    query: Query,
}

impl Extractor {
    pub fn new(lang: Language) -> Self {
        let query = Query::new(lang, ts_queries::ENTITY_AND_DEP_QUERY).expect("bad TS query");
        Self { query }
    }

    // returns (entities, relationships)
    pub fn run(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        file_path: &str,
    ) -> (Vec<CodeEntity>, Vec<(String, String, String)>) {
        let mut entities = Vec::new();
        let mut relations = Vec::new();
        let mut qc = QueryCursor::new();

        let matches = qc.matches(&self.query, tree.root_node(), source.as_bytes());
        for m in matches {
            for cap in m.captures {
                let node = cap.node;
                let text = node.utf8_text(source.as_bytes()).unwrap_or("");
                let (row, col) = (
                    node.start_position().row + 1,
                    node.start_position().column + 1,
                );

                match self.query.capture_names()[cap.index as usize].as_str() {
                    // ── entities ────────────────────────────────────
                    "class.node" => {
                        entities.push(CodeEntity {
                            entity_type: EntityType::Class,
                            name: text.to_owned(),
                            path: file_path.to_owned(),
                            start_line: Some(row),
                            end_line: Some(node.end_position().row + 1),
                            properties: {
                                let mut p = std::collections::HashMap::new();
                                p.insert("column_start".into(), col.to_string());
                                p
                            },
                        });
                    }
                    "fn.node" => {
                        entities.push(CodeEntity {
                            entity_type: EntityType::Function,
                            name: text.to_owned(),
                            path: file_path.to_owned(),
                            start_line: Some(row),
                            end_line: Some(node.end_position().row + 1),
                            properties: {
                                let mut p = std::collections::HashMap::new();
                                p.insert("column_start".into(), col.to_string());
                                p
                            },
                        });
                    }
                    "iface.node" => {
                        entities.push(CodeEntity {
                            entity_type: EntityType::Interface,
                            name: text.to_owned(),
                            path: file_path.to_owned(),
                            start_line: Some(row),
                            end_line: Some(node.end_position().row + 1),
                            properties: {
                                let mut p = std::collections::HashMap::new();
                                p.insert("column_start".into(), col.to_string());
                                p
                            },
                        });
                    }

                    // ── deps  (edges are made once per stmt) ────────
                    "import.src" => {
                        let target = text.trim_matches('"').to_owned();
                        relations.push((file_path.to_owned(), target, "IMPORTS".to_owned()));
                    }
                    "export.src" => {
                        let target = text.trim_matches('"').to_owned();
                        relations.push((file_path.to_owned(), target, "EXPORTS".to_owned()));
                    }
                    _ => {}
                }
            }
        }
        (entities, relations)
    }
}
