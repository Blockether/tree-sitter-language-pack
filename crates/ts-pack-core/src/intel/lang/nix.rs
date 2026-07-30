//! Nix: the whole file is one expression, so the meaningful "definitions" are
//! `binding` nodes (`attrpath = expr;`) in `let` blocks and attribute sets. A
//! binding whose value is a lambda is a Function, everything else a Constant.
//! Bindings nested INSIDE a binding's value are not re-listed — the outer
//! binding is the editable unit.

use super::LanguageIntel;
use crate::intel::intelligence::node_text;
use crate::intel::types::StructureKind;
use tree_sitter::Node;

pub(crate) struct Nix;

impl LanguageIntel for Nix {
    fn structure_kind_of(&self, node: &Node, _source: &str) -> Option<StructureKind> {
        if node.kind() != "binding" {
            return None;
        }
        let is_fn = node
            .child_by_field_name("expression")
            .is_some_and(|e| e.kind() == "function_expression");
        Some(if is_fn {
            StructureKind::Function
        } else {
            StructureKind::Constant
        })
    }

    fn name_of(&self, node: &Node, source: &str) -> Option<String> {
        node.child_by_field_name("attrpath")
            .map(|n| node_text(&n, source).to_string())
            .filter(|t| !t.is_empty())
    }
}
