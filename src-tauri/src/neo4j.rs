use neo4rs::{Graph, query};
use std::env;
use std::collections::HashMap;

use crate::error::{AppError, Result};
use crate::parser::{CodeEntity, EntityType};

pub struct NeoDB {
    graph: Graph,
}

impl NeoDB {
    pub async fn new() -> Result<Self> {
        // Get Neo4j connection details from environment variables
        let uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
        let user = env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
        let password = env::var("NEO4J_PASSWORD").expect("NEO4J_PASSWORD environment variable must be set");
        
        // Connect to Neo4j
        let graph = Graph::new(&uri, &user, &password)
            .await
            .map_err(|e| AppError::Neo4j(e))?;
        
        // Setup Neo4j schema constraints and indexes
        Self::setup_schema(&graph).await?;
        
        Ok(Self { graph })
    }
    
    // Setup Neo4j schema (constraints and indexes)
    async fn setup_schema(graph: &Graph) -> Result<()> {
        // Create constraints
        graph.execute(query("CREATE CONSTRAINT file_path IF NOT EXISTS FOR (f:File) REQUIRE f.path IS UNIQUE"))
            .await
            .map_err(|e| AppError::Neo4j(e))?;
        
        graph.execute(query("CREATE CONSTRAINT directory_path IF NOT EXISTS FOR (d:Directory) REQUIRE d.path IS UNIQUE"))
            .await
            .map_err(|e| AppError::Neo4j(e))?;
        
        // Create indexes
        graph.execute(query("CREATE INDEX function_name IF NOT EXISTS FOR (f:Function) ON (f.name)"))
            .await
            .map_err(|e| AppError::Neo4j(e))?;
        
        graph.execute(query("CREATE INDEX class_name IF NOT EXISTS FOR (c:Class) ON (c.name)"))
            .await
            .map_err(|e| AppError::Neo4j(e))?;
        
        graph.execute(query("CREATE INDEX file_language IF NOT EXISTS FOR (f:File) ON (f.language)"))
            .await
            .map_err(|e| AppError::Neo4j(e))?;
        
        Ok(())
    }
    
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
        let props_str = props_map.keys()
            .map(|k| format!("{}: ${}", k, k))
            .collect::<Vec<_>>()
            .join(", ");
        
        // Create Cypher query
        let cypher = format!("MERGE (n:{} {{ path: $path }}) ON CREATE SET n.{} ON MATCH SET n.{}", label, props_str, props_str);
        
        // Create a queryable with parameters
        let mut q = query(&cypher);
        for (k, v) in props_map {
            q = q.param(&k, v);
        }
        
        // Execute query
        self.graph.execute(q)
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
                ).await?;
            }
        }
        
        Ok(())
    }
    
    // Create a relationship between two entities
    pub async fn create_relationship(&self, from_path: &str, to_path: &str, rel_type: &str) -> Result<()> {
        // Determine node types based on the paths
        let from_type = if from_path.contains('.') { "File" } else { "Directory" };
        let to_type = if to_path.contains('.') { "File" } else { "Directory" };
        
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
        self.graph.execute(q)
            .await
            .map_err(|e| AppError::Neo4j(e))?;
        
        Ok(())
    }
}