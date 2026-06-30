//! Dart: a function is a `*_signature` node followed by a sibling
//! `function_body`/`block` (TS uses these node kinds for ambient/overload sigs,
//! so they are scoped to Dart here). The def's editable span covers both.

use super::{LanguageIntel, generic_structure_kind};
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
}
