//! Svelte and Vue single-file components. Both grammars parse the FILE, not the
//! script: `<script>` bodies arrive as one opaque `raw_text` node. So the
//! structure is the component's sections (`template` / `script` / `style`), and
//! each script's real definitions are recovered by re-parsing its body with the
//! TypeScript grammar and shifting the spans back (see [`super::embedded`]).

use super::LanguageIntel;
use super::embedded::{embedded_imports, embedded_structure};
use crate::intel::intelligence::{node_text, span_from_node};
use crate::intel::types::{ImportInfo, StructureItem, StructureKind};
use tree_sitter::Node;

pub(crate) struct Web;

/// TypeScript parses plain JavaScript too, so one grammar covers `lang="ts"` and
/// untyped scripts alike; `javascript` is the fallback for reduced builds.
const SCRIPT_LANGUAGES: [&str; 2] = ["typescript", "javascript"];

fn attribute_summary(node: &Node, source: &str) -> String {
    let Some(start) = node.child_by_field_name("start_tag").or_else(|| node.named_child(0)) else {
        return String::new();
    };
    let mut cursor = start.walk();
    let attrs: Vec<String> = start
        .named_children(&mut cursor)
        .filter(|c| c.kind() == "attribute" || c.kind() == "directive_attribute")
        .map(|c| node_text(&c, source).to_string())
        .collect();
    attrs.join(" ")
}

fn raw_text<'t>(node: &Node<'t>) -> Option<Node<'t>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).find(|c| c.kind() == "raw_text")
}

impl LanguageIntel for Web {
    fn structure(&self, root: &Node, source: &str) -> Vec<StructureItem> {
        let mut items = Vec::with_capacity(4);
        collect(root, source, &mut items);
        items
    }

    /// A component's imports are its `<script>`s' imports: the outer grammar
    /// sees only opaque text, so each script body is re-parsed and its spans
    /// shifted back into the component file.
    fn imports(&self, root: &Node, source: &str) -> Vec<ImportInfo> {
        let mut bodies = Vec::with_capacity(2);
        script_bodies(root, &mut bodies);
        let mut out = Vec::with_capacity(8);
        for body in bodies {
            let span = span_from_node(&body);
            let offset = (span.start_byte, span.start_line, span.start_column);
            if let Some(found) = SCRIPT_LANGUAGES
                .iter()
                .map(|lang| embedded_imports(node_text(&body, source), lang, offset))
                .find(|imports| !imports.is_empty())
            {
                out.extend(found);
            }
        }
        out
    }
}

/// Every `<script>` body in the component, in document order.
fn script_bodies<'t>(node: &Node<'t>, out: &mut Vec<Node<'t>>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "script_element" => out.extend(raw_text(&child)),
            "style_element" => {}
            _ => script_bodies(&child, out),
        }
    }
}

fn collect(node: &Node, source: &str, items: &mut Vec<StructureItem>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        let tag = match child.kind() {
            "script_element" => "script",
            "style_element" => "style",
            "template_element" => "template",
            _ => {
                collect(&child, source, items);
                continue;
            }
        };
        let body = raw_text(&child);
        let children = match (tag, &body) {
            ("script", Some(text)) => {
                let span = span_from_node(text);
                let offset = (span.start_byte, span.start_line, span.start_column);
                SCRIPT_LANGUAGES
                    .iter()
                    .map(|lang| embedded_structure(node_text(text, source), lang, offset))
                    .find(|items| !items.is_empty())
                    .unwrap_or_default()
            }
            _ => Vec::new(),
        };
        let attrs = attribute_summary(&child, source);
        let name = if attrs.is_empty() {
            tag.to_string()
        } else {
            format!("{tag} {attrs}")
        };
        items.push(StructureItem {
            kind: StructureKind::Other(tag.to_string()),
            name: Some(name),
            visibility: None,
            span: span_from_node(&child),
            children,
            decorators: Vec::new(),
            doc_comment: None,
            signature: None,
            body_span: body.as_ref().map(span_from_node),
        });
    }
}
