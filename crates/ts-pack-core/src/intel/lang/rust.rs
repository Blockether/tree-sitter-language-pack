//! Rust: generic structure; `use_declaration` imports.

use super::LanguageIntel;

pub(crate) struct Rust;

impl LanguageIntel for Rust {
    fn is_import(&self, node_kind: &str) -> bool {
        node_kind == "use_declaration"
    }
}
