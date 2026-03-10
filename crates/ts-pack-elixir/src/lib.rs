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
        .map_err(|_| Error::RaiseTerm(Box::new((atoms::language_not_found(), name.clone()))))?;
    let raw_ptr = language.into_raw();
    Ok(raw_ptr as u64)
}

#[rustler::nif]
fn parse_string(language: String, source: String) -> NifResult<ResourceArc<TreeResource>> {
    let tree = ts_pack_core::parse_string(&language, source.as_bytes())
        .map_err(|e| Error::RaiseTerm(Box::new((atoms::parse_error(), format!("{e}")))))?;
    Ok(ResourceArc::new(TreeResource(Mutex::new(tree))))
}

#[rustler::nif]
fn tree_root_node_type(tree: ResourceArc<TreeResource>) -> NifResult<String> {
    let guard = tree
        .0
        .lock()
        .map_err(|_| Error::RaiseTerm(Box::new((atoms::parse_error(), "lock poisoned".to_string()))))?;
    Ok(guard.root_node().kind().to_string())
}

#[rustler::nif]
fn tree_root_child_count(tree: ResourceArc<TreeResource>) -> NifResult<u32> {
    let guard = tree
        .0
        .lock()
        .map_err(|_| Error::RaiseTerm(Box::new((atoms::parse_error(), "lock poisoned".to_string()))))?;
    Ok(guard.root_node().named_child_count() as u32)
}

#[rustler::nif]
fn tree_contains_node_type(tree: ResourceArc<TreeResource>, node_type: String) -> NifResult<bool> {
    let guard = tree
        .0
        .lock()
        .map_err(|_| Error::RaiseTerm(Box::new((atoms::parse_error(), "lock poisoned".to_string()))))?;
    Ok(ts_pack_core::tree_contains_node_type(&guard, &node_type))
}

#[rustler::nif]
fn tree_has_error_nodes(tree: ResourceArc<TreeResource>) -> NifResult<bool> {
    let guard = tree
        .0
        .lock()
        .map_err(|_| Error::RaiseTerm(Box::new((atoms::parse_error(), "lock poisoned".to_string()))))?;
    Ok(ts_pack_core::tree_has_error_nodes(&guard))
}

// ---------------------------------------------------------------------------
// Intel: process / process_and_chunk
// ---------------------------------------------------------------------------

#[rustler::nif]
fn process(source: String, language: String) -> NifResult<String> {
    let registry = ts_pack_core::LanguageRegistry::new();
    let intel = registry
        .process(&source, &language)
        .map_err(|e| Error::RaiseTerm(Box::new((atoms::parse_error(), format!("{e}")))))?;
    serde_json::to_string(&intel)
        .map_err(|e| Error::RaiseTerm(Box::new((atoms::parse_error(), format!("serialization failed: {e}")))))
}

#[rustler::nif]
fn process_and_chunk(source: String, language: String, max_chunk_size: u64) -> NifResult<String> {
    let registry = ts_pack_core::LanguageRegistry::new();
    let (intel, chunks) = registry
        .process_and_chunk(&source, &language, max_chunk_size as usize)
        .map_err(|e| Error::RaiseTerm(Box::new((atoms::parse_error(), format!("{e}")))))?;
    let result = serde_json::json!({
        "intelligence": intel,
        "chunks": chunks,
    });
    serde_json::to_string(&result)
        .map_err(|e| Error::RaiseTerm(Box::new((atoms::parse_error(), format!("serialization failed: {e}")))))
}

rustler::init!("Elixir.TreeSitterLanguagePack");
