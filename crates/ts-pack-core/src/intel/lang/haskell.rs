//! Haskell: declarations are `function` / `bind` / `data_type` / `newtype` /
//! `class` / `instance` nodes under a `declarations` list, each preceded by an
//! optional `signature` (the type signature) which belongs to the definition and
//! is absorbed into its span via [`LanguageIntel::leading_sibling`].

use super::{LanguageIntel, compact_signature};
use crate::intel::intelligence::node_text;
use crate::intel::types::StructureKind;
use tree_sitter::Node;

pub(crate) struct Haskell;

fn field_text(node: &Node, field: &str, source: &str) -> Option<String> {
    node.child_by_field_name(field)
        .map(|n| node_text(&n, source).to_string())
        .filter(|t| !t.is_empty())
}

impl LanguageIntel for Haskell {
    fn structure_kind(&self, node_kind: &str) -> Option<StructureKind> {
        match node_kind {
            "function" => Some(StructureKind::Function),
            // A top-level `name = expr` with no parameters: a value binding.
            "bind" => Some(StructureKind::Constant),
            // `type_synomym` is the upstream grammar's own (misspelt) node kind;
            // `type_synonym` is accepted too so a future fix keeps working.
            "data_type" | "newtype" | "type_synomym" | "type_synonym" | "type_family" | "data_family" => {
                Some(StructureKind::Type)
            }
            // A Haskell type class is a trait / an instance is its impl.
            "class" => Some(StructureKind::Trait),
            "instance" => Some(StructureKind::Impl),
            "header" => Some(StructureKind::Module),
            _ => None,
        }
    }

    /// `function` is ALSO the node kind of a function TYPE (`a -> b`), so a
    /// declaration is only one when it sits directly in a declaration list.
    fn structure_kind_of(&self, node: &Node, _source: &str) -> Option<StructureKind> {
        let kind = self.structure_kind(node.kind())?;
        let in_declaration_list = node.parent().is_none_or(|p| {
            matches!(
                p.kind(),
                "haskell" | "declarations" | "class_declarations" | "instance_declarations" | "local_binds" | "where"
            )
        });
        in_declaration_list.then_some(kind)
    }

    /// A class/instance body is a `*_declarations` list, not a `body` field.
    fn body_of<'t>(&self, node: &Node<'t>) -> Option<Node<'t>> {
        let mut cursor = node.walk();
        node.named_children(&mut cursor)
            .find(|c| matches!(c.kind(), "class_declarations" | "instance_declarations"))
    }

    fn name_of(&self, node: &Node, source: &str) -> Option<String> {
        match node.kind() {
            "header" => field_text(node, "module", source),
            // `instance Renderable Widget` — the class name alone is ambiguous
            // across instances, so the head types are part of the name.
            "instance" => {
                let class = field_text(node, "name", source)?;
                Some(match field_text(node, "patterns", source) {
                    Some(args) => format!("{class} {args}"),
                    None => class,
                })
            }
            _ => super::resolve_structure_name(node, source),
        }
    }

    /// A `signature` sibling directly above a `function`/`bind` of the SAME name
    /// is that definition's type signature: editing the definition means editing
    /// both, so the span starts at the signature.
    fn leading_sibling<'t>(&self, node: &Node<'t>, source: &str) -> Option<Node<'t>> {
        if !matches!(node.kind(), "function" | "bind") {
            return None;
        }
        let name = field_text(node, "name", source)?;
        let prev = node.prev_named_sibling()?;
        if prev.kind() != "signature" {
            return None;
        }
        (field_text(&prev, "name", source).as_deref() == Some(name.as_str())).then_some(prev)
    }

    fn is_import(&self, node_kind: &str) -> bool {
        node_kind == "import"
    }
    fn signature_of(&self, node: &Node, source: &str) -> Option<String> {
        (node.kind() == "function")
            .then(|| self.leading_sibling(node, source))
            .flatten()
            .map(|signature| compact_signature(node_text(&signature, source)))
    }
}
