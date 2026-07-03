//! Elixir — adapts upstream's `intel::elixir` call-arm extractors
//! ([`collect_structure_call`](crate::intel::elixir) / `collect_import_call`)
//! to the [`LanguageIntel`] registry. Elixir has no dedicated definition or
//! import node kinds (modules, defs and directives are all `call` nodes), so
//! structure/imports run the elixir-only compat walkers in `intelligence`;
//! docstrings and exports fall through to the generic defaults.

use super::LanguageIntel;
use crate::intel::intelligence::{collect_imports, collect_structure};
use crate::intel::types::*;
use tree_sitter::Node;

pub(crate) struct Elixir;

impl LanguageIntel for Elixir {
    fn structure(&self, root: &Node, source: &str) -> Vec<StructureItem> {
        let mut items = Vec::with_capacity(32);
        collect_structure(root, source, "elixir", &mut items);
        items
    }

    fn imports(&self, root: &Node, source: &str) -> Vec<ImportInfo> {
        let mut imports = Vec::with_capacity(16);
        collect_imports(root, source, "elixir", &mut imports);
        imports
    }
}
