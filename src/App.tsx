import { Folder, Network } from "lucide-react";
import { useCodebase } from "@/hooks/use-codebase";
import { FileExplorer } from "@/components/file-explorer";
import { CodeGraph } from "@/components/code-graph";
import { DirectorySelector } from "@/components/directory-selector";
import { useState, useEffect, useRef, useCallback } from "react"; // Import useCallback
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Code, MessageSquare, Sparkles, Terminal, Zap } from "lucide-react";
import { ConversationBubble } from "@/components/conversation-bubble";
import { VoiceControls } from "@/components/voice-controls";
import { useConversation } from "@/hooks/use-conversation";
import { PearLogo } from "@/components/pear-logo";
import { Input } from "./components/ui/input";
import { WaveBackground } from "@/components/wave-background";
import { CodeSnippet } from "@/components/code-snippet";
import { useMicVAD } from "@ricky0123/vad-react";

// Import Tauri API
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { homeDir } from "@tauri-apps/api/path";

export default function PearInterface() {
  // --- Conversation Hook ---
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
    handleSpeechEnd, // Assuming handleSpeechEnd exists and works
    toggleMute,
    clearConversation,
  } = useConversation();

  // --- Codebase Hook ---
  const {
    selectedDirectory,
    setSelectedDirectory, // Get the setter
    fileStructure,
    codeGraph,
    isLoading,
    refreshCodebase, // Get the refresh function
  } = useCodebase();

  // --- State & Refs ---
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [activeTab, setActiveTab] = useState("conversation");

  const [input, setInput] = useState<String>("");

  // --- Effects ---
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // --- VAD Hook --- (Keep as is, assuming it interacts with useConversation correctly)
  const vad = useMicVAD({
    startOnLoad: false, // Don't start automatically maybe?
    onSpeechEnd: async (audio) => {
      // Simplified check - might need more robust logic from useConversation
      if (!isMuted && !isProcessing && !isSpeaking) {
        console.log("VAD Speech ended, processing...");
        // Ensure handleSpeechEnd is defined and works in useConversation
        //await handleSpeechEnd(audio);
      } else {
        console.log(
          "VAD Speech ended, but ignored (muted/processing/speaking)",
        );
      }
    },
    // Consider adding onSpeechStart if needed by useConversation
  });

  // Effect to control VAD based on mute state
  useEffect(() => {
    if (!isMuted && !vad.listening) {
      console.log("Starting VAD");
      vad.start();
    } else if (isMuted && vad.listening) {
      console.log("Pausing VAD");
      vad.pause();
    }
    // Cleanup function to stop VAD when component unmounts or isMuted changes significantly
    // return () => {
    //     if(vad.listening) vad.destroy(); // Or vad.pause() depending on desired behavior
    // }
  }, [isMuted, vad]); // Add vad to dependencies

  // --- Handlers ---

  // Centralized function to handle directory selection
  const handleSelectDirectory = useCallback(async () => {
    try {
      const defaultPath = selectedDirectory || (await homeDir());
      const result = await open({
        directory: true,
        multiple: false,
        title: "Select Project Directory",
        defaultPath: defaultPath,
      });

      if (typeof result === "string") {
        console.log("Directory selected via dialog:", result);
        setSelectedDirectory(result); // Update state using the setter from useCodebase
        invoke("embed_codebase", { dirPathStr: result });
      } else if (result === null) {
        console.log("Directory selection cancelled.");
      }
    } catch (error) {
      console.error("Error opening directory dialog:", error);
      // Optionally notify user
      // setSelectedDirectory(null); // Optionally clear selection on error
    }
  }, [selectedDirectory, setSelectedDirectory]); // Dependencies

  const handleToggleSpeechDetection = () => {
    if (isDetectingSpeech) {
      stopSpeechDetection();
      // vad.pause(); // Also control VAD here if useConversation doesn't
    } else {
      startSpeechDetection();
      // if (!isMuted) vad.start(); // Also control VAD here if useConversation doesn't
    }
  };

  // Handler for node selection in File Explorer (example)
  const handleNodeSelect = (node: any | null) => {
    // 'any' because FileNode type might not be directly usable depending on Tree component implementation
    if (node) {
      console.log("Node selected in File Explorer:", node.path);
      // Potentially open file, feed path to conversation, etc.
    } else {
      console.log("File Explorer selection cleared.");
    }
  };

  const getFileContent = async (filePath: string) => {
    const fileContent = await invoke("read_file", { path: filePath });
    return fileContent as string;
  };

  const submitQuery = async () => {
    console.log("aklsdjflkasjfklads");
    const files = await invoke("query_document", {
      query: input,
      directory: selectedDirectory,
    });
    console.log(files);
  };

  // --- Render ---
  return (
    <div className="flex flex-col h-screen bg-cream-50 dark:bg-gray-950 overflow-hidden">
      <WaveBackground />

      <header className="relative z-10 border-b border-zed-100 dark:border-zed-800 py-3 px-4 flex items-center justify-between bg-cream-50/80 backdrop-blur-md dark:bg-gray-900/80">
        {/* Header content remains the same */}
        <div className="flex items-center gap-3">
          <div className="relative">
            <PearLogo className="h-8 w-8" />
            {/* ... ping animation ... */}
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
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={clearConversation}
            className="text-zed-600 hover:text-zed-800 hover:bg-zed-50 dark:text-zed-300 dark:hover:text-zed-100 dark:hover:bg-zed-900/50"
          >
            <Sparkles className="h-4 w-4 mr-2" />
            New Session
          </Button>
        </div>
        <Input
          onSubmit={() => submitQuery}
          onChange={(e) => setInput(e.target.value)}
        ></Input>
        <Button onClick={submitQuery}>Submit</Button>
      </header>

      <main className="flex-1 flex flex-col md:flex-row overflow-hidden relative z-10">
        <Tabs
          value={activeTab}
          onValueChange={setActiveTab}
          className="flex-1 flex flex-col h-full"
        >
          {/* Tabs List remains the same */}
          <div className="px-4 pt-4 border-b border-zed-100 dark:border-zed-800/50">
            <TabsList className="w-full md:w-auto bg-cream-100 dark:bg-gray-800">
              <TabsTrigger
                value="conversation"
                className="flex-1 md:flex-initial data-[state=active]:bg-white dark:data-[state=active]:bg-gray-700"
              >
                <MessageSquare className="h-4 w-4 mr-2" />
                Conversation
              </TabsTrigger>
              <TabsTrigger
                value="code"
                className="flex-1 md:flex-initial data-[state=active]:bg-white dark:data-[state=active]:bg-gray-700"
              >
                <Terminal className="h-4 w-4 mr-2" />
                Code Snippets
              </TabsTrigger>
              <TabsTrigger
                value="files"
                className="flex-1 md:flex-initial data-[state=active]:bg-white dark:data-[state=active]:bg-gray-700"
              >
                <Folder className="h-4 w-4 mr-2" />
                File Explorer
              </TabsTrigger>
              <TabsTrigger
                value="graph"
                className="flex-1 md:flex-initial data-[state=active]:bg-white dark:data-[state=active]:bg-gray-700"
              >
                <Network className="h-4 w-4 mr-2" />
                Code Graph
              </TabsTrigger>
            </TabsList>
          </div>

          {/* Directory Selector Area */}
          <div className="px-4 pt-4 border-b border-zed-100 dark:border-zed-800">
            <DirectorySelector
              selectedDirectory={selectedDirectory}
              onSelectDirectoryClick={handleSelectDirectory} // Pass the handler
              onRefreshDirectoryClick={refreshCodebase} // Pass the refresh function from the hook
              isLoading={isLoading} // Pass loading state
            />
          </div>

          {/* Tabs Content */}

          {/* Conversation Tab */}
          <TabsContent
            value="conversation"
            className="flex-1 flex flex-col p-4 overflow-hidden"
          >
            <div className="flex-1 overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-zed-200 scrollbar-track-transparent dark:scrollbar-thumb-zed-700">
              {messages.length === 0 ? (
                <div className="h-full flex flex-col items-center justify-center text-center p-8">
                  {/* Empty state content */}
                  <div className="relative mb-6">
                    <PearLogo className="h-24 w-24 animate-float" />
                    <div className="absolute -right-4 top-0 transform rotate-12">
                      <Zap className="h-8 w-8 text-zed-500 animate-pulse-slow" />
                    </div>
                  </div>
                  <h2 className="text-3xl font-bold mb-3 bg-gradient-to-r from-zed-600 to-zed-400 bg-clip-text text-transparent">
                    Hello, I'm Pear
                  </h2>
                  <p className="text-zed-600 dark:text-zed-300 max-w-md mb-6">
                    Your AI pair programming buddy. I can help you write code,
                    debug issues, and discuss programming concepts.
                  </p>
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
            <Card className="mt-4 border-zed-200 dark:border-zed-800 bg-white/80 backdrop-blur-sm dark:bg-gray-800/80">
              <CardContent className="p-4">
                <VoiceControls
                  isDetectingSpeech={isDetectingSpeech}
                  isUserCurrentlySpeaking={isUserCurrentlySpeaking}
                  isProcessing={isProcessing}
                  isAssistantSpeaking={isSpeaking}
                  isMuted={isMuted}
                  onToggleSpeechDetection={handleToggleSpeechDetection}
                  onToggleMute={toggleMute}
                  onTextSubmit={(text) => console.log("Text submitted:", text)} // Replace with actual submission logic
                />
              </CardContent>
            </Card>
          </TabsContent>

          {/* Code Snippets Tab */}
          <TabsContent value="code" className="flex-1 p-4 overflow-auto">
            {codeSnippets.length === 0 ? (
              <div className="h-full flex flex-col items-center justify-center text-center p-8">
                <Code className="h-16 w-16 text-zed-300 mb-4" />
                <h3 className="text-xl font-medium mb-2 text-zed-600 dark:text-zed-300">
                  No Code Snippets Yet
                </h3>
                <p className="text-zed-500 dark:text-zed-400 max-w-md">
                  Code snippets from your conversation will appear here for easy
                  reference.
                </p>
              </div>
            ) : (
              <div className="space-y-6">
                {codeSnippets.map((snippet, index) => (
                  <CodeSnippet key={index} snippet={snippet} />
                ))}
              </div>
            )}
          </TabsContent>

          {/* Files Tab */}
          {/* NOTE: The original code had nested TabsContent for files/graph *inside* the code tab. This is likely incorrect. I'm moving them to be direct children of the main Tabs component. */}
          <TabsContent value="files" className="flex-1 p-4 overflow-hidden">
            <Card className="h-full overflow-hidden border-zed-200 dark:border-zed-800">
              <FileExplorer
                fileStructure={fileStructure}
                isLoading={isLoading}
                onDirectorySelectRequest={handleSelectDirectory} // Pass the handler
                onNodeSelect={handleNodeSelect} // Pass the node selection handler
                selectedDirectoryPath={selectedDirectory} // Pass the path for context
                onFileContentRequest={getFileContent}
              />
            </Card>
          </TabsContent>

          {/* Graph Tab */}
          <TabsContent value="graph" className="flex-1 p-4 overflow-hidden">
            <Card className="h-full overflow-hidden border-zed-200 dark:border-zed-800">
              <CodeGraph codeGraph={codeGraph} isLoading={isLoading} />
            </Card>
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
}
