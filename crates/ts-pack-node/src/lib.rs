use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Returns an array of all available language names.
#[napi(js_name = "availableLanguages")]
pub fn available_languages() -> Vec<String> {
    ts_pack_core::available_languages()
}

/// Checks whether a language with the given name is available.
#[napi(js_name = "hasLanguage")]
pub fn has_language(name: String) -> bool {
    ts_pack_core::has_language(&name)
}

/// Returns the number of available languages.
#[napi(js_name = "languageCount")]
pub fn language_count() -> u32 {
    ts_pack_core::language_count() as u32
}

/// Returns the raw TSLanguage pointer for interop with node-tree-sitter.
///
/// Throws an error if the language is not found.
#[napi(js_name = "getLanguagePtr")]
pub fn get_language_ptr(name: String) -> napi::Result<i64> {
    let language = ts_pack_core::get_language(&name).map_err(|e| napi::Error::from_reason(format!("{e}")))?;
    let ptr = language.into_raw() as i64;
    Ok(ptr)
}

// ---------------------------------------------------------------------------
// Parsing functions
// ---------------------------------------------------------------------------

/// Parse a source string using the named language and return an opaque tree handle.
///
/// Throws an error if the language is not found or parsing fails.
#[napi(js_name = "parseString")]
pub fn parse_string(language: String, source: String) -> napi::Result<External<tree_sitter::Tree>> {
    let lang = ts_pack_core::get_language(&language).map_err(|e| napi::Error::from_reason(format!("{e}")))?;
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang)
        .map_err(|e| napi::Error::from_reason(format!("failed to set language: {e}")))?;
    let tree = parser
        .parse(source.as_bytes(), None)
        .ok_or_else(|| napi::Error::from_reason("parsing returned no tree"))?;
    Ok(External::new(tree))
}

/// Get the type name of the root node.
#[napi(js_name = "treeRootNodeType")]
pub fn tree_root_node_type(tree: &External<tree_sitter::Tree>) -> String {
    tree.root_node().kind().to_string()
}

/// Get the number of named children of the root node.
#[napi(js_name = "treeRootChildCount")]
pub fn tree_root_child_count(tree: &External<tree_sitter::Tree>) -> u32 {
    tree.root_node().named_child_count() as u32
}

/// Check whether any node in the tree has the given type name.
#[napi(js_name = "treeContainsNodeType")]
pub fn tree_contains_node_type(tree: &External<tree_sitter::Tree>, node_type: String) -> bool {
    let mut cursor = tree.walk();
    traverse_looking_for(&mut cursor, |node| node.kind() == node_type)
}

/// Check whether the tree contains any ERROR or MISSING nodes.
#[napi(js_name = "treeHasErrorNodes")]
pub fn tree_has_error_nodes(tree: &External<tree_sitter::Tree>) -> bool {
    let mut cursor = tree.walk();
    traverse_looking_for(&mut cursor, |node| node.is_error() || node.is_missing())
}

fn traverse_looking_for(cursor: &mut tree_sitter::TreeCursor, predicate: impl Fn(tree_sitter::Node) -> bool) -> bool {
    loop {
        if predicate(cursor.node()) {
            return true;
        }
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return false;
            }
        }
    }
}
