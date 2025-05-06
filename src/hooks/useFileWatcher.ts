import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

export function fileWatcher() {
  const [currentDirectory, setDirectory] = useState<string>("");

  async function startWatching() {
    try {
      const result = await invoke("start_watching_directory", {
        directory: currentDirectory,
      });
      console.log(result);
    } catch (error) {
      console.error("Failed to start watching:", error);
    }
  }

  // Stop watching
  async function stopWatching() {
    try {
      const result = await invoke("stop_watching_directory");
      console.log(result);
    } catch (error) {
      console.error("Failed to stop watching:", error);
    }
  }

  return {
    currentDirectory,
    setDirectory,
    stopWatching,
  };
}
