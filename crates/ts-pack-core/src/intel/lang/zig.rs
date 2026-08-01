//! Zig: a function is `Decl > [FnProto, Block]` — `FnProto` carries the name
//! (IDENTIFIER) and the `Block` is its sibling body, same shape as Dart. The
//! def's editable span covers both.

use super::{LanguageIntel, compact_signature, generic_structure_kind};
use crate::intel::intelligence::node_text;
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
    fn signature_of(&self, node: &Node, source: &str) -> Option<String> {
        if node.kind() != "FnProto" {
            return None;
        }
        let mut cursor = node.walk();
        let parameters = node
            .named_children(&mut cursor)
            .find(|child| child.kind() == "ParamDeclList")?;
        let mut signature = compact_signature(node_text(&parameters, source));
        let mut cursor = node.walk();
        if let Some(return_type) = node
            .named_children(&mut cursor)
            .find(|child| child.start_byte() > parameters.end_byte())
        {
            signature.push_str(" -> ");
            signature.push_str(&compact_signature(node_text(&return_type, source)));
        }
        Some(signature)
    }
}
