use magnus::{Error, Ruby, function, method, prelude::*};
use std::sync::Mutex;

/// Wraps a tree-sitter Tree for safe sharing across the Ruby boundary.
#[magnus::wrap(class = "TreeSitterLanguagePack::Tree")]
struct TreeWrapper(Mutex<tree_sitter::Tree>);

/// Helper to create a runtime error from instance methods where `&Ruby` is not available.
fn lock_error() -> Error {
    // SAFETY: This is called from Ruby-invoked methods, so the Ruby VM is active.
    let ruby = unsafe { Ruby::get_unchecked() };
    Error::new(ruby.exception_runtime_error(), "lock poisoned")
}

impl TreeWrapper {
    fn root_node_type(&self) -> Result<String, Error> {
        let guard = self.0.lock().map_err(|_| lock_error())?;
        Ok(guard.root_node().kind().to_string())
    }

    fn root_child_count(&self) -> Result<usize, Error> {
        let guard = self.0.lock().map_err(|_| lock_error())?;
        Ok(guard.root_node().named_child_count())
    }

    fn contains_node_type(&self, node_type: String) -> Result<bool, Error> {
        let guard = self.0.lock().map_err(|_| lock_error())?;
        let mut cursor = guard.walk();
        Ok(traverse_looking_for(&mut cursor, |node| node.kind() == node_type))
    }

    fn has_error_nodes(&self) -> Result<bool, Error> {
        let guard = self.0.lock().map_err(|_| lock_error())?;
        let mut cursor = guard.walk();
        Ok(traverse_looking_for(&mut cursor, |node| {
            node.is_error() || node.is_missing()
        }))
    }
}

fn available_languages() -> Vec<String> {
    ts_pack_core::available_languages()
}

fn has_language(name: String) -> bool {
    ts_pack_core::has_language(&name)
}

fn language_count() -> usize {
    ts_pack_core::language_count()
}

fn get_language_ptr(ruby: &Ruby, name: String) -> Result<u64, Error> {
    let language = ts_pack_core::get_language(&name)
        .map_err(|_| Error::new(ruby.exception_runtime_error(), format!("language not found: {name}")))?;
    let raw_ptr = language.into_raw();
    Ok(raw_ptr as u64)
}

fn parse_string(ruby: &Ruby, language: String, source: String) -> Result<TreeWrapper, Error> {
    let lang = ts_pack_core::get_language(&language).map_err(|_| {
        Error::new(
            ruby.exception_runtime_error(),
            format!("language not found: {language}"),
        )
    })?;
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang)
        .map_err(|e| Error::new(ruby.exception_runtime_error(), format!("failed to set language: {e}")))?;
    let tree = parser
        .parse(source.as_bytes(), None)
        .ok_or_else(|| Error::new(ruby.exception_runtime_error(), "parsing returned no tree".to_string()))?;
    Ok(TreeWrapper(Mutex::new(tree)))
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

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("TreeSitterLanguagePack")?;

    module.define_module_function("available_languages", function!(available_languages, 0))?;
    module.define_module_function("has_language", function!(has_language, 1))?;
    module.define_module_function("language_count", function!(language_count, 0))?;
    module.define_module_function("get_language_ptr", function!(get_language_ptr, 1))?;
    module.define_module_function("parse_string", function!(parse_string, 2))?;

    let tree_class = module.define_class("Tree", ruby.class_object())?;
    tree_class.define_method("root_node_type", method!(TreeWrapper::root_node_type, 0))?;
    tree_class.define_method("root_child_count", method!(TreeWrapper::root_child_count, 0))?;
    tree_class.define_method("contains_node_type", method!(TreeWrapper::contains_node_type, 1))?;
    tree_class.define_method("has_error_nodes", method!(TreeWrapper::has_error_nodes, 0))?;

    Ok(())
}
