use crate::error::{AppError, Result};
use crate::models::{CodeEntity, EntityType, FileStructure, LinkEntity, LinkType};

use log::info;
use neo4rs::{query, BoltType, Graph, Row};
use std::collections::HashMap;
use std::path::Path;

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

    pub fn match_entity_type(&self, entity: &EntityType) -> Result<&str> {
        let label = match entity {
            EntityType::Project => "Project",
            EntityType::File => "File",
            EntityType::Directory => "Directory",
            EntityType::Function => "Function",
            EntityType::Method => "Method",
            EntityType::Class => "Class",
            EntityType::Interface => "Interface",
            EntityType::Import => "Import",
        };
        Ok(label)
    }

    pub async fn new_simple(uri: String, user: String, password: String) -> Result<Self> {
        // Generate default IDs if not provided
        let repository_id = "default-repo".to_string();
        let owner_id = "default-owner".to_string();

        Self::new(&uri, &user, &password, repository_id, owner_id).await
    }

    // Setup Neo4j schema (constraints and indexes)
    async fn setup_schema(graph: &Graph) -> Result<()> {
        // Create constraints
        let mut file_constraint_stream = graph
            .execute(query(
                "CREATE CONSTRAINT file_path IF NOT EXISTS FOR (f:File) REQUIRE f.path IS UNIQUE",
            ))
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        while let Some(_) = file_constraint_stream
            .next()
            .await
            .map_err(|e| AppError::Neo4j(e))?
        {
            // We don't need to do anything with the results, just consume them
        }

        let mut directory_constraint_stream = graph.execute(query("CREATE CONSTRAINT directory_path IF NOT EXISTS FOR (d:Directory) REQUIRE d.path IS UNIQUE"))
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        while let Some(_) = directory_constraint_stream
            .next()
            .await
            .map_err(|e| AppError::Neo4j(e))?
        {
            // We don't need to do anything with the results, just consume them
        }
        // Create indexes
        let mut function_index_stream = graph
            .execute(query(
                "CREATE INDEX function_name IF NOT EXISTS FOR (f:Function) ON (f.name)",
            ))
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        while let Some(_) = function_index_stream
            .next()
            .await
            .map_err(|e| AppError::Neo4j(e))?
        {
            // We don't need to do anything with the results, just consume them
        }

        let mut class_index_stream = graph
            .execute(query(
                "CREATE INDEX class_name IF NOT EXISTS FOR (c:Class) ON (c.name)",
            ))
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        while let Some(_) = class_index_stream
            .next()
            .await
            .map_err(|e| AppError::Neo4j(e))?
        {
            // We don't need to do anything with the results, just consume them
        }

        let mut language_index_stream = graph
            .execute(query(
                "CREATE INDEX file_language IF NOT EXISTS FOR (f:File) ON (f.language)",
            ))
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        while let Some(_) = language_index_stream
            .next()
            .await
            .map_err(|e| AppError::Neo4j(e))?
        {
            // We don't need to do anything with the results, just consume them
        }

        Ok(())
    }

    pub async fn ingest_entity(&self, entity: &CodeEntity) -> Result<()> {
        let label = match entity.entity_type {
            EntityType::Project => "Project",
            EntityType::File => "File",
            EntityType::Directory => "Directory",
            EntityType::Function => "Function",
            EntityType::Method => "Method",
            EntityType::Class => "Class",
            EntityType::Interface => "Interface",
            EntityType::Import => "Import",
        };

        let cypher_query = format!(
            "MERGE (n:{} {{path: $path, start_line: $start_line, end_line: $end_line}}",
            label
        );

        let q = query(&cypher_query)
            .param("path", entity.path.clone())
            .param("start_line", entity.start_line.unwrap_or(0) as i64)
            .param("end_line", entity.end_line.unwrap_or(0) as i64);

        let mut result_stream = self
            .graph
            .execute(q)
            .await
            .map_err(|e| AppError::Neo4j(e))?;
        while let Some(_) = result_stream.next().await.map_err(|e| AppError::Neo4j(e))? {}
        Ok(())
    }

    pub async fn create_db_link(&self, link: &LinkEntity) -> Result<()> {
        let label = match link.link_type {
            LinkType::Has => "Has",
            LinkType::Owns => "Owns",
            LinkType::Uses => "Uses",
            LinkType::Import => "Imports",
        };
        let cypher_query = format!(
            "MATCH (source {{id: $source_id}})
        MATCH (target {{id: $target_id}})
        MERGE (source)-[r:$relationship_type]->(target)
        ON CREATE SET r.created_at = datetime(), r.property1 = $property1, r.property2 = $property2
        RETURN r"
        );

        let q = query(&cypher_query)
            .param("source_id", link.from_name.clone())
            .param("target_id", link.to_name.clone())
            .param("relationship_type", label);

        let mut result_stream = self
            .graph
            .execute(q)
            .await
            .map_err(|e| AppError::Neo4j(e))?;
        while let Some(_) = result_stream.next().await.map_err(|e| AppError::Neo4j(e))? {}
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
        let mut result_stream = self
            .graph
            .execute(q)
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        while let Some(_) = result_stream.next().await.map_err(|e| AppError::Neo4j(e))? {
            // We don't need to do anything with the results, just consume them
        }

        Ok(())
    }

    // Register a repository in Neo4j
    pub async fn register_repository(&self, repo_path: &str) -> Result<()> {
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
            id: repo_path.to_string(),
            path: repo_path.to_string(),
            entity_type: EntityType::Directory,
            start_line: None,
            end_line: None,
            properties: HashMap::from([
                ("repositoryId".to_string(), self.repository_id.clone()),
                ("ownerId".to_string(), self.owner_id.clone()),
                ("isRepository".to_string(), "true".to_string()),
            ]),
            children: Some(Vec::new()),
        };

        // Ingest the repository entity
        self.ingest_entity(&repo_entity).await?;

        info!("Registered repository: {}", repo_path);
        Ok(())
    }

    // Remove a file and all its entities from Neo4j
    pub async fn remove_file(&self, path: &std::path::Path) -> Result<()> {
        let file_path = path.to_string_lossy().to_string();

        // Delete the file node and all its contained entities
        let cypher = r#"
        MATCH (f:File {path: $path})
        OPTIONAL MATCH (f)-[:CONTAINS]->(entity)
        DETACH DELETE f, entity
        "#;

        let q = query(cypher).param("path", file_path.clone());

        self.graph
            .execute(q)
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        info!("Removed file from graph: {}", file_path);
        Ok(())
    }

    // Update file path when a file is renamed
    pub async fn update_file_path(
        &self,
        from_path: &std::path::Path,
        to_path: &std::path::Path,
    ) -> Result<()> {
        let old_path = from_path.to_string_lossy().to_string();
        let new_path = to_path.to_string_lossy().to_string();
        let new_name = to_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Update the file node path and name
        let cypher = r#"
        MATCH (f:File {path: $old_path})
        SET f.path = $new_path, f.name = $new_name
        "#;

        let q = query(cypher)
            .param("old_path", old_path.clone())
            .param("new_path", new_path.clone())
            .param("new_name", new_name);

        let mut result_stream = self
            .graph
            .execute(q)
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        while let Some(_) = result_stream.next().await.map_err(|e| AppError::Neo4j(e))? {
            // We don't need to do anything with the results, just consume them
        }

        // Update parent relationship if needed
        let new_parent = to_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""))
            .to_string_lossy()
            .to_string();

        if !new_parent.is_empty() {
            self.create_relationship(&new_parent, &new_path, "CONTAINS")
                .await?;
        }

        info!(
            "Updated file path in graph from {} to {}",
            old_path, new_path
        );
        Ok(())
    }

    // Check if a file exists in the graph
    pub async fn file_exists(&self, path: &std::path::Path) -> Result<bool> {
        let file_path = path.to_string_lossy().to_string();

        let cypher = "MATCH (f:File {path: $path}) RETURN count(f) as count";
        let q = query(cypher).param("path", file_path);

        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        if let Some(row) = result.next().await.map_err(|e| AppError::Neo4j(e))? {
            let count: i64 = row.get("count").unwrap_or(0);
            return Ok(count > 0);
        }

        Ok(false)
    }

    pub async fn batch_ingest_entities(&self, entities: &[CodeEntity]) -> Result<()> {
        if entities.is_empty() {
            return Ok(());
        }

        // Build UNWIND query for multiple node creation in one transaction
        let mut query_parts = vec!["UNWIND $entities AS entity"];
        query_parts.push("MERGE (n:Code {id: entity.id})");

        // Set common properties
        query_parts.push("SET n.path = entity.path");
        query_parts.push("SET n.updated_at = datetime()");
        query_parts.push("SET n:Entity"); // Base label for all entities

        // Set type-specific label
        query_parts.push("WITH n, entity");
        query_parts.push("CALL apoc.create.addLabels(n, [entity.type]) YIELD node");

        // Set other properties
        query_parts.push("SET node.start_line = entity.start_line");
        query_parts.push("SET node.end_line = entity.end_line");
        query_parts.push("SET node.name = entity.name");

        // Handle properties map
        query_parts.push("WITH node, entity");
        query_parts.push("UNWIND keys(entity.properties) AS key");
        query_parts.push("SET node[key] = entity.properties[key]");

        let cypher = query_parts.join("\n");

        // Prepare entities data
        let entity_data: Vec<BoltType> = entities
            .iter()
            .map(|e| {
                let mut m: HashMap<String, BoltType> = HashMap::new();
                m.insert("id".into(), e.id.clone().into());
                m.insert("path".into(), e.path.clone().into());

                // label
                let t = match e.entity_type {
                    EntityType::Project => "Project",
                    EntityType::Directory => "Directory",
                    EntityType::File => "File",
                    EntityType::Class => "Class",
                    EntityType::Interface => "Interface",
                    EntityType::Method => "Method",
                    EntityType::Function => "Function",
                    EntityType::Import => "Import",
                };
                m.insert("type".into(), t.into());

                if let Some(sl) = e.start_line {
                    m.insert("start_line".into(), (sl as i64).into());
                }
                if let Some(el) = e.end_line {
                    m.insert("end_line".into(), (el as i64).into());
                }

                let name = std::path::Path::new(&e.path)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| e.id.clone());
                m.insert("name".into(), name.into());

                let props: HashMap<String, BoltType> = e
                    .properties
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone().into()))
                    .collect();
                m.insert("properties".into(), props.into());

                m.into() // HashMap<String,BoltType> → BoltType
            })
            .collect();

        // Execute the query
        let q = query(&cypher).param("entities", entity_data);
        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        // Consume the results
        while let Some(_) = result.next().await.map_err(|e| AppError::Neo4j(e))? {}

        info!("Batch ingested {} entities", entities.len());
        Ok(())
    }

    // Batch create multiple relationships at once
    pub async fn batch_create_links(&self, links: &[LinkEntity]) -> Result<()> {
        if links.is_empty() {
            return Ok(());
        }

        // Build UNWIND query for multiple relationship creation
        let cypher = r#"
            UNWIND $links AS link
            MATCH (source {id: link.from_id})
            MATCH (target {id: link.to_id})
            CALL apoc.merge.relationship(source, link.type,
                {created_at: datetime()},
                {updated_at: datetime()},
                target)
            YIELD rel
            RETURN count(rel) as count
            "#;

        // Prepare link data
        let link_data: Vec<BoltType> = links
            .iter()
            .map(|l| {
                let mut m: HashMap<String, BoltType> = HashMap::new();
                m.insert("from_id".into(), l.from_name.clone().into());
                m.insert("to_id".into(), l.to_name.clone().into());
                let kind = match l.link_type {
                    LinkType::Has => "HAS",
                    LinkType::Owns => "OWNS",
                    LinkType::Uses => "USES",
                    LinkType::Import => "IMPORTS",
                };
                m.insert("rel_type".into(), kind.into());
                m.into()
            })
            .collect();

        // Execute the query
        let q = query(cypher).param("links", link_data);
        let mut result = self
            .graph
            .execute(q)
            .await
            .map_err(|e| AppError::Neo4j(e))?;

        // Get the count of created relationships
        let mut count = 0;
        if let Some(row) = result.next().await.map_err(|e| AppError::Neo4j(e))? {
            count = row.get::<i64>("count").unwrap_or(0);
        }

        info!("Created {} relationships in batch", count);
        Ok(())
    }

    // Process a file and create all necessary nodes and relationships
    pub async fn process_file_structure(&self, file_structure: &FileStructure) -> Result<()> {
        // First, create file node
        let file_path = &file_structure.file_path;
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.clone());

        let file_extension = std::path::Path::new(file_path)
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        // Create file entity
        let file_id = format!("file:{}", file_path);
        let file_entity = CodeEntity {
            id: file_id.clone(),
            path: file_path.clone(),
            entity_type: EntityType::File,
            start_line: None,
            end_line: None,
            properties: {
                let mut props = HashMap::new();
                props.insert("name".to_string(), file_name);
                props.insert("extension".to_string(), file_extension);
                props.insert("hash".to_string(), file_structure.file_hash.clone());
                props
            },
            children: None,
        };

        // Collect all entities and relationships
        let mut all_entities = vec![file_entity];
        let mut all_links = Vec::new();

        // Process items within the file
        for item in &file_structure.items {
            // Create entity for each item
            let item_id = if item.id.is_empty() {
                // unwrap the &str from match_entity_type() or bubble the error up
                let label = self.match_entity_type(&item.entity_type)?;
                // name := last path component, or fall back to autogenerated uuid
                let name = Path::new(&item.path)
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                format!("{}:{}", label.to_lowercase(), name)
            } else {
                item.id.clone()
            };

            let entity = CodeEntity {
                id: item_id.clone(),
                path: item.path.clone(),
                entity_type: item.entity_type.clone(),
                start_line: item.start_line,
                end_line: item.end_line,
                properties: item.properties.clone(),
                children: None,
            };

            all_entities.push(entity);

            // Create link from file to item
            let link = LinkEntity {
                from_name: file_id.clone(),
                to_name: item_id.clone(),
                link_type: LinkType::Has,
            };

            all_links.push(link);

            // If this is an import, create an IMPORTS relationship
            if matches!(item.entity_type, EntityType::Import) {
                if let Some(target_path) = item.properties.get("target") {
                    let import_link = LinkEntity {
                        from_name: file_id.clone(),
                        to_name: format!("file:{}", target_path),
                        link_type: LinkType::Import,
                    };
                    all_links.push(import_link);
                }
            }

            // Process any children
            if let Some(children) = &item.children {
                for child in children {
                    let child_id = if child.id.is_empty() {
                        let label = self.match_entity_type(&child.entity_type)?;
                        let name = Path::new(&child.path)
                            .file_name()
                            .map(|f| f.to_string_lossy().into_owned())
                            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                        format!("{}:{}", label.to_lowercase(), name)
                    } else {
                        child.id.clone()
                    };

                    let child_entity = CodeEntity {
                        id: child_id.clone(),
                        path: child.path.clone(),
                        entity_type: child.entity_type.clone(),
                        start_line: child.start_line,
                        end_line: child.end_line,
                        properties: child.properties.clone(),
                        children: None,
                    };

                    all_entities.push(child_entity);

                    // Create link from parent to child
                    let parent_child_link = LinkEntity {
                        from_name: item_id.clone(),
                        to_name: child_id,
                        link_type: LinkType::Has,
                    };

                    all_links.push(parent_child_link);
                }
            }
        }

        // Batch process everything
        self.batch_ingest_entities(&all_entities).await?;
        self.batch_create_links(&all_links).await?;

        // Create relationship between file and its directory
        let dir_path = std::path::Path::new(file_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        if !dir_path.is_empty() {
            let dir_id = format!("directory:{}", dir_path);

            // Create directory entity if it doesn't exist
            let dir_entity = CodeEntity {
                id: dir_id.clone(),
                path: dir_path.clone(),
                entity_type: EntityType::Directory,
                start_line: None,
                end_line: None,
                properties: {
                    let mut props = HashMap::new();
                    props.insert(
                        "name".to_string(),
                        std::path::Path::new(&dir_path)
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_else(|| dir_path.clone()),
                    );
                    props
                },
                children: None,
            };

            self.batch_ingest_entities(&[dir_entity]).await?;

            // Create CONTAINS relationship
            let contains_link = LinkEntity {
                from_name: dir_id,
                to_name: file_id,
                link_type: LinkType::Has,
            };

            self.batch_create_links(&[contains_link]).await?;
        }

        info!("Processed file structure for {}", file_path);
        Ok(())
    }

    // Process multiple file structures at once
    pub async fn batch_process_file_structures(&self, structures: &[FileStructure]) -> Result<()> {
        for structure in structures {
            self.process_file_structure(structure).await?;
        }

        info!("Processed {} file structures in batch", structures.len());
        Ok(())
    }
}
