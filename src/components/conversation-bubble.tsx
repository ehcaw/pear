"use client";

import { cn } from "@/lib/utils";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Card, CardContent } from "@/components/ui/card";
import { Loader2, Copy, Check } from "lucide-react";
import { useState } from "react";
import { PearLogo } from "./pear-logo";
import { Button } from "@/components/ui/button";
import { extractCodeBlocks } from "@/lib/code-utils";

interface Message {
  role: "user" | "assistant";
  content: string;
  pending?: boolean;
}

interface ConversationBubbleProps {
  message: Message;
  isLast: boolean;
}

export function ConversationBubble({
  message,
  isLast,
}: ConversationBubbleProps) {
  const isUser = message.role === "user";
  const [copied, setCopied] = useState(false);

  const copyToClipboard = () => {
    navigator.clipboard.writeText(message.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  // Process message content to render code blocks
  const { textContent, hasCode } = extractCodeBlocks(message.content);

  return (
    <div
      className={cn(
        "group flex gap-3",
        isUser ? "flex-row-reverse" : "flex-row",
      )}
    >
      <div className="mt-1">
        {isUser ? (
          <Avatar className="h-8 w-8 bg-gradient-to-br from-zed-400 to-zed-600 border-2 border-white dark:border-gray-800">
            <AvatarFallback className="text-white font-medium">
              You
            </AvatarFallback>
          </Avatar>
        ) : (
          <div className="h-8 w-8 relative">
            <PearLogo className="h-8 w-8" />
            {isLast && (
              <span className="absolute -right-1 -top-1 flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-zed-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-2 w-2 bg-zed-500"></span>
              </span>
            )}
          </div>
        )}
      </div>

      <div className={cn("max-w-[85%]", isUser ? "items-end" : "items-start")}>
        <Card
          className={cn(
            "relative overflow-hidden transition-all",
            isUser
              ? "bg-gradient-to-br from-zed-50 to-zed-100 dark:from-zed-900 dark:to-zed-800 border-zed-200 dark:border-zed-700"
              : "bg-white dark:bg-gray-800 border-cream-200 dark:border-gray-700",
            message.pending && "opacity-70",
          )}
        >
          <CardContent className={cn("p-3 text-sm", hasCode ? "pb-1" : "")}>
            <div className="prose prose-sm max-w-none dark:prose-invert prose-p:leading-relaxed prose-pre:my-0">
              {textContent}
            </div>

            {message.pending && (
              <div className="flex items-center mt-1">
                <Loader2 className="h-3 w-3 mr-2 animate-spin text-zed-500" />
                <span className="text-xs text-zed-500">Typing...</span>
              </div>
            )}

            {isLast && !isUser && !message.pending && (
              <div className="flex space-x-1 mt-1">
                <span
                  className="w-1 h-3 bg-zed-400 rounded-full animate-bounce"
                  style={{ animationDelay: "0ms" }}
                ></span>
                <span
                  className="w-1 h-3 bg-zed-500 rounded-full animate-bounce"
                  style={{ animationDelay: "150ms" }}
                ></span>
                <span
                  className="w-1 h-3 bg-zed-600 rounded-full animate-bounce"
                  style={{ animationDelay: "300ms" }}
                ></span>
              </div>
            )}
          </CardContent>

          <div
            className={cn(
              "absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity",
              copied && "opacity-100",
            )}
          >
            <Button
              size="icon"
              variant="ghost"
              className="h-6 w-6 rounded-full bg-white/80 dark:bg-gray-800/80 hover:bg-white dark:hover:bg-gray-700"
              onClick={copyToClipboard}
            >
              {copied ? (
                <Check className="h-3 w-3 text-green-500" />
              ) : (
                <Copy className="h-3 w-3 text-zed-500" />
              )}
            </Button>
          </div>
        </Card>

        <div
          className={cn(
            "text-xs text-zed-400 dark:text-zed-500 mt-1",
            isUser ? "text-right mr-1" : "ml-1",
          )}
        >
          {isUser ? "You" : "Pear"} • Just now
        </div>
      </div>
    </div>
  );
}
