"use client";

import { useState, useEffect, useCallback } from "react";
import type { FileNode } from "../components/file-explorer"; // Keep this interface
import type { CodeGraphData } from "../components/code-graph";

// Import invoke from Tauri core
import { invoke } from "@tauri-apps/api/core"; // Correct import for invoke

// Remove the JS buildFileTree function and related FS imports
// import { readDir, type FileEntry } from "@tauri-apps/api/fs"; // No longer needed
// async function buildFileTree(...) { ... } // Remove this function

export function useCodebase() {
  const [selectedDirectory, setSelectedDirectory] = useState<string | null>(
    null,
  );
  const [fileStructure, setFileStructure] = useState<FileNode[]>([]);
  const [codeGraph, setCodeGraph] = useState<CodeGraphData>({
    nodes: [],
    links: [],
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

      setFileStructure(structureFromRust);

      // --- Generate Code Graph ---
      try {
        console.log("Tried to generate code graph");
      } catch (graphError: any) {
        console.error("Error generating code graph:", graphError);
        // Still continue with the file structure even if graph fails
      }
      console.log("File structure loaded successfully via backend.");
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
