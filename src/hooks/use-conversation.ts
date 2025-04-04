"use client";

import { useState, useEffect, useCallback } from "react";

interface Message {
  role: "user" | "assistant";
  content: string;
  pending?: boolean;
}

interface CodeSnippet {
  title: string;
  language: string;
  code: string;
}

export function useConversation() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isListening, setIsListening] = useState(false);
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [isMuted, setIsMuted] = useState(false);
  const [currentTranscription, setCurrentTranscription] = useState("");
  const [codeSnippets, setCodeSnippets] = useState<CodeSnippet[]>([]);

  // Extract code snippets from messages
  useEffect(() => {
    const extractCodeSnippets = () => {
      const newSnippets: CodeSnippet[] = [];

      messages.forEach((message) => {
        // Simple regex to find markdown code blocks
        const codeBlockRegex = /```(\w+)?\n([\s\S]*?)```/g;
        let match;

        while ((match = codeBlockRegex.exec(message.content)) !== null) {
          const language = match[1] || "plaintext";
          const code = match[2];

          // Create a title based on the code content
          let title = "Code Snippet";
          if (language === "jsx" || language === "tsx") {
            // Try to extract component name for React code
            const componentMatch = code.match(/function\s+([A-Z][a-zA-Z0-9]*)/);
            if (componentMatch) {
              title = componentMatch[1];
            } else {
              title = "React Component";
            }
          } else if (language === "javascript" || language === "js") {
            // Try to extract function name
            const functionMatch = code.match(
              /function\s+([a-zA-Z][a-zA-Z0-9]*)/,
            );
            if (functionMatch) {
              title = functionMatch[1] + "()";
            } else {
              title = "JavaScript Function";
            }
          }

          newSnippets.push({
            title,
            language,
            code,
          });
        }
      });

      setCodeSnippets(newSnippets);
    };

    extractCodeSnippets();
  }, [messages]);

  // Mock function for starting the listening process
  const startListening = useCallback(() => {
    setIsListening(true);
    setCurrentTranscription("");

    // This would be replaced with actual Groq transcription API
    const mockTranscriptionInterval = setInterval(() => {
      setCurrentTranscription((prev) => {
        const words = [
          "Can",
          "you",
          "help",
          "me",
          "create",
          "a",
          "React",
          "component",
          "for",
          "a",
          "user",
          "profile",
          "page?",
        ];
        const currentWords = prev.split(" ").filter(Boolean);
        const nextWordIndex = currentWords.length;

        if (nextWordIndex >= words.length) {
          clearInterval(mockTranscriptionInterval);
          setTimeout(() => {
            setIsListening(false);
            handleUserMessage(prev);
          }, 500);
          return prev;
        }

        return prev + (prev ? " " : "") + words[nextWordIndex];
      });
    }, 300);

    return () => clearInterval(mockTranscriptionInterval);
  }, []);

  const stopListening = useCallback(() => {
    setIsListening(false);
    if (currentTranscription) {
      handleUserMessage(currentTranscription);
    }
    setCurrentTranscription("");
  }, [currentTranscription]);

  const handleUserMessage = useCallback((content: string) => {
    const userMessage: Message = { role: "user", content };
    setMessages((prev) => [...prev, userMessage]);
    setCurrentTranscription("");

    // Simulate AI response
    setTimeout(() => {
      setIsSpeaking(true);

      // This would be replaced with actual AI response and TTS
      setTimeout(() => {
        const aiResponse: Message = {
          role: "assistant",
          content: `I'd be happy to help you create a React component for a user profile page! Here's a clean, modern implementation:

\`\`\`tsx
import React from 'react';
import { Avatar } from './Avatar';
import { Badge } from './Badge';

interface UserProfileProps {
  user: {
    name: string;
    role: string;
    bio: string;
    avatar: string;
    email: string;
    location: string;
    skills: string[];
    joinDate: string;
  }
}

export function UserProfile({ user }: UserProfileProps) {
  return (
    <div className="bg-white rounded-lg shadow-md overflow-hidden">
      <div className="bg-gradient-to-r from-blue-500 to-purple-600 h-32" />
      <div className="px-6 py-4 relative">
        <div className="absolute -top-16 left-6 border-4 border-white rounded-full overflow-hidden">
          <Avatar
            src={user.avatar}
            alt={user.name}
            size="large"
          />
        </div>

        <div className="mt-16">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold">{user.name}</h1>
              <p className="text-gray-600">{user.role}</p>
            </div>
            <button className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600">
              Edit Profile
            </button>
          </div>

          <p className="mt-4 text-gray-700">{user.bio}</p>

          <div className="mt-6 grid grid-cols-2 gap-4">
            <div>
              <h3 className="text-sm font-medium text-gray-500">Email</h3>
              <p className="mt-1">{user.email}</p>
            </div>
            <div>
              <h3 className="text-sm font-medium text-gray-500">Location</h3>
              <p className="mt-1">{user.location}</p>
            </div>
            <div>
              <h3 className="text-sm font-medium text-gray-500">Joined</h3>
              <p className="mt-1">{user.joinDate}</p>
            </div>
          </div>

          <div className="mt-6">
            <h3 className="text-sm font-medium text-gray-500">Skills</h3>
            <div className="mt-2 flex flex-wrap gap-2">
              {user.skills.map((skill) => (
                <Badge key={skill}>{skill}</Badge>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
\`\`\`

Would you like me to explain any part of this component in more detail?`,
        };

        setMessages((prev) => [...prev, aiResponse]);
        setIsSpeaking(false);
      }, 3000);
    }, 1000);
  }, []);

  const toggleMute = useCallback(() => {
    setIsMuted((prev) => !prev);
  }, []);

  const clearConversation = useCallback(() => {
    setMessages([]);
    setCurrentTranscription("");
    setIsListening(false);
    setIsSpeaking(false);
    setCodeSnippets([]);
  }, []);

  return {
    messages,
    isListening,
    isSpeaking,
    isMuted,
    currentTranscription,
    codeSnippets,
    startListening,
    stopListening,
    toggleMute,
    clearConversation,
    handleUserMessage,
  };
}
