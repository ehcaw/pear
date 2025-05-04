"use client";

import { Button } from "./ui/button";
import { FolderOpen, RefreshCw } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog"; // Import the 'open' function
import { homeDir } from "@tauri-apps/api/path"; // Import homeDir for default path

interface DirectorySelectorProps {
  selectedDirectory: string | null;
  // NEW: Callback function when a directory IS selected
  onDirectorySelected: (path: string | null) => void;
  onRefreshDirectoryClick: () => void;
  isLoading?: boolean;
}

export function DirectorySelector({
  selectedDirectory,
  onDirectorySelected, // Use the new callback
  onRefreshDirectoryClick,
  isLoading = false,
}: DirectorySelectorProps) {
  // Function to handle the directory selection using the frontend API
  const handleSelectDirectory = async () => {
    try {
      const defaultPath = await homeDir();
      const result = await open({
        directory: true, // Specify we want to select a directory
        multiple: false, // Only allow selecting one directory
        defaultPath: defaultPath, // Start in the user's home directory
      });

      if (typeof result === "string") {
        // User selected a directory
        onDirectorySelected(result);
      } else {
        // User cancelled or selected nothing
        onDirectorySelected(null); // Or maybe don't call it if cancelled? Depends on desired behavior.
      }
    } catch (error) {
      console.error("Error opening directory dialog:", error);
      // Optionally, notify the parent or show an error message
      onDirectorySelected(null);
    }
  };

  return (
    <div className="space-y-3">
      <div className="flex flex-col sm:flex-row gap-2">
        {/* Button now calls the local handleSelectDirectory */}
        <Button
          onClick={handleSelectDirectory}
          className="flex-grow"
          disabled={isLoading}
        >
          <FolderOpen className="h-4 w-4 mr-2" />
          {selectedDirectory ? "Change Directory" : "Select Directory"}
        </Button>

        {/* Refresh Button remains the same */}
        <Button
          variant="outline"
          onClick={onRefreshDirectoryClick}
          disabled={!selectedDirectory || isLoading}
          className="border-zed-200 dark:border-zed-700"
          title="Refresh directory contents"
        >
          <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
          <span className="ml-2 hidden sm:inline">Refresh</span>{" "}
        </Button>
      </div>

      {/* Display logic remains the same */}
      {selectedDirectory && (
        <div className="text-sm text-muted-foreground dark:text-zed-400 break-all">
          <span className="font-medium text-foreground dark:text-zed-300">
            Current:
          </span>{" "}
          {selectedDirectory}
        </div>
      )}
      {!selectedDirectory && !isLoading && (
        <div className="text-sm text-muted-foreground dark:text-zed-500">
          No directory selected. Click the button above to start.
        </div>
      )}
      {isLoading && selectedDirectory && (
        <div className="text-sm text-muted-foreground dark:text-zed-500 flex items-center gap-2">
          <RefreshCw className="h-4 w-4 animate-spin" />
          <span>Loading {selectedDirectory}...</span>
        </div>
      )}
    </div>
  );
}
