//! Per-language intelligence extraction.
//!
//! Each language is a unit struct implementing [`LanguageIntel`]. Language
//! behaviour is expressed as small hooks — `structure_kind`, `sibling_body`,
//! `docstring_at`, `is_import`, `is_export` — layered over shared recursive
//! walkers, so a language overrides ONLY what differs from the generic
//! node-kind logic, which lives in exactly one place. [`for_language`] is the
//! single dispatch point, replacing the `match language { … }` arms that were
//! previously scattered across `intelligence.rs`.
//!
//! Adding a language is now local: write one small module and add one line to
//! [`for_language`]. A language with no special rules needs no module at all —
//! it falls through to [`Generic`].

use crate::intel::intelligence::{node_text, span_from_node};
use crate::intel::types::*;
use tree_sitter::Node;

mod clojure;
mod dart;
mod go;
mod java;
mod jsts;
mod kotlin;
mod python;
mod ruby;
mod rust;
mod zig;

/// The per-language extraction strategy. Defaults implement the generic,
/// node-kind-uniform behaviour shared by most curly-brace languages; a language
/// overrides only the hooks where it diverges.
pub(crate) trait LanguageIntel {
    // ---- structure ----

    /// Map a tree-sitter node kind to a [`StructureKind`], or `None` when the
    /// node is not a top-level definition. Default covers the kinds shared
    /// across languages; override to add language-specific node kinds.
    fn structure_kind(&self, node_kind: &str) -> Option<StructureKind> {
        generic_structure_kind(node_kind)
    }

    /// For languages where a definition spans a *signature* node plus a
    /// following sibling *body* (Dart `function_signature` + `function_body`,
    /// Zig `FnProto` + `Block`), return that sibling so the span and body span
    /// can be extended to cover it. Default: none (the node spans itself).
    fn sibling_body<'t>(&self, _node: &Node<'t>) -> Option<Node<'t>> {
        None
    }

    /// Full structure extraction. Default: a generic recursive walk driven by
    /// [`Self::structure_kind`] / [`Self::sibling_body`]. Languages with
    /// non-node-kind structure (e.g. Clojure's s-expressions) override this.
    fn structure(&self, root: &Node, source: &str) -> Vec<StructureItem> {
        let mut items = Vec::with_capacity(32);
        walk_structure(self, root, source, &mut items);
        items
    }

    // ---- docstrings ----

    /// Emit any docstring *associated with this single node* into `out`. Default:
    /// nothing (most languages carry docs as comments, handled separately).
    fn docstring_at(&self, _node: &Node, _source: &str, _out: &mut Vec<DocstringInfo>) {}

    /// Full docstring extraction. Default: a recursive walk calling
    /// [`Self::docstring_at`] at every node.
    fn docstrings(&self, root: &Node, source: &str) -> Vec<DocstringInfo> {
        let mut out = Vec::with_capacity(16);
        walk_docstrings(self, root, source, &mut out);
        out
    }

    // ---- imports / exports ----

    /// Whether a node kind is an import statement for this language.
    fn is_import(&self, _node_kind: &str) -> bool {
        false
    }

    /// Whether a node kind is an export statement for this language.
    fn is_export(&self, _node_kind: &str) -> bool {
        false
    }

    fn imports(&self, root: &Node, source: &str) -> Vec<ImportInfo> {
        let mut out = Vec::with_capacity(16);
        walk_imports(self, root, source, &mut out);
        out
    }

    fn exports(&self, root: &Node, source: &str) -> Vec<ExportInfo> {
        let mut out = Vec::with_capacity(16);
        walk_exports(self, root, source, &mut out);
        out
    }
}

/// Fallback strategy: pure generic behaviour, no language-specific rules. Used
/// for every language that needs no overrides.
pub(crate) struct Generic;
impl LanguageIntel for Generic {}

/// The single dispatch point: pick the extraction strategy for `language`.
pub(crate) fn for_language(language: &str) -> Box<dyn LanguageIntel> {
    match language {
        "clojure" => Box::new(clojure::Clojure),
        "python" => Box::new(python::Python),
        "ruby" => Box::new(ruby::Ruby),
        "dart" => Box::new(dart::Dart),
        "zig" => Box::new(zig::Zig),
        "rust" => Box::new(rust::Rust),
        "go" => Box::new(go::Go),
        "java" => Box::new(java::Java),
        "kotlin" => Box::new(kotlin::Kotlin),
        "javascript" | "typescript" | "tsx" => Box::new(jsts::JsTs),
        _ => Box::new(Generic),
    }
}

// ---------------------------------------------------------------------------
// Shared, language-neutral helpers used by the default trait methods.
// ---------------------------------------------------------------------------

/// The node-kind → structure-kind mapping shared across curly-brace languages.
/// Language-specific kinds (Ruby `method`, Dart `function_signature`, Zig
/// `FnProto`, …) are added by the respective `structure_kind` overrides.
pub(crate) fn generic_structure_kind(node_kind: &str) -> Option<StructureKind> {
    match node_kind {
        "function_definition" | "function_declaration" | "function_item" | "arrow_function" => {
            Some(StructureKind::Function)
        }
        "method_definition" | "method_declaration" => Some(StructureKind::Method),
        "class_definition" | "class_declaration" | "class" => Some(StructureKind::Class),
        "struct_item" | "struct_definition" | "struct_declaration" => Some(StructureKind::Struct),
        "interface_declaration" | "interface_definition" => Some(StructureKind::Interface),
        "enum_item" | "enum_definition" | "enum_declaration" => Some(StructureKind::Enum),
        "module_definition" | "mod_item" | "package_header" | "package_declaration" => Some(StructureKind::Module),
        "trait_item" => Some(StructureKind::Trait),
        "impl_item" => Some(StructureKind::Impl),
        _ => None,
    }
}

/// Generic recursive structure walk: at a definition node, record it (extending
/// the span over a `sibling_body` when the language uses one) and descend into
/// its `body` field for nested defs; otherwise recurse into all children.
fn walk_structure<L: LanguageIntel + ?Sized>(rules: &L, node: &Node, source: &str, items: &mut Vec<StructureItem>) {
    if let Some(sk) = rules.structure_kind(node.kind()) {
        let name = resolve_structure_name(node, source);
        // Dart/Zig: the editable def spans a signature/proto node AND the
        // following sibling body. Other languages keep the node's own span.
        let sibling_body = rules.sibling_body(node);
        let span = match &sibling_body {
            Some(body) => {
                let mut s = span_from_node(node);
                let e = span_from_node(body);
                s.end_byte = e.end_byte;
                s.end_line = e.end_line;
                s.end_column = e.end_column;
                s
            }
            None => span_from_node(node),
        };
        let body_span = sibling_body
            .as_ref()
            .map(span_from_node)
            .or_else(|| node.child_by_field_name("body").map(|n| span_from_node(&n)));
        let mut children = Vec::new();
        if let Some(body) = node.child_by_field_name("body") {
            walk_structure(rules, &body, source, &mut children);
        }
        items.push(StructureItem {
            kind: sk,
            name,
            visibility: None,
            span,
            children,
            decorators: Vec::new(),
            doc_comment: None,
            signature: None,
            body_span,
        });
    } else {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk_structure(rules, &child, source, items);
        }
    }
}

fn walk_docstrings<L: LanguageIntel + ?Sized>(rules: &L, node: &Node, source: &str, out: &mut Vec<DocstringInfo>) {
    rules.docstring_at(node, source, out);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_docstrings(rules, &child, source, out);
    }
}

fn walk_imports<L: LanguageIntel + ?Sized>(rules: &L, node: &Node, source: &str, out: &mut Vec<ImportInfo>) {
    if rules.is_import(node.kind()) {
        let text = node_text(node, source);
        out.push(ImportInfo {
            source: text.to_string(),
            items: Vec::new(),
            alias: None,
            is_wildcard: text.contains('*'),
            span: span_from_node(node),
        });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_imports(rules, &child, source, out);
    }
}

fn walk_exports<L: LanguageIntel + ?Sized>(rules: &L, node: &Node, source: &str, out: &mut Vec<ExportInfo>) {
    if rules.is_export(node.kind()) {
        let export_kind = if node.child_by_field_name("default").is_some() {
            ExportKind::Default
        } else if node.child_by_field_name("source").is_some() {
            ExportKind::ReExport
        } else {
            ExportKind::Named
        };
        let text = node_text(node, source);
        out.push(ExportInfo {
            name: text.lines().next().unwrap_or("").to_string(),
            kind: export_kind,
            span: span_from_node(node),
        });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_exports(rules, &child, source, out);
    }
}

/// Resolve the name of a structure node using a fallback chain.
///
/// Tries the `"name"` field first (covers Python, Rust, Java classes), then the
/// C/C++ declarator chain, then the first named child whose kind is one of a
/// priority list of identifier kinds (Kotlin `type_identifier`/
/// `simple_identifier`, Java `scoped_identifier`, …). `None` if nothing matches.
fn resolve_structure_name(node: &Node, source: &str) -> Option<String> {
    // 1. Named field "name" — the common case.
    if let Some(n) = node.child_by_field_name("name") {
        let text = node_text(&n, source);
        if !text.is_empty() {
            return Some(text.to_string());
        }
    }
    // 2. C/C++ — the name is nested in the declarator chain
    //    (function_definition declarator: (function_declarator declarator: (identifier))).
    if let Some(decl) = node.child_by_field_name("declarator")
        && let Some(name) = declarator_identifier(&decl, source)
    {
        return Some(name);
    }
    // 3. Walk named children, trying each identifier kind in priority order.
    //    `simple_identifier` = Kotlin function / property / object names.
    for target_kind in &[
        "type_identifier",
        "simple_identifier",
        "identifier",
        "scoped_identifier",
        "IDENTIFIER",
    ] {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == *target_kind {
                let text = node_text(&child, source);
                if !text.is_empty() {
                    return Some(text.to_string());
                }
            }
        }
    }
    None
}

/// Descend a C/C++ declarator chain (function_declarator / pointer_declarator /
/// reference_declarator …) to the innermost identifier — the function/method name.
fn declarator_identifier(node: &Node, source: &str) -> Option<String> {
    match node.kind() {
        "identifier"
        | "field_identifier"
        | "type_identifier"
        | "qualified_identifier"
        | "destructor_name"
        | "operator_name" => {
            let t = node_text(node, source);
            (!t.is_empty()).then(|| t.to_string())
        }
        _ => {
            if let Some(inner) = node.child_by_field_name("declarator") {
                return declarator_identifier(&inner, source);
            }
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find_map(|child| declarator_identifier(&child, source))
        }
    }
}
