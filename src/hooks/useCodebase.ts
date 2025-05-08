"use client";

import { useState, useEffect, useCallback } from "react";
import type { FileNode } from "@/lib/types";
import type { CodeGraphData, GraphNode } from "@/lib/types";

// Import invoke from Tauri core
import { invoke } from "@tauri-apps/api/core"; // Correct import for invoke
import { directoryStore } from "@/context/state";
import { Neo4jDriver } from "@/lib/neo4j";

export function useCodebase(uri: string, username: string, password: string) {
  const { selectedDirectory, setSelectedDirectory } = directoryStore();
  const [fileStructure, setFileStructure] = useState<FileNode[]>([]);
  const [codeGraph, setCodeGraph] = useState<CodeGraphData>({
    nodes: [],
    links: [],
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const neo4jInstance = new Neo4jDriver(uri, username, password);

  // Encapsulate loading logic
  const loadCodebase = useCallback(async (directory: string | null) => {
    setError(null); // Clear previous errors

    if (!directory) {
      setFileStructure([]);
      setCodeGraph({ nodes: [], links: [] });
      setIsLoading(false);
      return;
    }

    console.log(`Loading codebase via backend for: ${directory}`); // Updated log
    setIsLoading(true);

    try {
      // --- Call the Rust backend command ---
      const structureFromRust = await invoke<FileNode[]>( // Expect FileNode[] which matches FileNodeRust structure
        "read_directory_structure",
        { dirPathStr: directory }, // Pass argument matching Rust function signature
      );

      console.log(structureFromRust);

      setFileStructure(structureFromRust);

      const parseCodebaseResult: string = await invoke(
        "parse_and_ingest_codebase",
        { directory: directory },
      );

      console.log(parseCodebaseResult);

      // --- Generate Code Graph ---
      try {
        console.log("Tried to generate code graph");
        const codeGraph =
          await neo4jInstance.getProjectGraph(selectedDirectory);
        console.log("nodes ", codeGraph.nodes);
        console.log("links ", codeGraph.links);
        setCodeGraph(codeGraph);
      } catch (graphError: any) {
        console.error("Error generating code graph:", graphError);

        // Still continue with the file structure even if graph fails
      }
      if (structureFromRust.length === 0) {
        console.warn(
          "Directory appears to be empty or only contains ignored files.",
        );
      }
    } catch (error: any) {
      // Handle errors from invoke (could be Rust error string or other JS errors)
      console.error("Error loading codebase via backend:", error);
      const errorMessage =
        typeof error === "string"
          ? error
          : error?.message || "Unknown error occurred";
      setError(
        `Failed to load codebase: ${errorMessage} (Check console and Tauri permissions)`,
      );
      setFileStructure([]);
      setCodeGraph({ nodes: [], links: [] });
    } finally {
      setIsLoading(false);
    }
  }, []); // Dependency array remains empty

  // Effect to load when selectedDirectory changes
  useEffect(() => {
    loadCodebase(selectedDirectory);
  }, [selectedDirectory, loadCodebase]);

  // Function to explicitly refresh the current directory
  const refreshCodebase = useCallback(() => {
    console.log("Refresh triggered for:", selectedDirectory);
    if (selectedDirectory) {
      loadCodebase(selectedDirectory);
    } else {
      console.log("Cannot refresh, no directory selected.");
      setError("Cannot refresh: No directory selected.");
    }
  }, [selectedDirectory, loadCodebase]);

  return {
    selectedDirectory,
    setSelectedDirectory,
    fileStructure,
    codeGraph,
    isLoading,
    error,
    refreshCodebase,
  };
}
