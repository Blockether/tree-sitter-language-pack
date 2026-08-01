//! Dart: a function is a `*_signature` node followed by a sibling
//! `function_body`/`block` (TS uses these node kinds for ambient/overload sigs,
//! so they are scoped to Dart here). The def's editable span covers both.

use super::{LanguageIntel, compact_signature, generic_structure_kind};
use crate::intel::intelligence::node_text;
use crate::intel::types::StructureKind;
use tree_sitter::Node;

pub(crate) struct Dart;

impl LanguageIntel for Dart {
    fn structure_kind(&self, node_kind: &str) -> Option<StructureKind> {
        match node_kind {
            "function_signature" => Some(StructureKind::Function),
            "method_signature" => Some(StructureKind::Method),
            other => generic_structure_kind(other),
        }
    }

    fn sibling_body<'t>(&self, node: &Node<'t>) -> Option<Node<'t>> {
        match node.kind() {
            "function_signature" | "method_signature" => node
                .next_named_sibling()
                .filter(|s| matches!(s.kind(), "function_body" | "block")),
            _ => None,
        }
    }
    fn signature_of(&self, node: &Node, source: &str) -> Option<String> {
        if !matches!(node.kind(), "function_signature" | "method_signature") {
            return None;
        }
        let mut cursor = node.walk();
        let parameters = node
            .named_children(&mut cursor)
            .find(|child| child.kind() == "formal_parameter_list")?;
        let mut signature = compact_signature(node_text(&parameters, source));
        if let Some(return_type) = node
            .named_child(0)
            .filter(|child| child.end_byte() < parameters.start_byte())
        {
            signature.push_str(" -> ");
            signature.push_str(&compact_signature(node_text(&return_type, source)));
        }
        Some(signature)
    }
}
