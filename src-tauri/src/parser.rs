use crate::error::{AppError, Result};
use crate::file_manager::neo4j::NeoDB;
use crate::models::{CodeEntity, CodeLanguage, EntityType, FileStructure, LinkEntity, LinkType};
use crate::ts_queries;
use queues::*;
use std::fs::{read_to_string, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tree_sitter::{Language, Parser as TSParser};
use ts_queries::{JS_ENTITY_AND_DEP_QUERY, TS_ENTITY_AND_DEP_QUERY};

// Define supported languages

impl CodeLanguage {
    fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "js" => CodeLanguage::JavaScript,
            "jsx" => CodeLanguage::Jsx,
            "ts" => CodeLanguage::TypeScript,
            "tsx" => CodeLanguage::Tsx,
            _ => CodeLanguage::Unknown,
        }
    }

    fn get_language(&self) -> Result<Language> {
        match self {
            CodeLanguage::JavaScript => Ok(tree_sitter_javascript::language()),
            CodeLanguage::TypeScript => Ok(tree_sitter_typescript::language_typescript()),
            CodeLanguage::Tsx => Ok(tree_sitter_typescript::language_tsx()),
            CodeLanguage::Jsx => Ok(tree_sitter_javascript::language()),
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
        directory: &str,
    ) -> Result<(Vec<CodeEntity>, Vec<LinkEntity>)> {
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
        let mut nodes: Vec<CodeEntity> = Vec::new();
        let mut links: Vec<LinkEntity> = Vec::new();
        let mut q: std::collections::VecDeque<PathBuf> = std::collections::VecDeque::new();
        q.push_back(dir_path.to_path_buf());

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

        while let Some(curr_node) = q.pop_front() {
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
                let dir_node = CodeEntity {
                    id: curr_node.to_string_lossy().to_string(),
                    entity_type: EntityType::Directory,
                    start_line: Some(0),
                    end_line: Some(0),
                    path: curr_node.to_string_lossy().to_string(),
                    properties: std::collections::HashMap::new(),
                    children: Some(Vec::new()),
                };
                nodes.push(dir_node);
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
                            q.push_back(entry_path.clone());
                            if entry.path().is_file() {
                                links.push(LinkEntity {
                                    from_name: curr_node.to_string_lossy().to_string(),
                                    to_name: path_str.clone().to_string(),
                                    link_type: LinkType::Owns,
                                });
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
                let extension = match curr_node.extension() {
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
                if matches!(language, CodeLanguage::Unknown) {
                    continue;
                }

                // for child in &file_node.children.clone().unwrap() {
                //     match child.entity_type {
                //         EntityType::Import => links.push(LinkEntity {
                //             from_name: file_node.id.clone(),
                //             to_name: child.id.clone(),
                //             link_type: LinkType::Import,
                //         }),
                //         EntityType::Method => {}
                //         EntityType::Function => {}
                //         EntityType::Class => {}
                //         _ => {}
                //     }
                // }
                match self.parse_file(&curr_node, &language).await {
                    Ok(file_breakdown) => {
                        let file_node = CodeEntity {
                            id: curr_node.to_string_lossy().to_string(),
                            entity_type: EntityType::File,
                            path: curr_node.to_string_lossy().to_string(),
                            start_line: Some(0),
                            end_line: Some(self.count_lines_in_file(&curr_node).unwrap_or(0)),
                            properties: std::collections::HashMap::new(),
                            children: Some(file_breakdown),
                        };
                        if let Some(children) = &file_node.children {
                            for child in children {
                                match child.entity_type {
                                    EntityType::Import => links.push(LinkEntity {
                                        from_name: file_node.id.clone(),
                                        to_name: child.id.clone(),
                                        link_type: LinkType::Import,
                                    }),
                                    EntityType::Method => links.push(LinkEntity {
                                        from_name: file_node.id.clone(),
                                        to_name: child.id.clone(),
                                        link_type: LinkType::Uses,
                                    }),
                                    EntityType::Function => links.push(LinkEntity {
                                        from_name: file_node.id.clone(),
                                        to_name: child.id.clone(),
                                        link_type: LinkType::Uses,
                                    }),
                                    EntityType::Class => links.push(LinkEntity {
                                        from_name: file_node.id.clone(),
                                        to_name: child.id.clone(),
                                        link_type: LinkType::Owns,
                                    }),
                                    _ => {}
                                }
                                nodes.push(child.to_owned());
                            }
                        }
                        nodes.push(file_node);
                    }
                    Err(e) => {
                        println!("failed to parse file {}: {:?}", curr_node.display(), e);
                    }
                }
            }
        }
        println!("Finished processing");
        Ok((nodes, links))
    }

    async fn parse_file(
        &mut self,
        path: &Path,
        language: &CodeLanguage,
    ) -> Result<Vec<CodeEntity>> {
        let content = std::fs::read_to_string(path).map_err(|e| AppError::Io(e))?;
        let mut children: Vec<CodeEntity> = Vec::new();

        println!("{}", path.to_string_lossy().to_string());

        let lang = language.get_language()?;
        self.ts_parser
            .set_language(lang)
            .map_err(|e| AppError::TreeSitter(e.to_string()))?;

        let tree = self
            .ts_parser
            .parse(&content, None)
            .ok_or_else(|| AppError::Parse("Failed to parse file".to_string()))?;
        let root = tree.root_node();
        let source = content.as_bytes();

        let query_str = match language {
            CodeLanguage::TypeScript => TS_ENTITY_AND_DEP_QUERY,
            CodeLanguage::JavaScript => JS_ENTITY_AND_DEP_QUERY, // (define this separately)
            CodeLanguage::Tsx => TS_ENTITY_AND_DEP_QUERY,
            CodeLanguage::Jsx => JS_ENTITY_AND_DEP_QUERY,
            _ => {
                return Err(AppError::UnsupportedLanguage(
                    "No query defined for this language".to_string(),
                ));
            }
        };

        let query = tree_sitter::Query::new(lang, query_str)
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

            children.push(CodeEntity {
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

        let content = read_to_string(path).map_err(AppError::Io)?;
        let file_hash = { format!("{:x}", md5::compute(content.as_bytes())) };

        // Use the existing parse_file method
        let parse_result = self.parse_file(path, &language).await.map_err(|e| e)?;
        let mut items: Vec<CodeEntity> = Vec::new();
        for ent in parse_result.into_iter() {
            items.push(ent);
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
