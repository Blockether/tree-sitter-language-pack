//! Ruby: adds `method`/`singleton_method` and bare `module` to the generic
//! structure kinds.

use super::{LanguageIntel, generic_structure_kind};
use crate::intel::types::StructureKind;

pub(crate) struct Ruby;

impl LanguageIntel for Ruby {
    fn structure_kind(&self, node_kind: &str) -> Option<StructureKind> {
        match node_kind {
            "method" | "singleton_method" => Some(StructureKind::Method),
            "module" => Some(StructureKind::Module),
            other => generic_structure_kind(other),
        }
    }
}
