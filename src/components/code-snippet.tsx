"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Copy, Check, Code } from "lucide-react";
import { useState } from "react";
import { cn } from "@/lib/utils";

interface CodeSnippetProps {
  snippet: {
    title: string;
    language: string;
    code: string;
  };
}

export function CodeSnippet({ snippet }: CodeSnippetProps) {
  const [copied, setCopied] = useState(false);

  const copyToClipboard = () => {
    navigator.clipboard.writeText(snippet.code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Card className="border-zed-200 dark:border-zed-800 overflow-hidden">
      <CardHeader className="bg-cream-50 dark:bg-gray-900 py-3 px-4 flex flex-row items-center justify-between">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <Code className="h-4 w-4 text-zed-500" />
          <span>{snippet.title}</span>
          <span className="text-xs px-2 py-0.5 rounded-full bg-zed-100 text-zed-600 dark:bg-zed-900 dark:text-zed-300">
            {snippet.language}
          </span>
        </CardTitle>
        <Button
          size="sm"
          variant="ghost"
          className="h-8 w-8 p-0 rounded-full hover:bg-zed-100 dark:hover:bg-zed-800"
          onClick={copyToClipboard}
        >
          {copied ? (
            <Check className="h-4 w-4 text-green-500" />
          ) : (
            <Copy className="h-4 w-4 text-zed-500" />
          )}
        </Button>
      </CardHeader>
      <CardContent className="p-0 overflow-x-auto">
        <pre
          className={cn(
            "p-4 text-sm font-mono bg-white dark:bg-gray-800",
            "border-t border-zed-100 dark:border-zed-800",
          )}
        >
          <code>{snippet.code}</code>
        </pre>
      </CardContent>
    </Card>
  );
}
