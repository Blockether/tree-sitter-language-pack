//! Kotlin: generic structure (`package_header`, `class_declaration`,
//! `function_declaration`, …); `import_declaration` imports.

use super::{LanguageIntel, compact_signature};
use crate::intel::intelligence::node_text;
use tree_sitter::Node;

pub(crate) struct Kotlin;

impl LanguageIntel for Kotlin {
    fn is_import(&self, node_kind: &str) -> bool {
        node_kind == "import_declaration"
    }
    /// Parameter list plus the declared return type. The return type is the
    /// child that follows the parameters — but a Kotlin body (`= expr` or a
    /// block) is a following child too, so bodies and type constraints are
    /// skipped; `fun f(a: Int) { … }` has no return type to show.
    fn signature_of(&self, node: &Node, source: &str) -> Option<String> {
        if node.kind() != "function_declaration" {
            return None;
        }
        let mut cursor = node.walk();
        let parameters = node
            .named_children(&mut cursor)
            .find(|child| child.kind() == "function_value_parameters")?;
        let mut signature = compact_signature(node_text(&parameters, source));
        let mut cursor = node.walk();
        if let Some(return_type) = node.named_children(&mut cursor).find(|child| {
            child.start_byte() > parameters.end_byte() && !matches!(child.kind(), "function_body" | "type_constraints")
        }) {
            signature.push_str(" -> ");
            signature.push_str(&compact_signature(node_text(&return_type, source)));
        }
        Some(signature)
    }
}
