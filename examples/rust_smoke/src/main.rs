fn main() {
    let langs = tree_sitter_language_pack::available_languages();
    assert!(!langs.is_empty(), "Expected languages to be available");
    println!("Available languages: {}", langs.len());

    assert!(tree_sitter_language_pack::has_language("rust"), "rust should be available");

    let tree = tree_sitter_language_pack::parse_string("rust", b"fn main() {}").expect("parse should succeed");
    assert!(!tree_sitter_language_pack::tree_has_error_nodes(&tree), "tree should have no errors");

    println!("Rust smoke test passed");
}
