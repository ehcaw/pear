import type { ReactNode } from "react";
import React from "react"; // Make sure to import React

export function extractCodeBlocks(content: string): {
  textContent: ReactNode[];
  hasCode: boolean;
} {
  // Simple regex to find markdown code blocks
  const codeBlockRegex = /```(\w+)?\n([\s\S]*?)```/g;
  const parts: ReactNode[] = [];
  let lastIndex = 0;
  let hasCode = false;
  let match: RegExpExecArray | null;

  // Find all code blocks
  while ((match = codeBlockRegex.exec(content)) !== null) {
    // Add text before code block
    if (match.index > lastIndex) {
      parts.push(content.substring(lastIndex, match.index));
    }

    // Add code block
    const language = match[1] || "plaintext";
    const code = match[2];
    hasCode = true;

    // Use proper key assignment
    const matchIndex = match.index; // Store the index locally
    parts.push(
      // React element with properly typed key
      React.createElement(
        "div",
        {
          key: `code-block-${matchIndex}`,
          className: "my-2 overflow-x-auto",
        },
        [
          React.createElement(
            "div",
            {
              key: "language",
              className:
                "text-xs px-2 py-1 bg-zed-50 dark:bg-zed-900 text-zed-600 dark:text-zed-300 border-t border-x border-zed-200 dark:border-zed-800 rounded-t-md",
            },
            language,
          ),
          React.createElement(
            "pre",
            {
              key: "code",
              className:
                "p-3 text-sm bg-white dark:bg-gray-900 border border-zed-200 dark:border-zed-800 rounded-b-md",
            },
            React.createElement("code", {}, code),
          ),
        ],
      ),
    );

    lastIndex = match.index + match[0].length;
  }

  // Add remaining text
  if (lastIndex < content.length) {
    parts.push(content.substring(lastIndex));
  }

  return { textContent: parts, hasCode };
}
