import type { FileNode } from "../components/file-explorer";
import type {
  CodeGraphData,
  GraphNode,
  GraphLink,
} from "../components/code-graph";

// Helper to generate a unique ID
const generateId = () => Math.random().toString(36).substring(2, 9);

// Generate mock file structure
export function generateMockFileStructure(basePath: string): FileNode[] {
  const parts = basePath.split("/");
  const projectName = parts[parts.length - 1];

  // Common file structure for a typical project
  return [
    {
      id: generateId(),
      name: "src",
      type: "directory",
      path: `${basePath}/src`,
      children: [
        {
          id: generateId(),
          name: "components",
          type: "directory",
          path: `${basePath}/src/components`,
          children: [
            {
              id: generateId(),
              name: "Button.tsx",
              type: "file",
              path: `${basePath}/src/components/Button.tsx`,
              language: "tsx",
              size: 2048,
            },
            {
              id: generateId(),
              name: "Card.tsx",
              type: "file",
              path: `${basePath}/src/components/Card.tsx`,
              language: "tsx",
              size: 1536,
            },
            {
              id: generateId(),
              name: "Input.tsx",
              type: "file",
              path: `${basePath}/src/components/Input.tsx`,
              language: "tsx",
              size: 1024,
            },
          ],
        },
        {
          id: generateId(),
          name: "hooks",
          type: "directory",
          path: `${basePath}/src/hooks`,
          children: [
            {
              id: generateId(),
              name: "useAuth.ts",
              type: "file",
              path: `${basePath}/src/hooks/useAuth.ts`,
              language: "ts",
              size: 3072,
            },
            {
              id: generateId(),
              name: "useForm.ts",
              type: "file",
              path: `${basePath}/src/hooks/useForm.ts`,
              language: "ts",
              size: 2560,
            },
          ],
        },
        {
          id: generateId(),
          name: "utils",
          type: "directory",
          path: `${basePath}/src/utils`,
          children: [
            {
              id: generateId(),
              name: "api.ts",
              type: "file",
              path: `${basePath}/src/utils/api.ts`,
              language: "ts",
              size: 1792,
            },
            {
              id: generateId(),
              name: "helpers.ts",
              type: "file",
              path: `${basePath}/src/utils/helpers.ts`,
              language: "ts",
              size: 1280,
            },
          ],
        },
        {
          id: generateId(),
          name: "App.tsx",
          type: "file",
          path: `${basePath}/src/App.tsx`,
          language: "tsx",
          size: 4096,
        },
        {
          id: generateId(),
          name: "main.tsx",
          type: "file",
          path: `${basePath}/src/main.tsx`,
          language: "tsx",
          size: 512,
        },
      ],
    },
    {
      id: generateId(),
      name: "public",
      type: "directory",
      path: `${basePath}/public`,
      children: [
        {
          id: generateId(),
          name: "index.html",
          type: "file",
          path: `${basePath}/public/index.html`,
          language: "html",
          size: 1024,
        },
        {
          id: generateId(),
          name: "favicon.ico",
          type: "file",
          path: `${basePath}/public/favicon.ico`,
          size: 4096,
        },
      ],
    },
    {
      id: generateId(),
      name: "package.json",
      type: "file",
      path: `${basePath}/package.json`,
      language: "json",
      size: 2048,
    },
    {
      id: generateId(),
      name: "tsconfig.json",
      type: "file",
      path: `${basePath}/tsconfig.json`,
      language: "json",
      size: 1536,
    },
    {
      id: generateId(),
      name: "README.md",
      type: "file",
      path: `${basePath}/README.md`,
      language: "md",
      size: 3072,
    },
  ];
}

// Generate mock code graph data
export function generateMockCodeGraph(basePath: string): CodeGraphData {
  // Create nodes for classes, methods, and functions
  const nodes: GraphNode[] = [
    // Classes
    { id: "App", name: "App", type: "class", group: "component" },
    { id: "Button", name: "Button", type: "class", group: "component" },
    { id: "Card", name: "Card", type: "class", group: "component" },
    { id: "Input", name: "Input", type: "class", group: "component" },
    {
      id: "AuthProvider",
      name: "AuthProvider",
      type: "class",
      group: "provider",
    },

    // Methods
    { id: "App.render", name: "render", type: "method", group: "component" },
    { id: "Button.render", name: "render", type: "method", group: "component" },
    { id: "Card.render", name: "render", type: "method", group: "component" },
    { id: "Input.render", name: "render", type: "method", group: "component" },
    {
      id: "Button.handleClick",
      name: "handleClick",
      type: "method",
      group: "component",
    },
    {
      id: "Input.handleChange",
      name: "handleChange",
      type: "method",
      group: "component",
    },
    {
      id: "AuthProvider.login",
      name: "login",
      type: "method",
      group: "provider",
    },
    {
      id: "AuthProvider.logout",
      name: "logout",
      type: "method",
      group: "provider",
    },

    // Functions
    { id: "useAuth", name: "useAuth", type: "function", group: "hook" },
    { id: "useForm", name: "useForm", type: "function", group: "hook" },
    {
      id: "api.fetchData",
      name: "fetchData",
      type: "function",
      group: "utility",
    },
    {
      id: "api.postData",
      name: "postData",
      type: "function",
      group: "utility",
    },
    {
      id: "helpers.formatDate",
      name: "formatDate",
      type: "function",
      group: "utility",
    },

    // Variables
    { id: "API_URL", name: "API_URL", type: "variable", group: "constant" },
    {
      id: "DEFAULT_THEME",
      name: "DEFAULT_THEME",
      type: "variable",
      group: "constant",
    },
  ];

  // Create links between nodes
  const links: GraphLink[] = [
    // Class containment
    { source: "App", target: "App.render", type: "contains" },
    { source: "Button", target: "Button.render", type: "contains" },
    { source: "Button", target: "Button.handleClick", type: "contains" },
    { source: "Card", target: "Card.render", type: "contains" },
    { source: "Input", target: "Input.render", type: "contains" },
    { source: "Input", target: "Input.handleChange", type: "contains" },
    { source: "AuthProvider", target: "AuthProvider.login", type: "contains" },
    { source: "AuthProvider", target: "AuthProvider.logout", type: "contains" },

    // Component imports
    { source: "App", target: "Button", type: "imports" },
    { source: "App", target: "Card", type: "imports" },
    { source: "App", target: "Input", type: "imports" },

    // Function calls
    { source: "App.render", target: "useAuth", type: "calls" },
    { source: "App.render", target: "useForm", type: "calls" },
    { source: "Button.handleClick", target: "api.postData", type: "calls" },
    {
      source: "Input.handleChange",
      target: "helpers.formatDate",
      type: "calls",
    },
    { source: "AuthProvider.login", target: "api.postData", type: "calls" },
    { source: "AuthProvider.logout", target: "api.postData", type: "calls" },
    { source: "useAuth", target: "AuthProvider.login", type: "calls" },
    { source: "useAuth", target: "AuthProvider.logout", type: "calls" },
    { source: "api.fetchData", target: "API_URL", type: "calls" },
    { source: "api.postData", target: "API_URL", type: "calls" },
  ];

  return { nodes, links };
}
