"use client";

import { Button } from "./ui/button";
import { FolderOpen, RefreshCw } from "lucide-react";
// Keep tauri imports if used here, or move logic to App.tsx if preferred
import { open } from "@tauri-apps/plugin-dialog";
import { homeDir } from "@tauri-apps/api/path";

interface DirectorySelectorProps {
  selectedDirectory: string | null;
  // NEW: Function to *initiate* the directory selection process
  onSelectDirectoryClick: () => Promise<void>;
  // NEW: Function to trigger a refresh of the current directory
  onRefreshDirectoryClick: () => void;
  isLoading?: boolean;
}

export function DirectorySelector({
  selectedDirectory,
  onSelectDirectoryClick, // Use the new prop name
  onRefreshDirectoryClick, // Use the new prop name
  isLoading = false,
}: DirectorySelectorProps) {
  // The handleSelectDirectory logic is now primarily managed by the parent via onSelectDirectoryClick
  // The refresh logic is now primarily managed by the parent via onRefreshDirectoryClick

  return (
    <div className="space-y-3">
      <div className="flex flex-col sm:flex-row gap-2">
        {/* Button to trigger the directory selection process */}
        <Button
          onClick={onSelectDirectoryClick} // Use the passed handler
          className="flex-grow"
          disabled={isLoading}
        >
          <FolderOpen className="h-4 w-4 mr-2" />
          {selectedDirectory ? "Change Directory" : "Select Directory"}
        </Button>

        {/* Refresh Button */}
        <Button
          variant="outline"
          onClick={onRefreshDirectoryClick} // Use the passed handler
          disabled={!selectedDirectory || isLoading}
          className="border-zed-200 dark:border-zed-700"
          title="Refresh directory contents"
        >
          <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
          <span className="ml-2 hidden sm:inline">Refresh</span>{" "}
        </Button>
      </div>

      {/* Display the currently selected directory */}
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
