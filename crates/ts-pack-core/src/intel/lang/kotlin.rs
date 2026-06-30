//! Kotlin: generic structure (`package_header`, `class_declaration`,
//! `function_declaration`, …); `import_declaration` imports.

use super::LanguageIntel;

pub(crate) struct Kotlin;

impl LanguageIntel for Kotlin {
    fn is_import(&self, node_kind: &str) -> bool {
        node_kind == "import_declaration"
    }
}
