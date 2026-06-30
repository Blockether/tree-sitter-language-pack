//! Java: generic structure (`package_declaration`, `class_declaration`, …);
//! `import_declaration` imports.

use super::LanguageIntel;

pub(crate) struct Java;

impl LanguageIntel for Java {
    fn is_import(&self, node_kind: &str) -> bool {
        node_kind == "import_declaration"
    }
}
