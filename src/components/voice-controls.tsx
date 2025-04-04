"use client";

import type React from "react";

import { Button } from "@/components/ui/button";
import {
  Mic,
  MicOff,
  Volume2,
  VolumeX,
  Keyboard,
  Sparkles,
} from "lucide-react";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useState } from "react";
import { cn } from "@/lib/utils";

interface VoiceControlsProps {
  isListening: boolean;
  isSpeaking: boolean;
  isMuted: boolean;
  onStartListening: () => void;
  onStopListening: () => void;
  onToggleMute: () => void;
}

export function VoiceControls({
  isListening,
  isSpeaking,
  isMuted,
  onStartListening,
  onStopListening,
  onToggleMute,
}: VoiceControlsProps) {
  const [showTextInput, setShowTextInput] = useState(false);
  const [textInput, setTextInput] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (textInput.trim()) {
      // Handle text submission
      setTextInput("");
    }
  };

  return (
    <div className="flex flex-col gap-4">
      {showTextInput && (
        <form onSubmit={handleSubmit} className="flex gap-2">
          <div className="relative flex-1">
            <input
              type="text"
              value={textInput}
              onChange={(e) => setTextInput(e.target.value)}
              placeholder="Type your message..."
              className="w-full px-4 py-2 pr-10 rounded-full border border-zed-200 focus:border-zed-400 focus:ring-1 focus:ring-zed-400 focus:outline-none dark:bg-gray-800 dark:border-zed-700 dark:focus:border-zed-500 dark:focus:ring-zed-500"
            />
            <Button
              type="submit"
              size="icon"
              className="absolute right-1 top-1 h-8 w-8 rounded-full bg-zed-500 hover:bg-zed-600 text-white"
              disabled={!textInput.trim()}
            >
              <Sparkles className="h-4 w-4" />
            </Button>
          </div>
          <Button
            type="button"
            size="icon"
            variant="outline"
            className="rounded-full border-zed-200 bg-black dark:border-zed-700"
            onClick={() => setShowTextInput(false)}
          >
            <Mic className="h-4 w-4 text-zed-500" />
          </Button>
        </form>
      )}

      {!showTextInput && (
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant={isListening ? "destructive" : "default"}
                    size="lg"
                    onClick={isListening ? onStopListening : onStartListening}
                    className={cn(
                      "rounded-full h-12 w-12 flex items-center justify-center",
                      isListening
                        ? "bg-red-500 hover:bg-red-600 animate-pulse"
                        : "bg-zed-500 hover:bg-zed-600",
                    )}
                  >
                    {isListening ? (
                      <MicOff className="h-5 w-5 text-white" />
                    ) : (
                      <Mic className="h-5 w-5 text-white" />
                    )}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  {isListening ? "Stop listening" : "Start listening"}
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>

            <div className="flex gap-2">
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={onToggleMute}
                      className="rounded-full border-zed-200 dark:border-zed-700"
                    >
                      {isMuted ? (
                        <VolumeX className="h-4 w-4 text-zed-500" />
                      ) : (
                        <Volume2 className="h-4 w-4 text-zed-500" />
                      )}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>{isMuted ? "Unmute" : "Mute"}</TooltipContent>
                </Tooltip>
              </TooltipProvider>

              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={() => setShowTextInput(true)}
                      className="rounded-full border-zed-200 dark:border-zed-700"
                    >
                      <Keyboard className="h-4 w-4 text-zed-500" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Type instead</TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </div>
          </div>

          <div className="flex items-center gap-3">
            {isListening && (
              <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800">
                <span className="relative flex h-2 w-2">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75"></span>
                  <span className="relative inline-flex rounded-full h-2 w-2 bg-red-500"></span>
                </span>
                <span className="text-xs font-medium text-red-600 dark:text-red-400">
                  Listening...
                </span>
              </div>
            )}

            {isSpeaking && (
              <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-zed-50 dark:bg-zed-900/30 border border-zed-200 dark:border-zed-800">
                <div className="flex space-x-1">
                  <span
                    className="w-1 h-2 bg-zed-400 rounded-full animate-bounce"
                    style={{ animationDelay: "0ms" }}
                  ></span>
                  <span
                    className="w-1 h-2 bg-zed-500 rounded-full animate-bounce"
                    style={{ animationDelay: "150ms" }}
                  ></span>
                  <span
                    className="w-1 h-2 bg-zed-600 rounded-full animate-bounce"
                    style={{ animationDelay: "300ms" }}
                  ></span>
                </div>
                <span className="text-xs font-medium text-zed-600 dark:text-zed-400">
                  Speaking...
                </span>
              </div>
            )}
          </div>
        </div>
      )}

      <div className="text-xs text-center text-zed-400 dark:text-zed-500">
        {isListening
          ? "I'm listening... speak clearly into your microphone"
          : "Click the microphone button or press Space to start speaking"}
      </div>
    </div>
  );
}
