//! File metadata extraction and code chunking using tree-sitter.
//!
//! This module provides rich AST metadata extraction and intelligent code chunking.
//! It analyzes source code to extract structure, imports, exports, comments,
//! docstrings, symbols, and diagnostics.
//!
//! # Concurrency Safety
//!
//! All public functions (`analyze`, `chunk`, `process`) create a fresh
//! `tree_sitter::Parser` per call — no parser state is shared between calls.
//! They are safe to call concurrently from multiple threads without additional
//! synchronization.

pub mod analysis;
pub mod chunking;
pub mod types;

pub use types::*;

/// Extract file metadata from source code (structural analysis only, no chunking).
///
/// Parses the source once and returns rich metadata including structure,
/// imports, exports, comments, symbols, metrics, and diagnostics.
///
/// # Concurrency
///
/// Thread-safe: creates a fresh parser per call with no shared mutable state.
pub fn analyze(
    source: &str,
    language: &str,
    registry: &crate::LanguageRegistry,
) -> Result<FileMetadata, crate::Error> {
    let (_lang, tree) = parse_source(source, language, registry)?;
    Ok(analysis::extract_metadata(source, language, &tree))
}

/// Chunk source code into semantically meaningful pieces (no metadata extraction).
///
/// Parses the source once and splits it at AST-aware boundaries, producing
/// chunks with per-chunk metadata (symbols, comments, error status).
///
/// # Concurrency
///
/// Thread-safe: creates a fresh parser per call with no shared mutable state.
pub fn chunk(
    source: &str,
    language: &str,
    max_chunk_size: usize,
    registry: &crate::LanguageRegistry,
) -> Result<Vec<CodeChunk>, crate::Error> {
    let (lang, tree) = parse_source(source, language, registry)?;
    Ok(chunking::chunk_source(source, language, max_chunk_size, &lang, &tree))
}

/// Extract file metadata and chunk source code in a single pass.
///
/// Parses once and produces both file-level metadata and per-chunk metadata.
/// More efficient than calling `analyze` and `chunk` separately.
///
/// # Concurrency
///
/// Thread-safe: creates a fresh parser per call with no shared mutable state.
pub fn process(
    source: &str,
    language: &str,
    max_chunk_size: usize,
    registry: &crate::LanguageRegistry,
) -> Result<ProcessResult, crate::Error> {
    let (lang, tree) = parse_source(source, language, registry)?;
    let metadata = analysis::extract_metadata(source, language, &tree);
    let chunks = chunking::chunk_source(source, language, max_chunk_size, &lang, &tree);
    Ok(ProcessResult { metadata, chunks })
}

/// Parse source code and return the tree-sitter language and tree.
fn parse_source(
    source: &str,
    language: &str,
    registry: &crate::LanguageRegistry,
) -> Result<(tree_sitter::Language, tree_sitter::Tree), crate::Error> {
    let lang = registry.get_language(language)?;
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang)
        .map_err(|e| crate::Error::ParserSetup(e.to_string()))?;
    let tree = parser.parse(source, None).ok_or(crate::Error::ParseFailed)?;
    Ok((lang, tree))
}

#[cfg(test)]
mod tests {
    use crate::LanguageRegistry;

    fn first_lang(registry: &LanguageRegistry) -> Option<String> {
        let langs = registry.available_languages();
        langs.into_iter().next()
    }

    #[test]
    fn test_analyze_returns_metadata() {
        let registry = LanguageRegistry::new();
        let Some(lang) = first_lang(&registry) else { return };
        let source = "x";
        let result = super::analyze(source, &lang, &registry);
        assert!(result.is_ok(), "analyze should succeed for available language");
        let metadata = result.unwrap();
        assert_eq!(metadata.language, lang);
        assert!(metadata.metrics.total_lines >= 1);
        assert!(metadata.metrics.node_count > 0);
    }

    #[test]
    fn test_chunk_returns_chunks() {
        let registry = LanguageRegistry::new();
        let Some(lang) = first_lang(&registry) else { return };
        let source = "x";
        let result = super::chunk(source, &lang, 1000, &registry);
        assert!(result.is_ok(), "chunk should succeed");
        let chunks = result.unwrap();
        assert!(!chunks.is_empty(), "should have at least one chunk");
        assert_eq!(chunks[0].metadata.language, lang);
    }

    #[test]
    fn test_process_returns_both() {
        let registry = LanguageRegistry::new();
        let Some(lang) = first_lang(&registry) else { return };
        let source = "x";
        let result = super::process(source, &lang, 1000, &registry);
        assert!(result.is_ok());
        let pr = result.unwrap();
        assert_eq!(pr.metadata.language, lang);
        assert!(!pr.chunks.is_empty(), "should have at least one chunk");
        assert_eq!(pr.chunks[0].metadata.language, lang);
    }

    #[test]
    fn test_analyze_invalid_language() {
        let registry = LanguageRegistry::new();
        let result = super::analyze("x", "nonexistent_lang_xyz", &registry);
        assert!(result.is_err(), "should fail for nonexistent language");
    }

    #[test]
    fn test_analyze_empty_source() {
        let registry = LanguageRegistry::new();
        let Some(lang) = first_lang(&registry) else { return };
        let result = super::analyze("", &lang, &registry);
        assert!(result.is_ok(), "empty source should parse without error");
        let metadata = result.unwrap();
        assert_eq!(metadata.metrics.total_bytes, 0);
    }

    #[test]
    fn test_process_small_max_size() {
        let registry = LanguageRegistry::new();
        if !registry.has_language("python") {
            return;
        }
        let source = "def foo():\n    pass\ndef bar():\n    pass\n";
        let result = super::process(source, "python", 20, &registry);
        assert!(result.is_ok());
        let pr = result.unwrap();
        assert!(
            pr.chunks.len() >= 2,
            "small max_chunk_size should split into multiple chunks"
        );
        assert_eq!(pr.metadata.language, "python");
    }
}
