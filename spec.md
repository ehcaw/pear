Project Specification: Tauri Codebase Analyzer & Neo4j Ingestor
1. Project Goal:

To create a cross-platform desktop application using Tauri that allows users to select a local codebase directory, parses the source code using tree-sitter, and ingests a graph representation of the code structure (files, functions, classes, calls, imports, etc.) into a Neo4j graph database for analysis.

2. Core Technology Stack:

Application Framework: Tauri (Rust backend, Webview frontend)

Backend Language: Rust

Parsing Library: tree-sitter (Rust core bindings + language grammar crates)

Database: Neo4j (compatible with AuraDB or local instances)

Neo4j Driver (Rust): neo4rs

Frontend: TypeScript/JavaScript (Specific framework like React, Vue, Svelte is optional, vanilla TS/JS is acceptable)

Directory Traversal: walkdir crate (Rust)

Configuration: .env file (via dotenvy crate in Rust backend) for Neo4j credentials during development.

3. Key Features:

Directory Selection: Provide a user interface element (e.g., a button) that uses Tauri's dialog API to allow users to select a local directory containing the codebase they wish to analyze.

Parsing Initiation: A button or action in the UI to trigger the parsing and ingestion process for the selected directory.

Multi-Language Parsing Engine (Rust Backend):

Implement logic to traverse the selected directory recursively.

Identify source files based on extensions (e.g., .rs, .py, .js, .ts, .jsx, .tsx).

Ignore common irrelevant directories (node_modules, .git, target, dist, build, etc.).

Load the appropriate tree-sitter language grammar based on the file extension.

Parse each valid source file into an Abstract Syntax Tree (AST).

Traverse the AST to identify relevant code entities.

Neo4j Graph Ingestion (Rust Backend):

Connect to the specified Neo4j database using credentials loaded securely (e.g., from environment variables).

Map identified code entities and their relationships to a predefined Neo4j graph schema (see Section 5).

Use idempotent MERGE operations in Cypher to create/update nodes and relationships, preventing duplicates on re-runs.

Perform database operations within transactions (ideally one transaction per file for atomicity).

Status Reporting & Error Handling:

The Rust backend should emit events (using app_handle.emit_all) to the frontend indicating:

Start of processing.

File currently being processed (optional, for progress).

Completion of processing (success or failure).

Specific errors encountered during parsing or database ingestion (e.g., file not found, parse error, DB connection error, unsupported language).

The frontend should display these status messages and errors clearly to the user.

Configuration: Neo4j connection details (URI, Username, Password) should be configurable, primarily via environment variables loaded by the Rust backend.

4. Backend Implementation Details (Rust - src-tauri):

Tauri Command: Define an async Tauri command (e.g., parse_and_ingest_codebase) that accepts the root directory path string as an argument.

Neo4j Connection: Implement robust connection logic using neo4rs, including error handling and potentially managing a shared Graph client instance (e.g., via Tauri state management or once_cell).

Tree-sitter Setup: Instantiate tree_sitter::Parser and dynamically load tree_sitter::Language based on file extensions using the appropriate grammar crates (e.g., tree_sitter_rust::language()).

AST Traversal: Use TreeCursor for efficient AST traversal.

Mapping Logic: Implement functions or logic to translate tree-sitter Node types and properties into Cypher MERGE statements corresponding to the defined Neo4j schema. Handle unique identification of nodes (e.g., file path, function name + file path).

Concurrency: Leverage Rust's async/await for non-blocking file I/O and database operations. Consider potential parallelization strategies for file processing if performance becomes an issue (though manage database write contention).

Error Handling: Use Result extensively and define custom error types (e.g., using thiserror) that can be serialized back to the frontend via Tauri command errors or events.

5. Neo4j Graph Schema:

Node Labels:

:File (props: path [unique], name, language)

:Directory (props: path [unique], name)

:Class, :Struct, :Enum, :Interface, :Trait (props: name, filePath, startLine, endLine, qualifiedName?)

:Function, :Method (props: name, filePath, startLine, endLine, signature?, isAsync?)

:Variable (props: name, filePath, declarationLine, type?, scope?)

:Parameter (props: name, type?, index)

:CallSite (props: filePath, line, calledName)

(Optional) :Import, :ExternalLibrary

Relationship Types:

:CONTAINS (Directory -> Directory|File)

:DECLARES (File|Class|Function|Method -> Class|Function|Method|Variable)

:HAS_PARAMETER (Function|Method -> Parameter)

:CALLS (Function|Method -> Function|Method) OR :HAS_CALL_SITE (Function|Method -> CallSite) + :CALLS (CallSite -> Function|Method)

:REFERENCES (Function|Method|Class -> Variable)

:IMPORTS (File -> File|ExternalLibrary)

:EXTENDS (Class -> Class)

:IMPLEMENTS (Class -> Interface|Trait)

:HAS_TYPE (Variable|Parameter|Function|Method -> Class|Interface|Primitive)

Indexes/Constraints: Create unique constraints on :File(path) and :Directory(path). Create indexes on frequently queried properties like :Function(name), :Class(name), :File(language).

6. Frontend Implementation Details (Webview - TS/JS):

UI Elements:

Button to trigger directory selection (dialog.open).

Display area for the selected directory path.

Button to start the parse_and_ingest_codebase Tauri command.

Status area (text box, log list) to display messages received via Tauri events (parse_progress, parse_error, parse_complete).

Logic:

Use invoke to call the Rust backend command.

Use listen to subscribe to events from the backend and update the UI accordingly.

Disable the "Start Parsing" button while processing is in progress.

Handle potential errors returned by the invoke call.

7. Assumptions & Constraints:

The user must have a running Neo4j instance accessible from where the Tauri application is run.

Correct Neo4j credentials must be provided (initially via .env for the backend). A strategy for managing credentials in a packaged application needs consideration (e.g., user input stored securely, OS keychain).

Parsing performance will depend on the size and complexity of the codebase and the machine running the application.

The accuracy of the :CALLS relationship depends on the complexity of the resolution logic implemented in the Rust backend. Basic name matching will be the starting point.

Initial language support will focus on a few key languages (e.g., Rust, Python, JS/TS), expandable later.

8. Future Enhancements (Optional):

UI to configure Neo4j connection details directly.

UI to browse/query the generated graph data within the Tauri app.

Integration with graph visualization libraries (e.g., rendering parts of the graph in the frontend).

Support for more programming languages by adding respective tree-sitter grammar crates.

More sophisticated analysis features (e.g., cycle detection, unused code analysis based on graph queries).

Incremental parsing (only processing changed files).

