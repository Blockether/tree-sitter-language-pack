//! Zig: a function is `Decl > [FnProto, Block]` — `FnProto` carries the name
//! (IDENTIFIER) and the `Block` is its sibling body, same shape as Dart. The
//! def's editable span covers both.

use super::{LanguageIntel, generic_structure_kind};
use crate::intel::types::StructureKind;
use tree_sitter::Node;

pub(crate) struct Zig;

impl LanguageIntel for Zig {
    fn structure_kind(&self, node_kind: &str) -> Option<StructureKind> {
        match node_kind {
            "FnProto" => Some(StructureKind::Function),
            other => generic_structure_kind(other),
        }
    }

    fn sibling_body<'t>(&self, node: &Node<'t>) -> Option<Node<'t>> {
        match node.kind() {
            "FnProto" => node.next_named_sibling().filter(|s| s.kind() == "Block"),
            _ => None,
        }
    }
}
