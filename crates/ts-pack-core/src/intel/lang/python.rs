//! Python: structure is generic node-kind based; docstrings and imports are
//! Python-specific.

use super::LanguageIntel;
use crate::intel::intelligence::{node_text, span_from_node};
use crate::intel::types::*;
use tree_sitter::Node;

pub(crate) struct Python;

impl LanguageIntel for Python {
    fn is_import(&self, node_kind: &str) -> bool {
        node_kind == "import_statement" || node_kind == "import_from_statement"
    }

    fn docstring_at(&self, node: &Node, source: &str, out: &mut Vec<DocstringInfo>) {
        // A Python docstring is the first statement of a module / function /
        // class body, written as a string literal. Two tree shapes occur across
        // tree-sitter-python versions: the string wrapped in an
        // `expression_statement`, or — in current grammars — a bare `string`
        // node directly under the `block` / `module`. Handle both, and only when
        // it is the FIRST statement (else it is just a string, not a doc).
        let string_child = if node.kind() == "expression_statement" {
            node.child(0)
                .filter(|c| c.kind() == "string" || c.kind() == "concatenated_string")
        } else if node.kind() == "string" || node.kind() == "concatenated_string" {
            Some(*node)
        } else {
            None
        };
        if let Some(child) = string_child
            && let Some(parent) = node.parent()
        {
            let parent_kind = parent.kind();
            let is_first = parent.named_child(0).map(|f| f.id()) == Some(node.id());
            if (parent_kind == "block" || parent_kind == "module") && is_first {
                out.push(DocstringInfo {
                    text: node_text(&child, source).to_string(),
                    format: DocstringFormat::PythonTripleQuote,
                    span: span_from_node(&child),
                    associated_item: parent.parent().and_then(|gp| {
                        gp.child_by_field_name("name")
                            .map(|n| node_text(&n, source).to_string())
                    }),
                    parsed_sections: Vec::new(),
                });
            }
        }
    }
}
