//! Per-language tree-sitter query definitions for intelligence extraction.
//!
//! Each submodule can define language-specific S-expression queries
//! for extracting imports, exports, comments, docstrings, etc.
//! These are currently placeholders — the intelligence module uses
//! generic AST walking. These will be populated with optimized
//! tree-sitter queries as the module matures.

pub mod generic;
pub mod go_lang;
pub mod javascript;
pub mod python;
pub mod rust_lang;
pub mod typescript;
