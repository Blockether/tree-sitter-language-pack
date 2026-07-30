//! Rust: generic structure plus the item kinds the shared table does not carry
//! (consts, statics, type aliases, unions, macros, trait method signatures),
//! `pub`/`pub(crate)` visibility, and `use_declaration` imports.

use super::{LanguageIntel, generic_structure_kind};
use crate::intel::intelligence::node_text;
use crate::intel::types::StructureKind;
use tree_sitter::Node;

pub(crate) struct Rust;

impl LanguageIntel for Rust {
    fn structure_kind(&self, node_kind: &str) -> Option<StructureKind> {
        match node_kind {
            "const_item" => Some(StructureKind::Constant),
            "static_item" => Some(StructureKind::Variable),
            "type_item" | "associated_type" => Some(StructureKind::Type),
            "union_item" => Some(StructureKind::Struct),
            "macro_definition" => Some(StructureKind::Macro),
            // A trait's method signature (`fn tag(&self) -> &str;`).
            "function_signature_item" => Some(StructureKind::Function),
            other => generic_structure_kind(other),
        }
    }

    /// `impl Render for Widget` — the trait alone repeats across impls, so the
    /// implementing TYPE is part of the name (the generic chain would return
    /// only the `trait` field).
    fn name_of(&self, node: &Node, source: &str) -> Option<String> {
        if node.kind() == "impl_item" {
            let ty = node
                .child_by_field_name("type")
                .map(|n| node_text(&n, source).to_string());
            let tr = node
                .child_by_field_name("trait")
                .map(|n| node_text(&n, source).to_string());
            return match (tr, ty) {
                (Some(tr), Some(ty)) => Some(format!("{tr} for {ty}")),
                (None, Some(ty)) => Some(ty),
                (Some(tr), None) => Some(tr),
                (None, None) => None,
            };
        }
        super::resolve_structure_name(node, source)
    }

    fn visibility_of(&self, node: &Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        node.named_children(&mut cursor)
            .find(|c| c.kind() == "visibility_modifier")
            .map(|c| node_text(&c, source).to_string())
    }

    fn is_import(&self, node_kind: &str) -> bool {
        node_kind == "use_declaration"
    }
}
