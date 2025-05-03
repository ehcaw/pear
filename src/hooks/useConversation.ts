"use client";

import { invoke } from "@tauri-apps/api/core";

import { useState, useCallback, useEffect } from "react";

interface Message {
  role: "user" | "assistant";
  content: string;
  pending?: boolean;
  timestamp: number;
}

interface CodeSnippet {
  title: string;
  language: string;
  code: string;
}

export function useConversation() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [isMuted, setIsMuted] = useState(false);
  const [codeSnippets, setCodeSnippets] = useState<CodeSnippet[]>([]);

  // Since VAD is handled in the App component, these are simplified
  const [isDetectingSpeech, setIsDetectingSpeech] = useState(false);
  const [isUserCurrentlySpeaking, setIsUserCurrentlySpeaking] = useState(false);
  const [activeAudio, setActiveAudio] = useState<String>("");

  // Function to handle audio from VAD
  const handleSpeechEnd = useCallback(
    async (audio: Float32Array, currentDirectory: string) => {
      setIsUserCurrentlySpeaking(false);
      setIsProcessing(true);
      if (activeAudio) {
        await invoke("stop_audio", { playbackId: activeAudio });
        await invoke("delete_audio_player_file", { playbackId: activeAudio });
        setActiveAudio("");
      }
      const audioArray = Array.from(audio);
      try {
        // const transcribe_generate_play_response = await invoke(
        //   "transcribe_generate_play",
        //   {
        //     audio: audioArray,
        //     currentDirectory: currentDirectory,
        //   },
        // );
        // const parsed = JSON.parse(transcribe_generate_play_response as string);
        // const { audio_path, transcription, llm_response } = parsed;
        // setActiveAudio(audio_path);
        // //const audio = new Audio(audio_path);
        // const playbackId: string = await invoke("play_audio_file", {
        //   path: audio_path,
        // });
        // await cleanupAudio(playbackId);
        // if (!transcription.trim()) {
        //   setIsProcessing(false);
        //   return;
        const transcription = "bruh";
        const llm_response = "bruh";
        // Add user message to conversation
        const newUserMessage: Message = {
          role: "user",
          content: transcription,
          timestamp: Date.now(),
        };
        const updatedMessages = [...messages, newUserMessage];
        setMessages(updatedMessages);

        // Get AI response
        setIsSpeaking(true);

        // Add assistant message to conversation
        setMessages((prev) => [
          ...prev,
          {
            role: "assistant",
            content: llm_response,
            timestamp: Date.now(),
          },
        ]);
      } catch (error) {
        console.error("Error processing speech:", error);
        setMessages((prev) => [
          ...prev,
          {
            role: "assistant",
            content: `Sorry, I encountered an error: ${error}`,
            timestamp: Date.now(),
          },
        ]);
      } finally {
        setIsProcessing(false);
        setIsSpeaking(false);
      }
    },
    [messages],
  );

  // Simple function to indicate user started speaking
  const handleSpeechStart = useCallback(() => {
    setIsUserCurrentlySpeaking(true);
  }, []);

  // Toggle mute state
  const toggleMute = useCallback(() => {
    setIsMuted((prev) => !prev);
  }, []);

  // Clear conversation
  const clearConversation = useCallback(() => {
    setMessages([]);
    setCodeSnippets([]);
    setIsProcessing(false);
    setIsSpeaking(false);
    setIsUserCurrentlySpeaking(false);
  }, []);

  // Start/stop speech detection (lightweight wrappers for App.tsx)
  const startSpeechDetection = useCallback(() => {
    setIsDetectingSpeech(true);
  }, []);

  const stopSpeechDetection = useCallback(() => {
    setIsDetectingSpeech(false);
  }, []);

  // Extract code snippets from messages
  useEffect(() => {
    const newSnippets: CodeSnippet[] = [];
    messages.forEach((message) => {
      const codeBlockRegex = /```(\w+)?\n([\s\S]*?)```/g;
      let match;
      while ((match = codeBlockRegex.exec(message.content)) !== null) {
        const language = match[1] || "plaintext";
        const code = match[2].trim();
        let title = `${language} Snippet`;
        newSnippets.push({ title, language, code });
      }
    });

    if (JSON.stringify(newSnippets) !== JSON.stringify(codeSnippets)) {
      setCodeSnippets(newSnippets);
    }
  }, [messages, codeSnippets]);

  async function stopAudio(playbackId: string) {
    return invoke("stop_audio", { playbackId });
  }

  async function pauseAudio(playbackId: string) {
    return invoke("pause_audio", { playbackId });
  }

  async function resumeAudio(playbackId: string) {
    return invoke("resume_audio", { playbackId });
  }

  async function checkIfPlaying(playbackId: string) {
    return invoke("is_audio_playing", { playbackId });
  }

  // Cleanup function to delete the audio file when no longer needed
  async function cleanupAudio(audioPath: string) {
    return invoke("delete_audio_player_file", { path: audioPath });
  }

  return {
    messages,
    isDetectingSpeech,
    isUserCurrentlySpeaking,
    isProcessing,
    isSpeaking,
    isMuted,
    codeSnippets,
    startSpeechDetection,
    stopSpeechDetection,
    handleSpeechStart,
    handleSpeechEnd,
    toggleMute,
    clearConversation,
  };
}
