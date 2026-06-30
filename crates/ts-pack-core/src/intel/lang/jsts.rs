//! JavaScript / TypeScript / TSX: generic structure; `import_statement` imports
//! and `export_statement` exports.

use super::LanguageIntel;

pub(crate) struct JsTs;

impl LanguageIntel for JsTs {
    fn is_import(&self, node_kind: &str) -> bool {
        node_kind == "import_statement"
    }

    fn is_export(&self, node_kind: &str) -> bool {
        node_kind == "export_statement"
    }
}
