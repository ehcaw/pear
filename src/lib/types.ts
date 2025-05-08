import { TreeDataItem } from "@/components/tree";

// Ensure FileNode includes type and potentially content/language later
export interface FileNode extends TreeDataItem {
  id: string; // Usually the path
  name: string;
  path: string;
  type: "file" | "directory";
  language?: string;
  content?: string; // Add content field for mock data
  children?: FileNode[];
}

export interface GraphNode {
  id: string;
  name: string;
  type:
    | "class"
    | "method"
    | "function"
    | "variable"
    | "file"
    | "directory"
    | "import";
  group?: string;
  value?: number;
}

export interface GraphLink {
  source: string;
  target: string;
  type: "calls" | "imports" | "extends" | "implements" | "contains";
}

export interface CodeGraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

// Add the SearchResult interface for Neo4j fuzzy search results
export interface SearchResult {
  name: string;
  path: string;
  type: string;
  startLine?: number;
  endLine?: number;
  score?: number; // For search relevance
}
