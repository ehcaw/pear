// src/ts_queries.rs

//! Tree-sitter query that extracts TypeScript/TSX code entities and their
//! dependency edges (imports / exports).
//!
//! Capture names follow the pattern `<kind>.<field>` so the extractor can
//! match on either the full node (e.g. `@class.node`) or just the identifier
//! (`@class.name`).

pub const ENTITY_AND_DEP_QUERY: &str = r#"
; ===== ENTITIES ==========================================================

; ── Class declarations & default export classes --------------------------
(class_declaration
  name: (identifier) @class.name) @class.node

(export_default_statement
  declaration: (class_declaration) @class.node)

; ── Interface declarations ------------------------------------------------
(interface_declaration
  name: (identifier) @interface.name) @interface.node

; ── Free function declarations & default export functions ----------------
(function_declaration
  name: (identifier) @function.name) @function.node

(export_default_statement
  declaration: (function_declaration) @function.node)

; ── Arrow‐function assignments (const Foo = () => { … }) -------------------
(variable_declarator
  name: (identifier) @function.name
  value: (arrow_function)) @function.node

; ── Method definitions ----------------------------------------------------
(method_definition
  name: (property_identifier) @method.name) @method.node

; ===== DEPENDENCIES =====================================================

; ── Static import … from "module" ----------------------------------------
(import_statement
  source: (string) @import.source) @import.statement

; ── Static export … from "module" ----------------------------------------
(export_statement
  source: (string) @export.source) @export.statement

; ── Named exports with clause --------------------------------------------
(export_statement
  (export_clause) @export.item
  source: (string) @export.source) @export.statement

; ── Dynamic import("module") ---------------------------------------------
(call_expression
  function: (identifier) @import.func
  arguments: (arguments (string) @import.source)) @import.dynamic

; ── require("module") calls ----------------------------------------------
(call_expression
  function: (identifier) @import.func
  arguments: (arguments (string) @import.source)) @import.require
"#;
