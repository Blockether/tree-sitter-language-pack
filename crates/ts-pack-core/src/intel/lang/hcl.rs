//! HCL / Terraform: a file is a list of `block`s (`resource "aws_s3_bucket"
//! "assets" { … }`) and top-level `attribute`s (`.tfvars`). The block TYPE is
//! the kind and the LABELS are the name, so a resource reads as
//! `resource aws_s3_bucket.assets` — exactly how Terraform addresses it.

use super::LanguageIntel;
use crate::intel::intelligence::node_text;
use crate::intel::types::StructureKind;
use tree_sitter::Node;

pub(crate) struct Hcl;

fn block_type(node: &Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|c| c.kind() == "identifier")
        .map(|c| node_text(&c, source).to_string())
}

impl LanguageIntel for Hcl {
    fn structure_kind_of(&self, node: &Node, source: &str) -> Option<StructureKind> {
        match node.kind() {
            "block" => Some(match block_type(node, source).as_deref() {
                Some("variable") => StructureKind::Variable,
                Some("module") => StructureKind::Module,
                Some(other) => StructureKind::Other(other.to_string()),
                None => StructureKind::Other("block".to_string()),
            }),
            "attribute" => Some(StructureKind::Constant),
            _ => None,
        }
    }

    fn name_of(&self, node: &Node, source: &str) -> Option<String> {
        match node.kind() {
            "block" => {
                let mut cursor = node.walk();
                let labels: Vec<String> = node
                    .named_children(&mut cursor)
                    .filter(|c| c.kind() == "string_lit" || c.kind() == "identifier")
                    .skip(1) // the first identifier is the block TYPE, not a label
                    .map(|c| node_text(&c, source).trim_matches('"').to_string())
                    .collect();
                if labels.is_empty() {
                    block_type(node, source)
                } else {
                    Some(labels.join("."))
                }
            }
            "attribute" => {
                let mut cursor = node.walk();
                node.named_children(&mut cursor)
                    .find(|c| c.kind() == "identifier")
                    .map(|c| node_text(&c, source).to_string())
            }
            _ => super::resolve_structure_name(node, source),
        }
    }
}
