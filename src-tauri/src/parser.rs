use crate::error::{AppError, Result};
use crate::file_manager::neo4j::NeoDB;
use crate::models::{CodeItem, EntityType, FileStructure, LinkEntity, LinkType};
use crate::ts_queries;
use queues::*;
use serde::Serialize;
use serde_json::Value;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use tree_sitter::{Language, Node, Parser as TSParser, Query, QueryCursor};
use ts_queries::ENTITY_AND_DEP_QUERY;
use walkdir::WalkDir;

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

    fn count_lines_in_file<P: AsRef<Path>>(&mut self, file_path: P) -> io::Result<usize> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;
        for _line in reader.lines() {
            line_count += 1;
        }
        Ok(line_count)
    }

    pub async fn parse_and_ingest_directory(
        &mut self,
        app_handle: &AppHandle,
        neo_db: &NeoDB,
        directory: &str,
    ) -> Result<(Vec<CodeItem>, Vec<LinkEntity>)> {
        let dir_path = Path::new(directory);
        // let dir_entity = CodeItem {
        //     id: dir_path.clone().to_string_lossy().to_string(),
        //     entity_type: EntityType::Project,
        //     name: dir_path.to_string_lossy().to_string(),
        //     path: directory.to_string(),
        //     start_line: Some(0),
        //     end_line: Some(0),
        //     properties: std::collections::HashMap::new(),
        //     children: Some(Vec::new()),
        // };
        let mut nodes: Vec<CodeItem> = Vec::new();
        let mut links: Vec<LinkEntity> = Vec::new();
        let mut q: Queue<PathBuf> = queue![dir_path.to_path_buf()];

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

        let mut gitignore = ignore::gitignore::GitignoreBuilder::new(directory);
        let gitignore_path = Path::new(directory).join(".gitignore");
        if gitignore_path.exists() {
            gitignore.add(gitignore_path);
        }
        let gitignore = gitignore
            .build()
            .unwrap_or_else(|_| ignore::gitignore::Gitignore::empty());

        while let Ok(curr_node) = q.remove() {
            let rel_path = match curr_node.strip_prefix(dir_path) {
                Ok(rel) => rel,
                Err(_) => &curr_node,
            };

            let path_str = rel_path.to_string_lossy();

            let should_ignore = ignore_dirs.iter().any(|&dir| {
                path_str.contains(&format!("/{}/", dir))
                    || path_str.starts_with(&format!("{}/", dir))
            });

            let gitignore_matched = gitignore.matched(rel_path, false).is_ignore();

            if should_ignore || gitignore_matched {
                continue;
            }

            if curr_node.is_dir() {
                match std::fs::read_dir(&curr_node) {
                    Ok(entries) => {
                        for entry in entries.flatten() {
                            let entry_path = entry.path();

                            // Get the path as a string for checking against ignore patterns
                            let path_str = entry_path.to_string_lossy();

                            // Create an owned PathBuf for the relative path
                            let rel_path = match entry_path.strip_prefix(dir_path) {
                                Ok(rel) => rel.to_path_buf(),
                                Err(_) => continue, // Skip if we can't get relative path
                            };

                            // Check if this path should be ignored
                            let should_ignore = ignore_dirs.iter().any(|&dir| {
                                path_str.contains(&format!("/{}/", dir))
                                    || path_str.starts_with(&format!("{}/", dir))
                            });

                            // Also check against .gitignore rules
                            let gitignore_matched = gitignore.matched(&rel_path, false).is_ignore();

                            if should_ignore || gitignore_matched {
                                println!("Skipping ignored path: {}", path_str);
                                continue;
                            }

                            // Add the path to the queue (using owned PathBuf)
                            q.add(entry_path.clone()).unwrap_or_else(|_| None);

                            let dir_node = CodeItem {
                                id: path_str.clone().to_string(),
                                entity_type: EntityType::Directory,
                                path: path_str.clone().to_string(),
                                start_line: Some(0),
                                end_line: Some(0),
                                properties: std::collections::HashMap::new(),
                                children: Some(Vec::new()),
                            };
                            nodes.push(dir_node);
                            if entry.path().is_file() {
                                links.push(LinkEntity {
                                    from_name: curr_node.to_string_lossy().to_string(),
                                    to_name: path_str.clone().to_string(),
                                    link_type: LinkType::Owns,
                                })
                            } else {
                                links.push(LinkEntity {
                                    from_name: curr_node.to_string_lossy().to_string(),
                                    to_name: path_str.clone().to_string(),
                                    link_type: LinkType::Has,
                                })
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading directory {}: {}", curr_node.display(), e);
                    }
                }
            } else if curr_node.is_file() {
                let extension = match dir_path.extension() {
                    Some(ext) => {
                        let ext_str = ext.to_string_lossy().to_string();
                        println!("Found file with extension: {}", ext_str);
                        ext_str
                    }
                    None => {
                        println!("Skipping file without extension: {}", dir_path.display());
                        continue;
                    }
                };
                // Determine language
                let language = CodeLanguage::from_extension(&extension);
                let file_breakdown = self.parse_file(&dir_path, &language).await.unwrap();
                let file_node = CodeItem {
                    id: curr_node.clone().to_string_lossy().to_string(),
                    entity_type: EntityType::Project,
                    path: dir_path.to_string_lossy().to_string(),
                    start_line: Some(0),
                    end_line: Some(self.count_lines_in_file(&dir_path).unwrap()),
                    properties: std::collections::HashMap::new(),
                    children: Some(file_breakdown),
                };
                for child in &file_node.children.clone().unwrap() {
                    match child.entity_type {
                        EntityType::Import => links.push(LinkEntity {
                            from_name: file_node.id.clone(),
                            to_name: child.id.clone(),
                            link_type: LinkType::Import,
                        }),
                        EntityType::Method => {}
                        EntityType::Function => {}
                        EntityType::Class => {}
                        _ => {}
                    }
                }
                nodes.push(file_node);
            }
        }

        Ok((nodes, links))
    }

    // Parse and ingest a directory
    pub async fn fparse_and_ingest_directory(
        &mut self,
        app_handle: &AppHandle,
        neo_db: &NeoDB,
        directory: &str,
    ) -> Result<usize> {
        println!(
            "Starting parse_and_ingestyeah im _directory for: {}",
            directory
        );
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

    async fn parse_file(&mut self, path: &Path, language: &CodeLanguage) -> Result<Vec<CodeItem>> {
        let content = std::fs::read_to_string(path).map_err(|e| AppError::Io(e))?;
        let mut children: Vec<CodeItem> = Vec::new();

        let lang = language.get_language()?;
        self.ts_parser
            .set_language(lang)
            .map_err(|e| AppError::TreeSitter((e.to_string())))?;

        let tree = self
            .ts_parser
            .parse(&content, None)
            .ok_or_else(|| AppError::Parse("Failed to parse file".to_string()))?;
        let root = tree.root_node();
        let source = content.as_bytes();

        let query = tree_sitter::Query::new(lang, ENTITY_AND_DEP_QUERY)
            .map_err(|e| AppError::TreeSitter(e.to_string()))?;

        let mut cursor = tree_sitter::QueryCursor::new();

        for (m, capture_idx) in cursor.captures(&query, root, source) {
            let node = &m.captures[capture_idx].node;
            let cap_name = &query.capture_names()[capture_idx];
            let start_line = node.start_position().row + 1;
            let end_line = node.end_position().row + 1;

            let name = match cap_name.as_str() {
                "import" => node.utf8_text(source).unwrap_or_default().to_string(),
                _ => {
                    // for class.name, function.name, method.name
                    node.utf8_text(source).unwrap_or_default().to_string()
                }
            };

            let entity_type = match cap_name.as_str() {
                "import" => EntityType::Import,
                "class" => EntityType::Class,
                "function" => EntityType::Function,
                "method" => EntityType::Method,
                _ => EntityType::Function,
            };

            children.push(CodeItem {
                id: name,
                path: path.to_string_lossy().to_string(),
                entity_type,
                start_line: Some(start_line),
                end_line: Some(end_line),
                properties: std::collections::HashMap::new(),
                children: None,
            });
        }

        Ok(children)
    }

    // Parse a single file
    async fn fparse_file(
        &mut self,
        path: &Path,
        language: &CodeLanguage,
    ) -> Result<(
        Vec<CodeItem>,
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
    ) -> Result<FileStructure> {
        // Check if file is TypeScript/TSX
        let is_typescript = extension == "ts" || extension == "tsx";

        // For now, only process TypeScript files
        if !is_typescript {
            return Err(AppError::UnsupportedLanguage(format!(
                "Only processing TypeScript files for now. Skipping file with extension: {}",
                extension
            )));
        }

        // Determine language from extension
        let language = CodeLanguage::from_extension(extension);

        match extension {
            "ts" => self
                .ts_parser
                .set_language(tree_sitter_typescript::language_typescript())
                .map_err(|e| AppError::TreeSitter(e.to_string()))?,
            "tsx" => self
                .ts_parser
                .set_language(tree_sitter_typescript::language_tsx())
                .map_err(|e| AppError::TreeSitter(e.to_string()))?,
            "js" | "jsx" => self
                .ts_parser
                .set_language(tree_sitter_javascript::language())
                .map_err(|e| AppError::TreeSitter(e.to_string()))?,
            "rs" => self
                .ts_parser
                .set_language(tree_sitter_rust::language())
                .map_err(|e| AppError::TreeSitter(e.to_string()))?,
            "py" => self
                .ts_parser
                .set_language(tree_sitter_python::language())
                .map_err(|e| AppError::TreeSitter(e.to_string()))?,
            _ => {
                return Err(AppError::UnsupportedLanguage(format!(
                    "Unsupported file extension: {}",
                    extension
                )))
            }
        }

        // Skip unsupported languages
        if matches!(language, CodeLanguage::Unknown) {
            return Err(AppError::UnsupportedLanguage(format!(
                "Unsupported file extension: {}",
                extension
            )));
        }

        let content = fs::read_to_string(path).map_err(AppError::Io)?;
        let file_hash = { format!("{:x}", md5::compute(content.as_bytes())) };

        // Use the existing parse_file method
        let (entities, _relations) = self.parse_file(path, &language).await.map_err(|e| e)?;
        let mut items: Vec<CodeItem> = Vec::new();
        for ent in entities.into_iter() {
            match ent.entity_type {
                EntityType::Function | EntityType::Method | EntityType::Class => {
                    let ci = CodeItem {
                        item_type: format!("{:?}", ent.entity_type).to_lowercase(),
                        name: ent.name.clone(),
                        start_line: ent.start_line.unwrap_or(0),
                        end_line: ent.end_line.unwrap_or(0),
                        children: None,
                    };
                    items.push(ci);
                }
                _ => {}
            }
        }
        let fs = FileStructure {
            structure_type: "file_structure".into(),
            file_path: path.to_string_lossy().into_owned(),
            items,
            file_hash,
        };
        Ok(fs)
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

    fn push_entity(
        &self,
        mut entities: Vec<CodeEntity>,
        et: EntityType,
        node: Node,
        name: &str,
        file_path: &str,
    ) {
        entities.push(CodeEntity {
            entity_type: et,
            name: name.to_owned(),
            path: file_path.to_owned(),
            start_line: Some(node.start_position().row + 1),
            end_line: Some(node.end_position().row + 1),
            properties: Default::default(),
        })
    }

    fn push_rel(
        &self,
        mut relations: Vec<(String, String, String)>,
        kind: &str,
        raw: &str,
        file_path: &str,
    ) {
        let target = raw.trim_matches('"').to_owned();
        relations.push((file_path.to_owned(), target, kind.to_owned()));
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
                // match self.query.capture_names()[cap.index as usize].as_str() {
                //     // ── entities ──
                //     "class.name" => self.push_entity(EntityType::Class, node, text),
                //     "iface.name" => push_entity(EntityType::Interface, node, text),
                //     "fn.name" => push_entity(EntityType::Function, node, text),
                //     "method.name" => push_entity(EntityType::Method, node, text),

                //     // ── deps ─────
                //     "import.src" => push_rel("IMPORTS", text),
                //     "export.src" => push_rel("EXPORTS", text),
                //     _ => {}
                // }

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
