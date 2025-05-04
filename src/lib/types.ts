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
