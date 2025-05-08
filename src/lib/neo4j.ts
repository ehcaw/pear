import * as neo4j from "neo4j-driver";
import type { CodeGraphData, SearchResult } from "./types";

export class Neo4jDriver {
  private uri: string;
  private username: string;
  private password: string;
  private driver: neo4j.Driver | null;

  constructor(uri: string, user: string, password: string) {
    this.uri = uri;
    this.username = user;
    this.password = password;
    this.driver = neo4j.driver(
      this.uri,
      neo4j.auth.basic(this.username, this.password),
    );
  }
  private getDriver(): neo4j.Driver {
    if (!this.driver) {
      this.driver = neo4j.driver(
        this.uri,
        neo4j.auth.basic(this.username, this.password),
      );
    }
    return this.driver;
  }

  // Close the driver when done
  async close(): Promise<void> {
    if (this.driver) {
      await this.driver.close();
      this.driver = null;
    }
  }

  /**
   * Performs a fuzzy search using Neo4j's APOC procedures or fallback to contains
   * @param searchTerm The text to search for
   * @param entityTypes Optional array of entity types to restrict search to
   * @param limit Maximum number of results to return
   */
  async fuzzySearch(
    searchTerm: string,
    entityTypes?: string[],
    limit: number = 20,
  ): Promise<SearchResult[]> {
    const driver = this.getDriver();
    const session = driver.session();

    try {
      // Build entity type filter
      const typeFilter =
        entityTypes && entityTypes.length > 0
          ? `(n:${entityTypes.join("|")})`
          : "(n)";

      // First, try checking if APOC is available
      const apocCheck = await session.run(
        "CALL dbms.procedures() YIELD name WHERE name STARTS WITH 'apoc' RETURN count(*) > 0 as hasApoc",
      );
      const hasApoc = apocCheck.records[0].get("hasApoc");

      let query;
      if (hasApoc) {
        // Use APOC fuzzy matching if available
        query = `
          MATCH ${typeFilter}
          WHERE apoc.text.fuzzyMatch(n.name, $searchTerm) > 0.5
             OR apoc.text.fuzzyMatch(n.path, $searchTerm) > 0.5
          RETURN n.name as name, n.path as path, labels(n)[0] as type,
                 n.startLine as startLine, n.endLine as endLine,
                 apoc.text.fuzzyMatch(n.name, $searchTerm) + apoc.text.fuzzyMatch(n.path, $searchTerm) as score
          ORDER BY score DESC
          LIMIT $limit
          `;
      } else {
        // Fallback to simple contains search
        query = `
          MATCH ${typeFilter}
          WHERE toLower(n.name) CONTAINS toLower($searchTerm)
             OR toLower(n.path) CONTAINS toLower($searchTerm)
          RETURN n.name as name, n.path as path, labels(n)[0] as type,
                 n.startLine as startLine, n.endLine as endLine
          LIMIT $limit
          `;
      }

      const result = await session.run(query, {
        searchTerm,
        limit: neo4j.int(limit),
      });

      return result.records.map((record) => ({
        name: record.get("name"),
        path: record.get("path"),
        type: record.get("type")?.toLowerCase() || "unknown",
        startLine: record.has("startLine")
          ? record.get("startLine").toNumber()
          : undefined,
        endLine: record.has("endLine")
          ? record.get("endLine").toNumber()
          : undefined,
        score: record.has("score") ? record.get("score") : undefined,
      }));
    } finally {
      await session.close();
    }
  }

  /**
   * Sets up and uses Neo4j's built-in full-text search for more advanced text searching
   * @param searchTerm The text to search for
   * @param limit Maximum number of results to return
   */
  async fullTextSearch(
    searchTerm: string,
    limit: number = 20,
  ): Promise<SearchResult[]> {
    const driver = this.getDriver();
    const session = driver.session();

    try {
      // First ensure the index exists
      try {
        await session.run(`
            CALL db.index.fulltext.createNodeIndex(
              'entitySearch',
              ['File', 'Directory', 'Function', 'Method', 'Class', 'Struct', 'Interface', 'Trait', 'Enum', 'Variable'],
              ['name', 'path']
            )
          `);
      } catch (error) {
        // Index might already exist, which is fine
        console.log("Index creation attempt:", error);
      }

      // Then perform the search
      const query = `
        CALL db.index.fulltext.queryNodes('entitySearch', $searchTerm)
        YIELD node, score
        RETURN node.name as name, node.path as path, labels(node)[0] as type,
               node.startLine as startLine, node.endLine as endLine, score
        ORDER BY score DESC
        LIMIT $limit
        `;

      const result = await session.run(query, {
        searchTerm,
        limit: neo4j.int(limit),
      });

      return result.records.map((record) => ({
        name: record.get("name"),
        path: record.get("path"),
        type: record.get("type")?.toLowerCase() || "unknown",
        startLine: record.has("startLine")
          ? record.get("startLine").toNumber()
          : undefined,
        endLine: record.has("endLine")
          ? record.get("endLine").toNumber()
          : undefined,
        score: record.has("score") ? record.get("score") : undefined,
      }));
    } finally {
      await session.close();
    }
  }

  /**
   * Get a complete 1-1 copy of the Neo4j database for a specific project (directory)
   * @param projectPath The path to the root directory of the project
   * @returns A CodeGraphData object with all nodes and relationships
   */
  async getProjectGraph(projectPath: string): Promise<CodeGraphData> {
    const driver = this.getDriver();
    const session = driver.session();

    try {
      // First ensure a project node exists (create if it doesn't)
      await this.ensureProjectNode(projectPath);

      // Get all nodes connected to the project, directly or indirectly
      const result = await session.run(
        `
        // Start with the project directory as root
        MATCH (project:Directory {path: $projectPath})

        // Get all nodes connected to this project (up to 10 hops away to ensure we get everything)
        MATCH (project)-[*0..10]-(node)

        // For each node, find its relationships with other nodes in this subgraph
        WITH collect(distinct node) as nodes
        UNWIND nodes as n
        MATCH (n)-[r]->(m)
        WHERE m IN nodes

        // Return all nodes and relationships
        RETURN collect(distinct n) as allNodes, collect(distinct r) as allRelationships
        `,
        { projectPath },
      );

      if (result.records.length === 0) {
        return { nodes: [], links: [] };
      }

      // Process all nodes
      const record = result.records[0];
      const allNodes = record.get("allNodes");
      const allRelationships = record.get("allRelationships");

      const nodesMap = new Map<string, any>();
      const links: any[] = [];

      // Add all nodes
      allNodes.forEach((node: any) => {
        const props = node.properties;
        // Use path as the ID to ensure uniqueness
        const id = props.path;

        if (!nodesMap.has(id)) {
          nodesMap.set(id, {
            id: id,
            name: props.name || id.split("/").pop(),
            type: this.mapNodeTypeToGraphType(node.labels[0]),
            group: node.labels[0],
            // For project root node
            isProjectRoot: props.path === projectPath,
            // Include additional properties that might be useful
            startLine: props.startLine?.toNumber(),
            endLine: props.endLine?.toNumber(),
            language: props.language,
            path: props.path,
          });
        }
      });

      // Add all relationships
      allRelationships.forEach((rel: any) => {
        const source = rel.start.toNumber();
        const target = rel.end.toNumber();
        const type = rel.type;

        // Find the nodes with these IDs
        const sourceNode = allNodes.find(
          (n: any) => n.identity.toNumber() === source,
        );
        const targetNode = allNodes.find(
          (n: any) => n.identity.toNumber() === target,
        );

        if (sourceNode && targetNode) {
          links.push({
            source: sourceNode.properties.path,
            target: targetNode.properties.path,
            type: type.toLowerCase(),
          });
        }
      });

      return {
        nodes: Array.from(nodesMap.values()),
        links,
      };
    } finally {
      await session.close();
    }
  }

  /**
   * Ensure a project node exists in the database
   * @param projectPath The path to the root directory of the project
   */
  private async ensureProjectNode(projectPath: string): Promise<void> {
    const driver = this.getDriver();
    const session = driver.session();

    try {
      // Check if the project node exists, create it if not
      const result = await session.run(
        `
        MERGE (project:Directory {path: $projectPath})
        ON CREATE SET
          project.name = $projectName,
          project.isRepository = true,
          project.isProjectRoot = true
        RETURN project
        `,
        {
          projectPath,
          projectName: projectPath.split("/").pop() || projectPath,
        },
      );
    } finally {
      await session.close();
    }
  }
  /**
   * Maps Neo4j node types to graph visualization types
   */
  private mapNodeTypeToGraphType(nodeType: string): string {
    // Map Neo4j node types to your graph node types
    switch (nodeType.toLowerCase()) {
      case "class":
      case "struct":
      case "interface":
      case "trait":
        return "class";
      case "method":
        return "method";
      case "function":
        return "function";
      case "variable":
      case "parameter":
        return "variable";
      case "file":
        return "file";
      case "directory":
        return "directory";
      case "import":
        return "import";
      case "callsite":
        return "callsite";
      default:
        return "other";
    }
  }
}
