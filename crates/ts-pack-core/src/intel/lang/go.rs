//! Go: generic structure; `import_declaration`/`import_spec` imports.

use super::LanguageIntel;

pub(crate) struct Go;

impl LanguageIntel for Go {
    fn is_import(&self, node_kind: &str) -> bool {
        node_kind == "import_declaration" || node_kind == "import_spec"
    }
}
