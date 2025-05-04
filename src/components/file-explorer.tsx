"use client";

import { useState, useMemo, useEffect } from "react"; // Added useEffect
import { Tree, type TreeDataItem } from "./tree";
import { Button } from "./ui/button";
import {
  FileText,
  Folder as FolderIcon,
  FolderOpen,
  Search,
  Code2, // Use a code icon
} from "lucide-react";
import { Input } from "./ui/input";
import { Skeleton } from "./ui/skeleton";
import { ScrollArea } from "./ui/scroll-area";
import { CodeSnippet } from "./code-snippet"; // Import CodeSnippet
import { FileNode } from "@/lib/types";

interface FileExplorerProps {
  fileStructure: FileNode[];
  isLoading: boolean;
  onDirectorySelectRequest: () => Promise<void>;
  onNodeSelect: (node: FileNode | null) => void; // Keep this prop for external use if needed
  selectedDirectoryPath: string | null;
  // Add a function prop to fetch file content (mocked for now)
  onFileContentRequest: (filePath: string) => Promise<string | null>;
}

export function FileExplorer({
  fileStructure,
  isLoading,
  onDirectorySelectRequest,
  onNodeSelect, // Keep this prop
  selectedDirectoryPath,
  onFileContentRequest, // Use this prop
}: FileExplorerProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedNode, setSelectedNode] = useState<FileNode | null>(null); // Store the whole node
  const [selectedFileContent, setSelectedFileContent] = useState<string | null>(
    null,
  );
  const [isFileLoading, setIsFileLoading] = useState(false);

  const handleSelectDirectoryClick = () => {
    onDirectorySelectRequest();
  };

  // Filter logic remains the same
  const filterTree = (nodes: FileNode[], query: string): FileNode[] => {
    if (!query) {
      return nodes;
    }
    const lowerQuery = query.toLowerCase();

    return nodes.reduce((filtered: FileNode[], node) => {
      const isMatch = node.name.toLowerCase().includes(lowerQuery);
      const filteredChildren =
        node.type === "directory" && node.children
          ? filterTree(node.children, query)
          : [];

      if (
        isMatch ||
        (node.type === "directory" && filteredChildren.length > 0)
      ) {
        filtered.push({
          ...node,
          // Important: Pass down the original children if the node matches,
          // otherwise pass the filtered children. This preserves the structure
          // for nodes that match the search term directly.
          children: node.children
            ? isMatch
              ? node.children
              : filteredChildren
            : undefined,
        });
      }
      return filtered;
    }, []);
  };

  const filteredFileStructure = useMemo(
    () => filterTree(fileStructure || [], searchQuery),
    [fileStructure, searchQuery],
  );

  // Handle selection changes in the Tree
  const handleTreeSelectChange = async (item: TreeDataItem | undefined) => {
    const node = item as FileNode | undefined;
    setSelectedNode(node ?? null); // Update selected node state
    onNodeSelect(node ?? null); // Call external handler

    // Clear content if nothing selected or it's a directory
    if (!node || node.type === "directory") {
      setSelectedFileContent(null);
      return;
    }

    // If it's a file, fetch its content
    if (node.type === "file") {
      setIsFileLoading(true);
      setSelectedFileContent(null); // Clear previous content while loading
      try {
        // Use the passed function to get content (replace with actual API/FS call later)
        // For demo, let's assume content might be directly on the node or fetched
        const content = node.content ?? (await onFileContentRequest(node.path));
        setSelectedFileContent(content ?? "// Failed to load file content.");
      } catch (error) {
        console.error("Error fetching file content:", error);
        setSelectedFileContent("// Error loading file content.");
      } finally {
        setIsFileLoading(false);
      }
    }
  };

  // Reset selected node and content when directory changes or structure reloads
  useEffect(() => {
    setSelectedNode(null);
    setSelectedFileContent(null);
  }, [fileStructure]); // Depend on fileStructure

  return (
    <div className="flex flex-col h-full">
      {/* Top Section: Search (remains the same) */}
      <div className="p-2 border-b border-border flex-shrink-0">
        {" "}
        {/* Use border color variable */}
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search files..."
            className="pl-8 h-9" // Adjust padding and height
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            disabled={!selectedDirectoryPath || isLoading}
          />
        </div>
      </div>

      {/* Main Content Area: Split Panel */}
      <div className="flex flex-row flex-1 overflow-hidden">
        {/* Left Panel: File Tree */}
        <div className="w-1/3 border-r border-border flex flex-col overflow-hidden">
          {isLoading ? (
            // Loading Skeleton
            <div className="p-2 space-y-2 flex-1 overflow-y-auto">
              {Array.from({ length: 15 }).map((_, i) => (
                <div
                  key={i}
                  className="flex items-center gap-2"
                  style={{
                    paddingLeft: `${Math.random() * 3 * 1.25 + 0.5}rem`,
                  }} // Random indent
                >
                  <Skeleton className="h-4 w-4" />
                  <Skeleton className="h-4 w-[70%]" />
                </div>
              ))}
            </div>
          ) : selectedDirectoryPath && fileStructure.length > 0 ? (
            // Tree View wrapped in ScrollArea
            <ScrollArea className="flex-1">
              {" "}
              {/* ScrollArea takes available space */}
              <Tree
                data={filteredFileStructure}
                className="p-2" // Add padding inside ScrollArea
                // Pass selectedNode?.id to sync selection state
                initialSlelectedItemId={selectedNode?.id}
                onSelectChange={handleTreeSelectChange}
                folderIcon={FolderIcon}
                itemIcon={FileText}
              />
            </ScrollArea>
          ) : (
            // Empty State or No Directory Selected
            <div className="flex flex-col items-center justify-center h-full text-center p-4">
              <FolderOpen className="h-12 w-12 text-muted-foreground mb-4" />
              <h3 className="text-base font-medium mb-1 text-foreground">
                {selectedDirectoryPath
                  ? "Empty Directory"
                  : "Open a Project Folder"}
              </h3>
              <p className="text-sm text-muted-foreground mb-4 max-w-xs">
                {selectedDirectoryPath
                  ? `The selected folder is empty or contains no viewable files.`
                  : "Use the 'Select Directory' button above to load a project."}
              </p>
              {!selectedDirectoryPath && (
                <Button
                  size="sm"
                  onClick={handleSelectDirectoryClick}
                  disabled={isLoading}
                >
                  Select Directory
                </Button>
              )}
            </div>
          )}
        </div>

        {/* Right Panel: Code Snippet Viewer */}
        <div className="flex-1 flex flex-col overflow-hidden bg-background">
          {isFileLoading ? (
            // Loading state for file content
            <div className="flex items-center justify-center h-full">
              <p className="text-muted-foreground">Loading file...</p>
            </div>
          ) : selectedNode &&
            selectedNode.type === "file" &&
            selectedFileContent ? (
            // Display Code Snippet
            // Wrap CodeSnippet in ScrollArea for long files
            <ScrollArea className="flex-1">
              <CodeSnippet
                snippet={{
                  // Use node properties for the snippet data
                  title: selectedNode.name, // Or path for more context? selectedNode.path
                  language: selectedNode.language || "plaintext", // Provide fallback
                  code: selectedFileContent,
                }}
              />
            </ScrollArea>
          ) : (
            // Placeholder when no file is selected or content not available
            <div className="flex flex-col items-center justify-center h-full text-center p-8">
              <Code2 className="h-16 w-16 text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2 text-foreground">
                {selectedNode?.type === "directory"
                  ? `Folder Selected: ${selectedNode.name}`
                  : "No File Selected"}
              </h3>
              <p className="text-sm text-muted-foreground max-w-md">
                {selectedNode?.type === "directory"
                  ? "Select a file from the tree on the left to view its content."
                  : "Select a file from the tree on the left to view its content here."}
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
