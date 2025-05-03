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
  isDetectingSpeech: boolean; // Is the VAD system active?
  isUserCurrentlySpeaking: boolean; // Is the user audibly speaking right now?
  isProcessing: boolean; // Is the backend transcribing/generating response?
  isAssistantSpeaking: boolean; // Is the AI currently playing TTS? (Optional, but good for UI feedback)
  isMuted: boolean; // Is TTS output muted?
  onToggleSpeechDetection: () => void; // Function to toggle VAD on/off
  onToggleMute: () => void; // Function to toggle TTS mute
  onTextSubmit: (text: string) => void; // Function to handle text input submission
}

export function VoiceControls({
  isDetectingSpeech,
  isUserCurrentlySpeaking,
  isProcessing,
  isAssistantSpeaking,
  isMuted,
  onToggleSpeechDetection,
  onToggleMute,
  onTextSubmit, // Added prop for text submission
}: VoiceControlsProps) {
  const [showTextInput, setShowTextInput] = useState(false);
  const [textInput, setTextInput] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmedInput = textInput.trim();
    if (trimmedInput && !isProcessing && !isAssistantSpeaking) {
      onTextSubmit(trimmedInput); // Call the handler from props
      setTextInput(""); // Clear input after submission
      setShowTextInput(false); // Optionally switch back to voice controls
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
              placeholder={
                isProcessing ? "Processing..." : "Type your message..."
              }
              className="w-full px-4 py-2 pr-10 rounded-full border border-zed-200 focus:border-zed-400 focus:ring-1 focus:ring-zed-400 focus:outline-none dark:bg-gray-800 dark:border-zed-700 dark:focus:border-zed-500 dark:focus:ring-zed-500 disabled:opacity-50"
              disabled={isProcessing || isAssistantSpeaking}
            />
            <Button
              type="submit"
              size="icon"
              className="absolute right-1 top-1 h-8 w-8 rounded-full bg-zed-500 hover:bg-zed-600 text-white disabled:opacity-50"
              disabled={
                !textInput.trim() || isProcessing || isAssistantSpeaking
              }
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
            disabled={isProcessing || isAssistantSpeaking}
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
                    variant={isDetectingSpeech ? "destructive" : "default"}
                    size="lg"
                    onClick={onToggleSpeechDetection}
                    disabled={isProcessing || isAssistantSpeaking} // Disable while processing/AI speaking
                    className={cn(
                      "rounded-full h-12 w-12 flex items-center justify-center transition-opacity",
                      isDetectingSpeech
                        ? "bg-red-500 hover:bg-red-600 animate-pulse"
                        : "bg-zed-500 hover:bg-zed-600",
                      (isProcessing || isAssistantSpeaking) &&
                        "opacity-50 cursor-not-allowed", // Style when disabled
                    )}
                  >
                    {isDetectingSpeech ? (
                      <MicOff className="h-5 w-5 text-white" />
                    ) : (
                      <Mic className="h-5 w-5 text-white" />
                    )}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  {isProcessing
                    ? "Processing..."
                    : isAssistantSpeaking
                      ? "Assistant speaking..."
                      : isDetectingSpeech
                        ? "Stop detection"
                        : "Start detection"}
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
                      disabled={isProcessing || isAssistantSpeaking}
                      className="rounded-full border-zed-200 dark:border-zed-700 disabled:opacity-50"
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
                      disabled={
                        isDetectingSpeech || isProcessing || isAssistantSpeaking
                      } // Also disable if detecting speech
                      className="rounded-full border-zed-200 dark:border-zed-700 disabled:opacity-50"
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
            {isProcessing && (
              <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-yellow-50 dark:bg-yellow-900/30 border border-yellow-200 dark:border-yellow-800">
                {/* Basic spinner example */}
                <div className="animate-spin rounded-full h-3 w-3 border-b-2 border-yellow-600 dark:border-yellow-400"></div>
                <span className="text-xs font-medium text-yellow-600 dark:text-yellow-400">
                  Processing...
                </span>
              </div>
            )}

            {/* Show listening only when detecting AND not processing AND user isn't speaking */}
            {isDetectingSpeech && !isProcessing && !isUserCurrentlySpeaking && (
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

            {/* Show speaking only when user is speaking AND not processing */}
            {isUserCurrentlySpeaking && !isProcessing && (
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

      <div className="text-xs text-center text-zed-400 dark:text-zed-500 h-4">
        {" "}
        {/* Added fixed height */}
        {isProcessing
          ? "Processing your request..."
          : isAssistantSpeaking
            ? "Assistant is speaking..."
            : isDetectingSpeech
              ? isUserCurrentlySpeaking
                ? "I hear you..."
                : "Listening for your voice..."
              : "Click the microphone or press Space to start voice detection"}
      </div>
    </div>
  );
}
