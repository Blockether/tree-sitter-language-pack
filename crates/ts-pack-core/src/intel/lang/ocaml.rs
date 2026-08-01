//! OCaml (and `.mli` interfaces): every definition is a `*_definition` wrapper
//! around a `*_binding` that carries the name, so names come from the binding's
//! `name`/`pattern` field. A `value_definition` is a Function when its binding
//! takes `parameter`s and a Constant otherwise.

use super::{LanguageIntel, compact_signature};
use crate::intel::intelligence::node_text;
use crate::intel::types::StructureKind;
use tree_sitter::Node;

pub(crate) struct Ocaml;

/// The `*_binding` child holding the name, or the node itself.
fn binding<'t>(node: &Node<'t>) -> Node<'t> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|c| c.kind().ends_with("_binding"))
        .unwrap_or(*node)
}

fn has_parameter(node: &Node) -> bool {
    let b = binding(node);
    let mut cursor = b.walk();
    b.named_children(&mut cursor).any(|c| c.kind() == "parameter")
}

fn name_text(node: &Node, source: &str) -> Option<String> {
    let b = binding(node);
    for field in ["name", "pattern"] {
        if let Some(n) = b.child_by_field_name(field) {
            let t = node_text(&n, source);
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    let mut cursor = b.walk();
    b.named_children(&mut cursor)
        .find(|c| {
            matches!(
                c.kind(),
                "value_name" | "type_constructor" | "module_name" | "class_name" | "method_name" | "constructor_name"
            )
        })
        .map(|c| node_text(&c, source).to_string())
        .filter(|t| !t.is_empty())
}

impl LanguageIntel for Ocaml {
    fn structure_kind_of(&self, node: &Node, _source: &str) -> Option<StructureKind> {
        match node.kind() {
            "value_definition" => Some(if has_parameter(node) {
                StructureKind::Function
            } else {
                StructureKind::Constant
            }),
            // `.mli` signatures and `external` FFI declarations.
            "value_specification" | "external" => Some(StructureKind::Function),
            "type_definition" => Some(StructureKind::Type),
            "module_definition" | "module_type_definition" => Some(StructureKind::Module),
            "class_definition" | "class_type_definition" => Some(StructureKind::Class),
            "method_definition" | "method_specification" => Some(StructureKind::Method),
            "exception_definition" => Some(StructureKind::Other("exception".to_string())),
            _ => None,
        }
    }

    fn name_of(&self, node: &Node, source: &str) -> Option<String> {
        if node.kind() == "exception_definition" {
            let mut cursor = node.walk();
            let ctor = node
                .named_children(&mut cursor)
                .find(|c| c.kind() == "constructor_declaration")?;
            return name_text(&ctor, source).or_else(|| ctor.named_child(0).map(|n| node_text(&n, source).to_string()));
        }
        name_text(node, source)
    }

    fn is_import(&self, node_kind: &str) -> bool {
        node_kind == "open_module" || node_kind == "include_module"
    }
    fn signature_of(&self, node: &Node, source: &str) -> Option<String> {
        if !matches!(node.kind(), "value_definition" | "method_definition") {
            return None;
        }
        let binding = binding(node);
        let mut cursor = binding.walk();
        let parameters = binding
            .named_children(&mut cursor)
            .filter(|child| child.kind() == "parameter")
            .map(|parameter| compact_signature(node_text(&parameter, source)))
            .collect::<Vec<_>>();
        (!parameters.is_empty()).then(|| parameters.join(" "))
    }
}
