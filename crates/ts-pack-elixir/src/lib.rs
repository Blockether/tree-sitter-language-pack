use rustler::{Error, NifResult, ResourceArc};
use std::sync::Mutex;

mod atoms {
    rustler::atoms! {
        language_not_found,
        parse_error,
    }
}

/// Wraps a tree-sitter Tree for safe sharing across the NIF boundary.
pub struct TreeResource(Mutex<tree_sitter::Tree>);

#[rustler::resource_impl]
impl rustler::Resource for TreeResource {}

#[rustler::nif]
fn available_languages() -> Vec<String> {
    ts_pack_core::available_languages()
}

#[rustler::nif]
fn has_language(name: String) -> bool {
    ts_pack_core::has_language(&name)
}

#[rustler::nif]
fn language_count() -> usize {
    ts_pack_core::language_count()
}

#[rustler::nif]
fn get_language_ptr(name: String) -> NifResult<u64> {
    let language = ts_pack_core::get_language(&name)
        .map_err(|_| Error::Term(Box::new((atoms::language_not_found(), name.clone()))))?;
    let raw_ptr = language.into_raw();
    Ok(raw_ptr as u64)
}

#[rustler::nif]
fn parse_string(language: String, source: String) -> NifResult<ResourceArc<TreeResource>> {
    let lang = ts_pack_core::get_language(&language)
        .map_err(|_| Error::Term(Box::new((atoms::language_not_found(), language.clone()))))?;
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang)
        .map_err(|e| Error::Term(Box::new((atoms::parse_error(), format!("{e}")))))?;
    let tree = parser
        .parse(source.as_bytes(), None)
        .ok_or_else(|| Error::Term(Box::new((atoms::parse_error(), "parsing returned no tree".to_string()))))?;
    Ok(ResourceArc::new(TreeResource(Mutex::new(tree))))
}

#[rustler::nif]
fn tree_root_node_type(tree: ResourceArc<TreeResource>) -> NifResult<String> {
    let guard = tree
        .0
        .lock()
        .map_err(|_| Error::Term(Box::new((atoms::parse_error(), "lock poisoned".to_string()))))?;
    Ok(guard.root_node().kind().to_string())
}

#[rustler::nif]
fn tree_root_child_count(tree: ResourceArc<TreeResource>) -> NifResult<u32> {
    let guard = tree
        .0
        .lock()
        .map_err(|_| Error::Term(Box::new((atoms::parse_error(), "lock poisoned".to_string()))))?;
    Ok(guard.root_node().named_child_count() as u32)
}

#[rustler::nif]
fn tree_contains_node_type(tree: ResourceArc<TreeResource>, node_type: String) -> NifResult<bool> {
    let guard = tree
        .0
        .lock()
        .map_err(|_| Error::Term(Box::new((atoms::parse_error(), "lock poisoned".to_string()))))?;
    let mut cursor = guard.walk();
    Ok(traverse_looking_for(&mut cursor, |node| node.kind() == node_type))
}

#[rustler::nif]
fn tree_has_error_nodes(tree: ResourceArc<TreeResource>) -> NifResult<bool> {
    let guard = tree
        .0
        .lock()
        .map_err(|_| Error::Term(Box::new((atoms::parse_error(), "lock poisoned".to_string()))))?;
    let mut cursor = guard.walk();
    Ok(traverse_looking_for(&mut cursor, |node| {
        node.is_error() || node.is_missing()
    }))
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

rustler::init!("Elixir.TreeSitterLanguagePack");
