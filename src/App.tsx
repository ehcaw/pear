"use client";

import { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Code, MessageSquare, Sparkles, Terminal, Zap } from "lucide-react";
import { ConversationBubble } from "@/components/conversation-bubble";
import { VoiceControls } from "@/components/voice-controls";
import { useConversation } from "@/hooks/use-conversation";
import { PearLogo } from "@/components/pear-logo";
import { WaveBackground } from "@/components/wave-background";
import { CodeSnippet } from "@/components/code-snippet";

export default function PearInterface() {
  const {
    messages,
    isListening,
    isSpeaking,
    startListening,
    stopListening,
    toggleMute,
    isMuted,
    currentTranscription,
    clearConversation,
    codeSnippets,
    handleUserMessage,
  } = useConversation();

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [activeTab, setActiveTab] = useState("conversation");

  useEffect(() => {
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages]);

  return (
    <div className="flex flex-col h-screen bg-cream-50 dark:bg-gray-950 overflow-hidden">
      <WaveBackground />

      <header className="relative z-10 border-b border-zed-100 dark:border-zed-800 py-3 px-4 flex items-center justify-between bg-cream-50/80 backdrop-blur-md dark:bg-gray-900/80">
        <div className="flex items-center gap-3">
          <div className="relative">
            <PearLogo className="h-8 w-8" />
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
      </header>

      <main className="flex-1 flex flex-col md:flex-row overflow-hidden relative z-10">
        <Tabs
          value={activeTab}
          onValueChange={setActiveTab}
          className="flex-1 flex flex-col h-full"
        >
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
            </TabsList>
          </div>

          <TabsContent
            value="conversation"
            className="flex-1 flex flex-col p-4 overflow-hidden"
          >
            <div className="flex-1 overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-zed-200 scrollbar-track-transparent dark:scrollbar-thumb-zed-700">
              {messages.length === 0 ? (
                <div className="h-full flex flex-col items-center justify-center text-center p-8">
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
                  {currentTranscription && (
                    <ConversationBubble
                      message={{
                        role: "user",
                        content: currentTranscription,
                        pending: true,
                      }}
                      isLast={false}
                    />
                  )}
                  <div ref={messagesEndRef} />
                </div>
              )}
            </div>

            <Card className="mt-4 border-zed-200 dark:border-zed-800 bg-white/80 backdrop-blur-sm dark:bg-gray-800/80">
              <CardContent className="p-4">
                <VoiceControls
                  isListening={isListening}
                  isSpeaking={isSpeaking}
                  isMuted={isMuted}
                  onStartListening={startListening}
                  onStopListening={stopListening}
                  onToggleMute={toggleMute}
                />
              </CardContent>
            </Card>
          </TabsContent>

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
        </Tabs>
      </main>
    </div>
  );
}
