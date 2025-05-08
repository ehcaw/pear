// ts_queries.rs
pub const ENTITY_AND_DEP_QUERY: &str = r#"
; ── declarations ──────────────────────────────────────────
(
  (class_declaration name: (type_identifier) @class.name) @class.node
)
(
  (function_declaration name: (identifier) @function.name) @function.node
)
(
  (interface_declaration name: (type_identifier) @interface.name) @interface.node
)

; ── imports / re-exports (dependencies) ───────────────────
(
  (import_clause
      (_)* @import.item)
  source: (string) @import.source
) @import.statement

(
  (export_statement
      (export_clause) @export.item
      source: (string) @export.source)?
) @export.statement
"#;
