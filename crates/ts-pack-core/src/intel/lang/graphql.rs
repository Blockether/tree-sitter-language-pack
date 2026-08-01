//! GraphQL: SDL type-system definitions plus executable definitions. The kind is
//! the GraphQL keyword itself (`type`, `input`, `union`, `fragment`, `query`, …)
//! carried as [`StructureKind::Other`], because GraphQL's categories do not map
//! onto host-language ones without losing information.

use super::{LanguageIntel, compact_signature};
use crate::intel::intelligence::node_text;
use crate::intel::types::StructureKind;
use tree_sitter::Node;

pub(crate) struct GraphQl;

fn keyword(node_kind: &str) -> Option<&'static str> {
    Some(match node_kind {
        "object_type_definition" | "object_type_extension" => "type",
        "interface_type_definition" | "interface_type_extension" => "interface",
        "union_type_definition" | "union_type_extension" => "union",
        "enum_type_definition" | "enum_type_extension" => "enum",
        "input_object_type_definition" | "input_object_type_extension" => "input",
        "scalar_type_definition" | "scalar_type_extension" => "scalar",
        "schema_definition" | "schema_extension" => "schema",
        "directive_definition" => "directive",
        "fragment_definition" => "fragment",
        "operation_definition" => "operation",
        // Members, reached only through `body_of` below.
        "field_definition" | "input_value_definition" => "field",
        "enum_value_definition" => "value",
        _ => return None,
    })
}

/// The first `name` node at any depth of `node`'s own name-carrying children —
/// GraphQL wraps names differently per definition (`fragment_name > name`).
fn first_name(node: &Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "name" => return Some(node_text(&child, source).to_string()),
            "fragment_name" | "enum_value" => return first_name(&child, source),
            _ => {}
        }
    }
    None
}

impl LanguageIntel for GraphQl {
    fn structure_kind_of(&self, node: &Node, source: &str) -> Option<StructureKind> {
        let kw = keyword(node.kind())?;
        // Fields and enum values are real members, not GraphQL-only categories.
        match kw {
            "field" => return Some(StructureKind::Field),
            "value" => return Some(StructureKind::Constant),
            _ => {}
        }
        // `query GetWidget { … }` / `mutation … ` — report the operation type.
        if kw == "operation" {
            let mut cursor = node.walk();
            let op = node
                .named_children(&mut cursor)
                .find(|c| c.kind() == "operation_type")
                .map(|c| node_text(&c, source).to_string())
                .unwrap_or_else(|| "query".to_string());
            return Some(StructureKind::Other(op));
        }
        Some(StructureKind::Other(kw.to_string()))
    }

    fn name_of(&self, node: &Node, source: &str) -> Option<String> {
        first_name(node, source)
    }

    /// The members of a type live in a `*fields_definition` / `enum_values_definition`
    /// list, and an operation's selections are not definitions at all.
    fn body_of<'t>(&self, node: &Node<'t>) -> Option<Node<'t>> {
        let mut cursor = node.walk();
        node.named_children(&mut cursor).find(|c| {
            matches!(
                c.kind(),
                "fields_definition" | "input_fields_definition" | "enum_values_definition"
            )
        })
    }
    fn signature_of(&self, node: &Node, source: &str) -> Option<String> {
        if node.kind() != "field_definition" {
            return None;
        }
        let mut cursor = node.walk();
        let parameters = node
            .named_children(&mut cursor)
            .find(|child| child.kind() == "arguments_definition")?;
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
