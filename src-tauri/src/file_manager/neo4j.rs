use crate::error::{AppError, Result};
use crate::models::{CodeEntity, EntityType};

use log::{error, info};
use neo4rs::{query, Graph};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct NeoDB {
    graph: Graph,
    repository_id: String,
    owner_id: String,
}

impl NeoDB {
    pub async fn new(
        uri: &str,
        user: &str,
        password: &str,
        repository_id: String,
        owner_id: String,
    ) -> Result<Self> {
        // Connect to Neo4j
        let graph = Graph::new(uri, user, password)
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        // Setup Neo4j schema constraints and indexes
        Self::setup_schema(&graph).await?;

        Ok(NeoDB {
            graph,
            repository_id,
            owner_id,
        })
    }

    pub async fn new_simple(
        uri: String,
        user: String,
        password: String,
    ) -> Result<Self> {
        // Generate default IDs if not provided
        let repository_id = "default-repo".to_string();
        let owner_id = "default-owner".to_string();
        
        Self::new(&uri, &user, &password, repository_id, owner_id).await
    }

    // Setup Neo4j schema (constraints and indexes)
    async fn setup_schema(graph: &Graph) -> Result<()> {
        // Create constraints
        graph
            .execute(query(
                "CREATE CONSTRAINT file_path IF NOT EXISTS FOR (f:File) REQUIRE f.path IS UNIQUE",
            ))
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        graph.execute(query("CREATE CONSTRAINT directory_path IF NOT EXISTS FOR (d:Directory) REQUIRE d.path IS UNIQUE"))
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        // Create indexes
        graph
            .execute(query(
                "CREATE INDEX function_name IF NOT EXISTS FOR (f:Function) ON (f.name)",
            ))
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        graph
            .execute(query(
                "CREATE INDEX class_name IF NOT EXISTS FOR (c:Class) ON (c.name)",
            ))
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        graph
            .execute(query(
                "CREATE INDEX file_language IF NOT EXISTS FOR (f:File) ON (f.language)",
            ))
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        Ok(())
    }

    // pub async fn process_changed_files(
    //     &mut self,
    //     changed_files: Vec<PathBuf>,
    //     parser: &CodeParser,
    // ) -> Result<()> {
    //     for file_path in changed_files {
    //         info!("Processing file: {}", file_path.display());
    //         match parser.parse_file(&file_path).await {
    //             Ok(ast) => {
    //                 if let Err(e) = self.update_file_in_graph(&file_path, &ast).await {
    //                     error!("Failed to update file in graph: {}", e);
    //                 }
    //             }
    //             Err(e) => error!("Failed to parse file {}: {}", file_path.display(), e),
    //         }
    //     }

    //     Ok(())
    // }

    // async fn update_file_in_graph(&mut self, file_path: &Path, ast: &CodeAst) -> Result<()> {
    //     // Create the file entity
    //     let file_entity = CodeEntity {
    //         name: file_path
    //             .file_name()
    //             .unwrap_or_default()
    //             .to_string_lossy()
    //             .to_string(),
    //         path: file_path.to_string_lossy().to_string(),
    //         entity_type: EntityType::File,
    //         start_line: None,
    //         end_line: None,
    //         properties: HashMap::from([
    //             ("repositoryId".to_string(), self.repository_id.clone()),
    //             ("ownerId".to_string(), self.owner_id.clone()),
    //         ]),
    //     };

    //     // Ingest the file entity
    //     self.ingest_entity(&file_entity).await?;

    //     // Process all entities from the AST
    //     for entity in &ast.entities {
    //         self.ingest_entity(entity).await?;
    //     }

    //     Ok(())
    // }

    // Ingest a code entity into Neo4j
    pub async fn ingest_entity(&self, entity: &CodeEntity) -> Result<()> {
        let label = match entity.entity_type {
            EntityType::File => "File",
            EntityType::Directory => "Directory",
            EntityType::Function => "Function",
            EntityType::Method => "Method",
            EntityType::Class => "Class",
            EntityType::Struct => "Struct",
            EntityType::Interface => "Interface",
            EntityType::Trait => "Trait",
            EntityType::Enum => "Enum",
            EntityType::Variable => "Variable",
            EntityType::Parameter => "Parameter",
            EntityType::CallSite => "CallSite",
            EntityType::Import => "Import",
        };

        // Build base properties map
        let mut props_map = HashMap::new();
        props_map.insert("name".to_string(), entity.name.clone());
        props_map.insert("path".to_string(), entity.path.clone());

        // Add line numbers if available
        if let Some(start_line) = entity.start_line {
            props_map.insert("startLine".to_string(), start_line.to_string());
        }

        if let Some(end_line) = entity.end_line {
            props_map.insert("endLine".to_string(), end_line.to_string());
        }

        // Add custom properties
        for (key, value) in &entity.properties {
            props_map.insert(key.clone(), value.clone());
        }

        // Create Cypher query with parameters
        let props_str = props_map
            .keys()
            .map(|k| format!("{}: ${}", k, k))
            .collect::<Vec<_>>()
            .join(", ");

        // Create Cypher query
        let cypher = format!(
            "MERGE (n:{} {{ path: $path }}) ON CREATE SET n.{} ON MATCH SET n.{}",
            label, props_str, props_str
        );

        // Create a queryable with parameters
        let mut q = query(&cypher);
        for (k, v) in props_map {
            q = q.param(&k, v);
        }

        // Execute query
        self.graph
            .execute(q)
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        // If this is a File, create CONTAINS relationship with its directory
        if matches!(entity.entity_type, EntityType::File) {
            let file_path = std::path::Path::new(&entity.path);
            if let Some(parent) = file_path.parent() {
                self.create_relationship(
                    &parent.to_string_lossy().to_string(),
                    &entity.path,
                    "CONTAINS",
                )
                .await?;
            }
        }

        Ok(())
    }

    // Create a relationship between two entities
    pub async fn create_relationship(
        &self,
        from_path: &str,
        to_path: &str,
        rel_type: &str,
    ) -> Result<()> {
        // Determine node types based on the paths
        let from_type = if from_path.contains('.') {
            "File"
        } else {
            "Directory"
        };
        let to_type = if to_path.contains('.') {
            "File"
        } else {
            "Directory"
        };

        // Create Cypher query
        let cypher = format!(
            "MATCH (from:{} {{ path: $from_path }}), (to:{} {{ path: $to_path }}) MERGE (from)-[r:{}]->(to)",
            from_type, to_type, rel_type
        );

        // Create a queryable with parameters
        let q = query(&cypher)
            .param("from_path", from_path)
            .param("to_path", to_path);

        // Execute query
        self.graph
            .execute(q)
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        Ok(())
    }

    // Register a repository in Neo4j
    pub async fn register_repository(&self, repo_path: &str) -> Result<()> {
        use crate::models::{CodeEntity, EntityType};
        use std::collections::HashMap;
        use std::path::Path;

        let path = Path::new(repo_path);
        let repo_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Create repository entity
        let repo_entity = CodeEntity {
            name: repo_name,
            path: repo_path.to_string(),
            entity_type: EntityType::Directory,
            start_line: None,
            end_line: None,
            properties: HashMap::from([
                ("repositoryId".to_string(), self.repository_id.clone()),
                ("ownerId".to_string(), self.owner_id.clone()),
                ("isRepository".to_string(), "true".to_string()),
            ]),
        };

        // Ingest the repository entity
        self.ingest_entity(&repo_entity).await?;

        info!("Registered repository: {}", repo_path);
        Ok(())
    }
}
