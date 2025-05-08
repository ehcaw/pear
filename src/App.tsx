import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { homeDir } from "@tauri-apps/api/path";

import { Tabs, TabsTrigger, TabsContent, TabsList } from "./components/ui/tabs";
import { Button } from "./components/ui/button";
import { Input } from "./components/ui/input";
import { Card, CardContent } from "./components/ui/card";
import { ScrollArea } from "./components/ui/scroll-area";

import { WaveBackground } from "./components/wave-background";
import { PearLogo } from "./components/pear-logo";
import { DirectorySelector } from "./components/directory-selector";
import { FileExplorer } from "./components/file-explorer";
import { CodeGraph } from "./components/code-graph";
import { CodeSnippet } from "./components/code-snippet";
import { VoiceControls } from "./components/voice-controls";
import { ConversationBubble } from "./components/conversation-bubble";
import type { FileNode } from "@/lib/types";

import { useCodebase } from "./hooks/useCodebase";
import { useConversation } from "./hooks/useConversation";

import {
  Sparkles,
  MessageSquare,
  Network,
  Terminal,
  Folder,
  Zap,
  Send,
  Code2,
} from "lucide-react";

console.log(
  "config ",
  import.meta.env.VITE_NEO4J_URI!,
  import.meta.env.VITE_NEO4J_USERNAME!,
  import.meta.env.VITE_NEO4J_PASSWORD!,
);

function App() {
  const [status, setStatus] = useState("Ready");
  const [loading, setLoading] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const [selectedTab, setSelectedTab] = useState("conversation");
  const [inputText, setInputText] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [currentSelectedNode, setCurrentSelectedNode] =
    useState<FileNode | null>(null);
  // --- New state for file content ---
  const [currentFileContent, setCurrentFileContent] = useState<string | null>(
    null,
  );
  const [isFileContentLoading, setIsFileContentLoading] =
    useState<boolean>(false);

  const {
    messages,
    isDetectingSpeech,
    isUserCurrentlySpeaking,
    isProcessing,
    isSpeaking,
    isMuted,
    codeSnippets,
    startSpeechDetection,
    stopSpeechDetection,
    handleSpeechEnd,
    toggleMute,
    clearConversation,
  } = useConversation();

  // --- Codebase Hook ---
  const {
    selectedDirectory,
    setSelectedDirectory,
    fileStructure,
    codeGraph,
    isLoading,
    refreshCodebase,
  } = useCodebase(
    import.meta.env.VITE_NEO4J_URI!,
    import.meta.env.VITE_NEO4J_USERNAME!,
    import.meta.env.VITE_NEO4J_PASSWORD!,
  );

  // Setup event listeners for backend events
  useEffect(() => {
    const unlisten1 = listen("parse_progress", (event) => {
      setStatus(`In progress: ${event.payload as string}`);
      setLogs((prev) => [...prev, `Progress: ${event.payload as string}`]);
    });

    const unlisten2 = listen("parse_error", (event) => {
      setStatus(`Error: ${event.payload as string}`);
      setLogs((prev) => [...prev, `Error: ${event.payload as string}`]);
    });

    const unlisten3 = listen("parse_complete", (event) => {
      setStatus(`${event.payload as string}`);
      setLogs((prev) => [...prev, `Complete: ${event.payload as string}`]);
      setLoading(false);
    });

    return () => {
      unlisten1.then((unsub) => unsub());
      unlisten2.then((unsub) => unsub());
      unlisten3.then((unsub) => unsub());
    };
  }, []);

  // Select a directory using Tauri dialog
  async function selectDirectory() {
    try {
      const selectedDir = await invoke<string>("select_directory");
      if (selectedDir) {
        setSelectedDirectory(selectedDir);
        setLogs((prev) => [...prev, `Selected directory: ${selectedDir}`]);
      }
    } catch (error) {
      setStatus(`Error: ${error}`);
      setLogs((prev) => [...prev, `Error selecting directory: ${error}`]);
    }
  }
  const handleSubmitQuery = () => {
    if (inputText.trim()) {
      console.log("Submitted:", inputText);
      // Add your submission logic here
      setInputText("");
    }
  };

  const handleDirectorySelected = async (path: string | null) => {
    console.log("Directory selected in App:", path);
    setSelectedDirectory(path || ""); // Update the local state
    if (path) {
      // Update the codebase hook's state as well
      await invoke<string>("select_directory");
      setSelectedDirectory(path);
      setLogs((prev) => [...prev, `Selected directory: ${path}`]);
    } else {
      setLogs((prev) => [...prev, "Directory selection cancelled."]);
    }
  };

  async function getFileContent(filepath: string): Promise<string | null> {
    try {
      // Invoke expects the specific success type (string)
      const content = await invoke<string>("read_file_content", {
        filePath: filepath, // Ensure the key matches the Rust function argument name (snake_case)
      });
      return content; // Return the string if successful
    } catch (error) {
      // Log the error received from the Rust backend
      console.error(`Failed to read file content for '${filepath}':`, error);
      return null; // Return null if the promise was rejected (Rust returned Err)
    }
  }

  // --- Function to handle the request from FileExplorer's internal button ---
  // This duplicates the logic from DirectorySelector's internal handler.
  // Consider refactoring if possible, but this works for now.
  const requestDirectorySelection = async () => {
    try {
      const defaultPath = await homeDir();
      const result = await open({
        directory: true,
        multiple: false,
        defaultPath: defaultPath,
      });
      // Call the main handler with the result
      handleDirectorySelected(typeof result === "string" ? result : null);
    } catch (error) {
      console.error("Error opening directory dialog from request:", error);
      handleDirectorySelected(null);
    }
  };

  const handleNodeSelected = async (node: FileNode | null) => {
    console.log("Node selected in App:", node);
    setCurrentSelectedNode(node); // Update the selected node state

    if (node && node.type === "file") {
      // It's a file, try to fetch its content
      setIsFileContentLoading(true);
      setCurrentFileContent(null); // Clear previous content
      console.log(`Fetching content for file: ${node.path}`); // Log fetch attempt
      try {
        const content = await getFileContent(node.path);
        console.log(content);
        if (content !== null) {
          setCurrentFileContent(content);
          console.log(`Content fetched successfully for ${node.path}`);
        } else {
          // getFileContent handles its own errors and returns null
          setCurrentFileContent("// Failed to load file content.");
          console.log(`getFileContent returned null for ${node.path}`);
        }
      } catch (error) {
        // Catch any unexpected errors from getFileContent itself (though it should return null)
        console.error("Unexpected error fetching file content:", error);
        setCurrentFileContent("// Error loading file content.");
      } finally {
        setIsFileContentLoading(false); // Stop loading indicator
      }
    } else {
      // It's a directory or null selection
      setCurrentFileContent(null); // Clear content display
      setIsFileContentLoading(false); // Ensure loading is off
      if (node) {
        console.log(`Directory selected: ${node.path}`);
      } else {
        console.log("Selection cleared.");
      }
    }
  };

  return (
    <main className="container mx-auto h-screen overflow-hidden">
      <div className="flex flex-col h-full bg-cream-50 dark:bg-gray-950">
        <WaveBackground />

        {/* Improved Header */}
        <header className="relative z-10 border-b border-zed-100 dark:border-zed-800 py-3 px-6 flex items-center justify-between bg-cream-50/90 backdrop-blur-md dark:bg-gray-900/90 shadow-sm">
          <div className="flex items-center gap-3">
            <div className="relative">
              <PearLogo className="h-9 w-9" />
              <span className="absolute -top-1 -right-1 flex h-3 w-3">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-zed-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-3 w-3 bg-zed-500"></span>
              </span>
            </div>
            <div>
              <h1 className="text-xl font-bold bg-gradient-to-r from-zed-500 to-zed-700 bg-clip-text text-transparent">
                Pear
              </h1>
              <p className="text-xs text-zed-600 dark:text-zed-300">
                Your AI Pair Programmer
              </p>
            </div>
          </div>

          <div className="flex items-center gap-4">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => clearConversation?.()}
              className="text-zed-600 hover:text-zed-800 hover:bg-zed-50 dark:text-zed-300 dark:hover:text-zed-100 dark:hover:bg-zed-900/50"
            >
              <Sparkles className="h-4 w-4 mr-2" />
              New Session
            </Button>

            <div className="relative w-64 md:w-80">
              <Input
                value={inputText}
                onChange={(e) => setInputText(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleSubmitQuery()}
                placeholder="Ask me anything..."
                className="pr-10 bg-white/90 dark:bg-gray-800/90 border-zed-200 dark:border-zed-700"
              />
              <Button
                size="icon"
                variant="ghost"
                onClick={handleSubmitQuery}
                className="absolute right-1 top-1/2 -translate-y-1/2 h-7 w-7 text-zed-500 hover:text-zed-700"
              >
                <Send className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </header>
        <main className="flex-1 flex flex-col md:flex-row overflow-hidden relative z-10">
          <Tabs
            value={selectedTab}
            onValueChange={setSelectedTab}
            className="flex-1 flex flex-col h-full"
          >
            {/* Improved Tabs Navigation */}
            <div className="px-6 pt-4 border-b border-zed-100 dark:border-zed-800/50 bg-cream-50/80 dark:bg-gray-900/80 backdrop-blur-sm">
              <TabsList className="w-full md:w-auto bg-cream-100 dark:bg-gray-800 p-1 rounded-md">
                <TabsTrigger
                  value="conversation"
                  className="flex-1 md:flex-initial rounded-md data-[state=active]:bg-white data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-700"
                >
                  <MessageSquare className="h-4 w-4 mr-2" />
                  Conversation
                </TabsTrigger>
                <TabsTrigger
                  value="code"
                  className="flex-1 md:flex-initial rounded-md data-[state=active]:bg-white data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-700"
                >
                  <Terminal className="h-4 w-4 mr-2" />
                  Code Snippets
                </TabsTrigger>
                <TabsTrigger
                  value="files"
                  className="flex-1 md:flex-initial rounded-md data-[state=active]:bg-white data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-700"
                >
                  <Folder className="h-4 w-4 mr-2" />
                  Files
                </TabsTrigger>
                <TabsTrigger
                  value="graph"
                  className="flex-1 md:flex-initial rounded-md data-[state=active]:bg-white data-[state=active]:shadow-sm dark:data-[state=active]:bg-gray-700"
                >
                  <Network className="h-4 w-4 mr-2" />
                  Graph
                </TabsTrigger>
              </TabsList>
            </div>

            {/* Directory Selector with improved styling */}
            <div className="px-6 py-3 border-b border-zed-100 dark:border-zed-800 bg-white/50 dark:bg-gray-800/50 backdrop-blur-sm">
              <DirectorySelector
                selectedDirectory={selectedDirectory} // Pass the directory state
                onDirectorySelected={handleDirectorySelected} // Pass the new callback
                onRefreshDirectoryClick={refreshCodebase}
                isLoading={isLoading || loading}
              />
            </div>
            {/* Conversation Tab */}
            <TabsContent
              value="conversation"
              className="flex-1 flex flex-col p-6 overflow-hidden bg-white/30 dark:bg-gray-900/30 backdrop-blur-sm"
            >
              <div className="flex-1 overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-zed-200 scrollbar-track-transparent dark:scrollbar-thumb-zed-700">
                {messages.length === 0 ? (
                  <div className="h-full flex flex-col items-center justify-center text-center p-8">
                    <div className="relative mb-8">
                      <PearLogo className="h-28 w-28 animate-float" />
                      <div className="absolute -right-4 top-0 transform rotate-12">
                        <Zap className="h-10 w-10 text-zed-500 animate-pulse-slow" />
                      </div>
                    </div>
                    <h2 className="text-3xl font-bold mb-4 bg-gradient-to-r from-zed-600 to-zed-400 bg-clip-text text-transparent">
                      Hello, I'm Pear
                    </h2>
                    <p className="text-zed-600 dark:text-zed-300 max-w-md mb-6 text-lg">
                      Your AI pair programming buddy. I can help you write code,
                      debug issues, and discuss programming concepts.
                    </p>
                    <Button className="bg-zed-500 hover:bg-zed-600 text-white">
                      Start Conversation
                    </Button>
                  </div>
                ) : (
                  <div className="space-y-6 py-2">
                    {messages.map((message, index) => (
                      <ConversationBubble
                        key={index}
                        message={message}
                        isLast={index === messages.length - 1 && isSpeaking}
                      />
                    ))}
                    <div ref={messagesEndRef} />
                  </div>
                )}
              </div>
              {/* Voice Controls Card with improved styling */}
              <Card className="mt-4 border-zed-200 dark:border-zed-800 bg-white/90 backdrop-blur-sm shadow-sm dark:bg-gray-800/90">
                <CardContent className="p-4">
                  <VoiceControls
                    isDetectingSpeech={isDetectingSpeech}
                    isUserCurrentlySpeaking={isUserCurrentlySpeaking}
                    isProcessing={isProcessing}
                    isAssistantSpeaking={isSpeaking}
                    isMuted={isMuted}
                    onToggleSpeechDetection={
                      isDetectingSpeech
                        ? stopSpeechDetection
                        : startSpeechDetection
                    }
                    onToggleMute={toggleMute}
                    onTextSubmit={(text) => {
                      console.log("Text submitted:", text);
                      // Replace with actual submission logic
                    }}
                  />
                </CardContent>
              </Card>
            </TabsContent>

            {/* Code Snippets Tab */}
            <TabsContent
              value="code"
              className="flex-1 p-6 overflow-hidden bg-white/30 dark:bg-gray-900/30"
            >
              <Card className="h-full overflow-hidden border-zed-200 dark:border-zed-800 shadow-sm">
                <CardContent className="p-0 h-full">
                  {" "}
                  {/* Remove padding if CodeSnippet handles it */}
                  {isFileContentLoading ? (
                    // --- Loading State ---
                    <div className="flex items-center justify-center h-full">
                      <p className="text-muted-foreground">
                        Loading content for {currentSelectedNode?.name}...
                      </p>
                    </div>
                  ) : currentSelectedNode &&
                    currentSelectedNode.type === "file" &&
                    currentFileContent !== null ? (
                    // --- Content Loaded State ---
                    <ScrollArea className="h-full">
                      {" "}
                      {/* Wrap in ScrollArea */}
                      <CodeSnippet
                        snippet={{
                          title: currentSelectedNode.path, // Use full path for clarity
                          language: currentSelectedNode.language || "plaintext", // Use language from node if available
                          code: currentFileContent,
                        }}
                      />
                    </ScrollArea>
                  ) : (
                    // --- Empty/Placeholder State ---
                    <div className="flex flex-col items-center justify-center h-full text-center p-8">
                      <Code2 className="h-16 w-16 text-muted-foreground mb-4" />
                      <h3 className="text-lg font-medium mb-2 text-foreground">
                        {currentSelectedNode?.type === "directory"
                          ? `Folder Selected: ${currentSelectedNode.name}`
                          : "No File Selected"}
                      </h3>
                      <p className="text-sm text-muted-foreground max-w-md">
                        {currentSelectedNode?.type === "directory"
                          ? "Select a file from the 'Files' tab to view its content here."
                          : "Select a file from the 'Files' tab to view its content here."}
                      </p>
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>
            {/* Files Tab */}
            <TabsContent
              value="files"
              className="flex-1 p-6 overflow-hidden bg-white/30 dark:bg-gray-900/30"
            >
              <Card className="h-full overflow-hidden border-zed-200 dark:border-zed-800 shadow-sm">
                <CardContent className="p-4 h-full">
                  {fileStructure ? (
                    <FileExplorer
                      fileStructure={fileStructure || []} // Pass file structure from hook (provide default empty array)
                      // Use the loading state specific to fetching the file structure from useCodebase
                      isLoading={isLoading}
                      // Pass the function to request directory selection (for the internal button)
                      onDirectorySelectRequest={requestDirectorySelection}
                      // Pass the handler for when a node is selected in the tree
                      onNodeSelect={handleNodeSelected}
                      // Pass the currently selected root directory path
                      selectedDirectoryPath={selectedDirectory}
                      // Pass the function to fetch file content when a file node is clicked
                      onFileContentRequest={async (filepath) => {
                        console.log(`Requesting content for: ${filepath}`); // Add log
                        return await getFileContent(filepath);
                      }}
                    />
                  ) : (
                    <div className="h-full flex items-center justify-center text-center">
                      <p className="text-zed-500 dark:text-zed-400">
                        {isLoading
                          ? "Loading file structure..."
                          : "Select a directory to view files"}
                      </p>
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>

            {/* Graph Tab */}
            <TabsContent
              value="graph"
              className="flex-1 p-6 overflow-hidden bg-white/30 dark:bg-gray-900/30"
            >
              <Card className="h-full overflow-hidden border-zed-200 dark:border-zed-800 shadow-sm">
                <CardContent className="p-0 h-full">
                  {" "}
                  {/* No padding for the graph */}
                  <CodeGraph
                    codeGraph={codeGraph}
                    isLoading={isLoading || loading}
                  />
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
        </main>
      </div>
    </main>
  );
}

export default App;
