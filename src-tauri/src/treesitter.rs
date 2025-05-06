use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor};

// Load the languages
extern "C" {
    fn tree_sitter_typescript() -> Language;
    fn tree_sitter_tsx() -> Language;
    fn tree_sitter_javascript() -> Language;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeItem {
    pub name: String,
    pub kind: String,
    pub start_line: usize,
    pub end_line: usize,
    pub children: Vec<CodeItem>,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileStructure {
    pub path: String,
    pub items: Vec<CodeItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryTree {
    pub name: String,
    pub children: Vec<DirectoryTree>,
    pub files: Vec<FileStructure>,
}

const IMPORTANT_EXTENSIONS: &[&str] = &[".js", ".jsx", ".ts", ".tsx", ".py"];

// Query patterns similar to your TypeScript implementation
const TS_BASE_QUERY: &str = r#"
(
    (class_declaration
        name: (type_identifier) @class.name
    ) @class.declaration

    (function_declaration
        name: (identifier) @function.name
    ) @function.declaration

    (interface_declaration
        name: (type_identifier) @interface.name
    ) @interface.declaration

    (export_statement
        (class_declaration
            name: (type_identifier) @class.name
        ) @class.declaration
    )

    (export_statement
        (function_declaration
            name: (identifier) @function.name
        ) @function.declaration
    )

    (export_statement
        (interface_declaration
            name: (type_identifier) @interface.name
        ) @interface.declaration
    )
)
"#;

struct ParserConfig {
    language: Language,
    query: Query,
}

pub struct TreeSitterParser {
    parser: Parser,
    configs: HashMap<String, ParserConfig>,
}

impl TreeSitterParser {
    pub fn new() -> Self {
        let parser = Parser::new();
        let mut configs = HashMap::new();

        // Pre-initialize configurations for supported languages
        let ts_lang = unsafe { tree_sitter_typescript() };
        let tsx_lang = unsafe { tree_sitter_tsx() };
        let js_lang = unsafe { tree_sitter_javascript() };

        // Create queries for each language
        let ts_query =
            Query::new(ts_lang, TS_BASE_QUERY).expect("Failed to create TypeScript query");
        let tsx_query = Query::new(tsx_lang, TS_BASE_QUERY).expect("Failed to create TSX query");
        let js_query =
            Query::new(js_lang, TS_BASE_QUERY).expect("Failed to create JavaScript query");

        configs.insert(
            "ts".to_string(),
            ParserConfig {
                language: ts_lang,
                query: ts_query,
            },
        );
        configs.insert(
            "tsx".to_string(),
            ParserConfig {
                language: tsx_lang,
                query: tsx_query,
            },
        );
        configs.insert(
            "js".to_string(),
            ParserConfig {
                language: js_lang,
                query: js_query,
            },
        );

        Self { parser, configs }
    }

    pub fn parse_file(&mut self, file_path: &Path) -> Result<FileStructure> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("Invalid file extension"))?;

        let config = self
            .configs
            .get(extension)
            .ok_or_else(|| anyhow!("Unsupported file type: {}", extension))?;

        let source_code = fs::read_to_string(file_path)?;

        self.parser.set_language(config.language)?;

        let tree = self
            .parser
            .parse(&source_code, None)
            .ok_or_else(|| anyhow!("Failed to parse file"))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&config.query, tree.root_node(), source_code.as_bytes());

        let mut items = Vec::new();
        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &config.query.capture_names()[capture.index as usize];

                if capture_name.ends_with(".declaration") {
                    let name = node
                        .child_by_field_name("name")
                        .map(|n| n.utf8_text(source_code.as_bytes()).unwrap_or(""))
                        .unwrap_or("")
                        .to_string();

                    let kind = if capture_name.starts_with("class") {
                        "class"
                    } else if capture_name.starts_with("function") {
                        "function"
                    } else if capture_name.starts_with("interface") {
                        "interface"
                    } else {
                        continue;
                    };

                    items.push(CodeItem {
                        name,
                        kind: kind.to_string(),
                        start_line: node.start_position().row + 1,
                        end_line: node.end_position().row + 1,
                        children: Vec::new(),
                        path: file_path.to_string_lossy().to_string(),
                    });
                }
            }
        }

        Ok(FileStructure {
            path: file_path.to_string_lossy().to_string(),
            items,
        })
    }
}
