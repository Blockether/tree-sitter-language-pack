---
description: "Rust API reference for tree-sitter-language-pack"
---

# Rust API Reference

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
tree-sitter-language-pack = "1.0"
```

With download feature (default):

```toml
[dependencies]
tree-sitter-language-pack = { version = "1.0", features = ["download"] }
```

## Quick Start

```rust
use tree_sitter_language_pack::{
    ProcessConfig, available_languages, has_language,
    get_language, get_parser, process, download,
};

// Pre-download languages
download(&["python", "rust"]).unwrap();

// Get a language
let language = get_language("python").unwrap();

// Get a pre-configured parser
let mut parser = get_parser("python").unwrap();
let tree = parser.parse("def hello(): pass", None).unwrap();
println!("{}", tree.root_node().to_sexp());

// Extract code intelligence
let config = ProcessConfig::new("python").all();
let result = process("def hello(): pass", &config).unwrap();
println!("Functions: {}", result.structure.len());
```

## Download Management

These functions require the `download` feature (enabled by default).

### `init(config: &PackConfig) -> Result<(), Error>`

Initialize the language pack with configuration. Downloads all languages and groups specified in the config.

**Parameters:**

- `config` (&PackConfig): Configuration with cache directory, languages, and groups

**Returns:** Result<(), Error>

**Example:**

```rust
use tree_sitter_language_pack::{PackConfig, init};

let config = PackConfig {
    cache_dir: None,
    languages: Some(vec!["python".to_string(), "rust".to_string()]),
    groups: None,
};
init(&config)?;
```

### `configure(config: &PackConfig) -> Result<(), Error>`

Apply configuration without downloading. Use to set a custom cache directory before the first `get_language` call.

**Parameters:**

- `config` (&PackConfig): Configuration

**Returns:** Result<(), Error>

**Example:**

```rust
use tree_sitter_language_pack::{PackConfig, configure};
use std::path::PathBuf;

let config = PackConfig {
    cache_dir: Some(PathBuf::from("/opt/ts-pack-cache")),
    languages: None,
    groups: None,
};
configure(&config)?;
```

### `download(names: &[&str]) -> Result<usize, Error>`

Download specific languages to cache. Returns the number of newly downloaded languages.

**Parameters:**

- `names` (&[&str]): Language names to download

**Returns:** Result<usize, Error>

**Example:**

```rust
use tree_sitter_language_pack::download;

let count = download(&["python", "rust", "typescript"])?;
println!("Downloaded {} new languages", count);
```

### `download_all() -> Result<usize, Error>`

Download all available languages from the remote manifest. Returns the number of newly downloaded languages.

**Returns:** Result<usize, Error>

**Example:**

```rust
use tree_sitter_language_pack::download_all;

let count = download_all()?;
println!("Downloaded {} languages total", count);
```

### `manifest_languages() -> Result<Vec<String>, Error>`

Get all available languages from the remote manifest. Fetches and caches the manifest.

**Returns:** Result<Vec<String>, Error> - Sorted language names

**Example:**

```rust
use tree_sitter_language_pack::manifest_languages;

let languages = manifest_languages()?;
println!("Available: {}", languages.len());
```

### `downloaded_languages() -> Vec<String>`

Get languages already cached locally. No network requests. Returns empty if cache unavailable.

**Returns:** Vec<String>

**Example:**

```rust
use tree_sitter_language_pack::downloaded_languages;

let cached = downloaded_languages();
for lang in cached {
    println!("{}", lang);
}
```

### `clean_cache() -> Result<(), Error>`

Delete all cached parser shared libraries.

**Returns:** Result<(), Error>

**Example:**

```rust
use tree_sitter_language_pack::clean_cache;

clean_cache()?;
```

### `cache_dir() -> Result<PathBuf, Error>`

Get the effective cache directory path. Either the custom path set via `configure`/`init` or the default `~/.cache/tree-sitter-language-pack/v{version}/libs/`.

**Returns:** Result<PathBuf, Error>

**Example:**

```rust
use tree_sitter_language_pack::cache_dir;

let dir = cache_dir()?;
println!("Cache: {}", dir.display());
```

## Language Discovery

### `get_language(name: &str) -> Result<Language, Error>`

Get a tree-sitter `Language` by name. Resolves aliases (e.g., `"shell"` maps to `"bash"`). With the `download` feature, auto-downloads the parser if not found locally.

**Parameters:**

- `name` (&str): Language name or alias

**Returns:** Result<Language, Error>

**Example:**

```rust
use tree_sitter_language_pack::get_language;

let language = get_language("python")?;
let mut parser = tree_sitter::Parser::new();
parser.set_language(&language)?;
let tree = parser.parse("x = 1", None).unwrap();
assert_eq!(tree.root_node().kind(), "module");
```

### `get_parser(name: &str) -> Result<Parser, Error>`

Get a pre-configured `Parser` for a language. Convenience wrapper that calls `get_language` and sets up a new parser.

**Parameters:**

- `name` (&str): Language name or alias

**Returns:** Result<Parser, Error>

**Example:**

```rust
use tree_sitter_language_pack::get_parser;

let mut parser = get_parser("rust")?;
let tree = parser.parse("fn main() {}", None).unwrap();
assert!(!tree.root_node().has_error());
```

### `available_languages() -> Vec<String>`

List all available language names (sorted, deduplicated, includes aliases).

**Returns:** Vec<String>

**Example:**

```rust
use tree_sitter_language_pack::available_languages;

let langs = available_languages();
for lang in &langs {
    println!("{}", lang);
}
```

### `has_language(name: &str) -> bool`

Check if a language is available by name or alias.

**Parameters:**

- `name` (&str): Language name or alias

**Returns:** bool

**Example:**

```rust
use tree_sitter_language_pack::has_language;

assert!(has_language("python"));
assert!(has_language("shell")); // alias for bash
assert!(!has_language("nonexistent_language"));
```

### `language_count() -> usize`

Return the number of available languages.

**Returns:** usize

**Example:**

```rust
use tree_sitter_language_pack::language_count;

let count = language_count();
println!("{} languages", count);
```

## Language Detection

### `detect_language_from_extension(ext: &str) -> Option<&'static str>`

Detect language name from a file extension (without leading dot). Case-insensitive.

**Parameters:**

- `ext` (&str): File extension without dot

**Returns:** Option<&'static str>

**Example:**

```rust
use tree_sitter_language_pack::detect_language_from_extension;

assert_eq!(detect_language_from_extension("py"), Some("python"));
assert_eq!(detect_language_from_extension("RS"), Some("rust"));
assert_eq!(detect_language_from_extension("xyz"), None);
```

### `detect_language_from_path(path: &str) -> Option<&'static str>`

Detect language name from a file path by extracting and looking up the extension.

**Parameters:**

- `path` (&str): File path

**Returns:** Option<&'static str>

**Example:**

```rust
use tree_sitter_language_pack::detect_language_from_path;

assert_eq!(detect_language_from_path("src/main.rs"), Some("rust"));
assert_eq!(detect_language_from_path("README.md"), Some("markdown"));
assert_eq!(detect_language_from_path("Makefile"), None);
```

### `detect_language_from_content(content: &str) -> Option<&'static str>`

Detect language from file content using the shebang line (`#!`). Inspects only the first line.

**Parameters:**

- `content` (&str): File content

**Returns:** Option<&'static str>

**Example:**

```rust
use tree_sitter_language_pack::detect_language_from_content;

assert_eq!(detect_language_from_content("#!/usr/bin/env python3\npass"), Some("python"));
assert_eq!(detect_language_from_content("#!/bin/bash\necho hi"), Some("bash"));
assert_eq!(detect_language_from_content("no shebang here"), None);
```

### `extension_ambiguity(ext: &str) -> Option<(&'static str, &'static [&'static str])>`

Check if a file extension is ambiguous. Returns the assigned language and alternatives if ambiguous, or `None` if unambiguous or unrecognized.

**Parameters:**

- `ext` (&str): File extension without dot

**Returns:** Option<(&'static str, &'static [&'static str])>

**Example:**

```rust
use tree_sitter_language_pack::extension_ambiguity;

if let Some((assigned, alternatives)) = extension_ambiguity("m") {
    assert_eq!(assigned, "objc");
    assert!(alternatives.contains(&"matlab"));
}
assert!(extension_ambiguity("py").is_none());
```

## Parsing

### `parse_string(language: &str, source: &[u8]) -> Result<Tree, Error>`

Parse source code with the named language, returning the syntax tree. Uses the global registry to look up the language.

**Parameters:**

- `language` (&str): Language name
- `source` (&[u8]): Source code as bytes

**Returns:** Result<Tree, Error>

**Example:**

```rust
use tree_sitter_language_pack::parse_string;

let tree = parse_string("python", b"x = 1")?;
assert_eq!(tree.root_node().kind(), "module");
```

### `tree_contains_node_type(tree: &Tree, node_type: &str) -> bool`

Check if any node in the tree matches the given type name. Performs a depth-first traversal.

**Parameters:**

- `tree` (&Tree): Syntax tree
- `node_type` (&str): Node type name

**Returns:** bool

**Example:**

```rust
use tree_sitter_language_pack::{parse_string, tree_contains_node_type};

let tree = parse_string("python", b"def foo(): pass")?;
assert!(tree_contains_node_type(&tree, "function_definition"));
```

### `tree_has_error_nodes(tree: &Tree) -> bool`

Check whether the tree contains any ERROR or MISSING nodes.

**Parameters:**

- `tree` (&Tree): Syntax tree

**Returns:** bool

### `tree_to_sexp(tree: &Tree) -> String`

Return the S-expression representation of the tree.

**Parameters:**

- `tree` (&Tree): Syntax tree

**Returns:** String

**Example:**

```rust
use tree_sitter_language_pack::{parse_string, tree_to_sexp};

let tree = parse_string("python", b"x = 1")?;
println!("{}", tree_to_sexp(&tree));
```

### `tree_error_count(tree: &Tree) -> usize`

Count the number of ERROR and MISSING nodes in the tree. Returns 0 for a clean parse.

**Parameters:**

- `tree` (&Tree): Syntax tree

**Returns:** usize

## Node Inspection

### `NodeInfo`

Lightweight owned snapshot of a tree-sitter node's properties.

**Fields:**

```rust
pub struct NodeInfo {
    pub kind: Cow<'static, str>,
    pub is_named: bool,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
    pub named_child_count: usize,
    pub is_error: bool,
    pub is_missing: bool,
}
```

### `node_info_from_node(node: Node) -> NodeInfo`

Extract a `NodeInfo` snapshot from a tree-sitter `Node`.

### `root_node_info(tree: &Tree) -> NodeInfo`

Get a `NodeInfo` snapshot of the root node.

### `find_nodes_by_type(tree: &Tree, node_type: &str) -> Vec<NodeInfo>`

Find all nodes matching the given type name. Returns their `NodeInfo` via depth-first traversal.

### `named_children_info(tree: &Tree) -> Vec<NodeInfo>`

Get `NodeInfo` for all named children of the root node.

### `extract_text<'a>(source: &'a [u8], node_info: &NodeInfo) -> Result<&'a str, Error>`

Extract the source text corresponding to a node's byte range.

**Parameters:**

- `source` (&[u8]): Source code bytes
- `node_info` (&NodeInfo): Node with byte range

**Returns:** Result<&str, Error>

## Query Execution

### `run_query(tree: &Tree, language: &str, query_source: &str, source: &[u8]) -> Result<Vec<QueryMatch>, Error>`

Execute a tree-sitter query pattern against a parsed tree. Returns all matches with their captured nodes.

**Parameters:**

- `tree` (&Tree): Parsed syntax tree
- `language` (&str): Language name (used to compile the query)
- `query_source` (&str): Tree-sitter query pattern string
- `source` (&[u8]): Original source code bytes

**Returns:** Result<Vec<QueryMatch>, Error>

**Example:**

```rust
use tree_sitter_language_pack::{parse_string, run_query};

let tree = parse_string("python", b"def hello(): pass")?;
let matches = run_query(
    &tree,
    "python",
    "(function_definition name: (identifier) @fn_name)",
    b"def hello(): pass",
)?;
assert!(!matches.is_empty());
```

### `QueryMatch`

A single match from a tree-sitter query.

**Fields:**

```rust
pub struct QueryMatch {
    pub pattern_index: usize,
    pub captures: Vec<(Cow<'static, str>, NodeInfo)>,
}
```

## Bundled Queries

### `get_highlights_query(language: &str) -> Option<&'static str>`

Get the bundled highlights query (`highlights.scm`) for a language, if available.

### `get_injections_query(language: &str) -> Option<&'static str>`

Get the bundled injections query (`injections.scm`) for a language, if available.

### `get_locals_query(language: &str) -> Option<&'static str>`

Get the bundled locals query (`locals.scm`) for a language, if available.

## Code Chunking

### `split_code(source: &str, tree: &Tree, max_chunk_size: usize) -> Vec<(usize, usize)>`

Split source code into chunks using tree-sitter AST structure for intelligent boundaries. Returns `(start_byte, end_byte)` ranges.

**Parameters:**

- `source` (&str): Full source code
- `tree` (&Tree): Parsed tree-sitter tree
- `max_chunk_size` (usize): Maximum chunk size in bytes

**Returns:** Vec<(usize, usize)>

## Code Intelligence

### `process(source: &str, config: &ProcessConfig) -> Result<ProcessResult, Error>`

Parse source code and extract file intelligence using the global registry. Extracts metrics, structure, imports, exports, comments, docstrings, symbols, diagnostics, and/or chunks based on config flags.

**Parameters:**

- `source` (&str): Source code
- `config` (&ProcessConfig): Configuration

**Returns:** Result<ProcessResult, Error>

**Example:**

```rust
use tree_sitter_language_pack::{ProcessConfig, process};

let config = ProcessConfig::new("python").all();
let result = process("def hello(): pass", &config)?;
println!("Functions: {}", result.structure.len());
println!("Total lines: {}", result.metrics.total_lines);
```

## Types

### `ProcessConfig`

Configuration for the `process()` function. Controls which analysis features are enabled.

**Fields:**

```rust
pub struct ProcessConfig {
    pub language: Cow<'static, str>,
    pub structure: bool,       // default: true
    pub imports: bool,         // default: true
    pub exports: bool,         // default: true
    pub comments: bool,        // default: false
    pub docstrings: bool,      // default: false
    pub symbols: bool,         // default: false
    pub diagnostics: bool,     // default: false
    pub chunk_max_size: Option<usize>,  // default: None
}
```

**Methods:**

#### `new(language: impl Into<String>) -> Self`

Create a config with defaults (structure, imports, exports enabled).

#### `with_chunking(mut self, max_size: usize) -> Self`

Enable chunking with the given maximum chunk size in bytes.

#### `all(mut self) -> Self`

Enable all analysis features (structure, imports, exports, comments, docstrings, symbols, diagnostics).

#### `minimal(mut self) -> Self`

Disable all analysis features (only metrics computed).

**Example:**

```rust
use tree_sitter_language_pack::ProcessConfig;

let config = ProcessConfig::new("python")
    .all()
    .with_chunking(2000);
```

### `ProcessResult`

Result from code intelligence analysis.

**Fields:**

```rust
pub struct ProcessResult {
    pub language: String,
    pub metrics: FileMetrics,
    pub structure: Vec<StructureItem>,
    pub imports: Vec<ImportInfo>,
    pub exports: Vec<ExportInfo>,
    pub comments: Vec<CommentInfo>,
    pub docstrings: Vec<DocstringInfo>,
    pub symbols: Vec<SymbolInfo>,
    pub diagnostics: Vec<Diagnostic>,
    pub chunks: Vec<CodeChunk>,
    pub parse_errors: usize,
}
```

### `PackConfig`

Configuration for initialization and downloading.

**Fields:**

```rust
pub struct PackConfig {
    pub cache_dir: Option<PathBuf>,
    pub languages: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}
```

### `Error`

Error type for all operations.

**Variants:**

```rust
pub enum Error {
    LanguageNotFound(String),
    DynamicLoad(String),
    NullLanguagePointer(String),
    ParserSetup(String),
    LockPoisoned(String),
    Config(String),
    ParseFailed,
    QueryError(String),
    InvalidRange(String),
    Io(std::io::Error),
    Json(serde_json::Error),           // requires "config" or "download" feature
    Toml(toml::de::Error),            // requires "config" feature
    Download(String),                   // requires "download" feature
    ChecksumMismatch { file, expected, actual },  // requires "download" feature
}
```

**Example:**

```rust
use tree_sitter_language_pack::{get_language, Error};

match get_language("python") {
    Ok(lang) => println!("Got Python"),
    Err(Error::LanguageNotFound(name)) => println!("Not found: {}", name),
    Err(e) => println!("Error: {:?}", e),
}
```

## Feature Flags

### `download` (default)

Enables download API and automatic language retrieval: `init`, `configure`, `download`, `download_all`, `manifest_languages`, `downloaded_languages`, `clean_cache`, `cache_dir`.

### `serde`

Enables JSON serialization for `ProcessConfig`, `ProcessResult`, and related types. Also enables `extension_ambiguity_json`.

### `config`

Enables `PackConfig::from_toml_file` and `PackConfig::discover`.

### Minimal Installation (no download)

```toml
[dependencies]
tree-sitter-language-pack = { version = "1.0", default-features = false }
```

## Re-exports

The crate re-exports `tree_sitter::{Language, Parser, Tree}` for convenience.
