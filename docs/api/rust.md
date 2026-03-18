---
description: "Rust API reference for tree-sitter-language-pack"
---

# Rust API Reference

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
tree-sitter-language-pack = "1.0"
```text

With default features (includes download API):

```toml
[dependencies]
tree-sitter-language-pack = { version = "1.0", features = ["download"] }
```text

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
println!("{}", tree.root_node().sexp());

// Extract code intelligence
let config = ProcessConfig::new("python").all();
let result = process("def hello(): pass", &config).unwrap();
println!("Functions: {}", result.structure.len());
```text

## Download Management

### `init(config: &PackConfig) -> Result<(), Error>`

Initialize the language pack with configuration.

Downloads all languages and groups specified in the config.

**Parameters:**

- `config` (&PackConfig): Configuration with languages and cache dir

**Returns:** Result<(), Error>

**Errors:**

- Error::Download: If downloads fail or network unavailable
- Error::LockPoisoned: If mutex lock fails

**Example:**

```rust
use tree_sitter_language_pack::{PackConfig, init};
use std::path::PathBuf;

let config = PackConfig {
    cache_dir: None,
    languages: Some(vec!["python".to_string(), "rust".to_string()]),
    groups: None,
};
init(&config)?;
```text

### `configure(config: &PackConfig) -> Result<(), Error>`

Apply configuration without downloading.

Use to set a custom cache directory before first `get_language` call.

**Parameters:**

- `config` (&PackConfig): Configuration

**Returns:** Result<(), Error>

**Errors:**

- Error::LockPoisoned: If mutex lock fails

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
```text

### `download(names: &[&str]) -> Result<usize, Error>`

Download specific languages to cache.

Returns number of newly downloaded languages.

**Parameters:**

- `names` (&[&str]): Language names to download

**Returns:** Result<usize, Error> - Count of newly downloaded

**Errors:**

- Error::Download: If language not in manifest or download fails
- Error::LanguageNotFound: If language not recognized

**Example:**

```rust
use tree_sitter_language_pack::download;

let count = download(&["python", "rust", "typescript"])?;
println!("Downloaded {} new languages", count);
```text

### `download_all() -> Result<usize, Error>`

Download all available languages (170+).

Returns number of newly downloaded languages.

**Returns:** Result<usize, Error> - Count of newly downloaded

**Errors:**

- Error::Download: If manifest fetch fails

**Example:**

```rust
use tree_sitter_language_pack::download_all;

let count = download_all()?;
println!("Downloaded {} languages total", count);
```text

### `manifest_languages() -> Result<Vec<String>, Error>`

Get all available languages from remote manifest.

Fetches and caches the manifest.

**Returns:** Result<Vec<String>, Error> - Sorted language names

**Errors:**

- Error::Download: If manifest fetch fails

**Example:**

```rust
use tree_sitter_language_pack::manifest_languages;

let languages = manifest_languages()?;
println!("Available: {}", languages.len());
```text

### `downloaded_languages() -> Vec<String>`

Get languages already cached locally.

No network requests. Returns empty if cache unavailable.

**Returns:** Vec<String> - Cached language names

**Example:**

```rust
use tree_sitter_language_pack::downloaded_languages;

let cached = downloaded_languages();
for lang in cached {
    println!("{}", lang);
}
```text

### `clean_cache() -> Result<(), Error>`

Delete all cached parser libraries.

**Returns:** Result<(), Error>

**Errors:**

- Error::Download: If cache cannot be removed

**Example:**

```rust
use tree_sitter_language_pack::clean_cache;

clean_cache()?;
println!("Cache cleaned");
```text

### `cache_dir() -> Result<PathBuf, Error>`

Get the current cache directory path.

**Returns:** Result<PathBuf, Error>

**Example:**

```rust
use tree_sitter_language_pack::cache_dir;

let dir = cache_dir()?;
println!("Cache: {}", dir.display());
```text

## Language Discovery

### `get_language(name: &str) -> Result<Language, Error>`

Get a tree-sitter Language by name.

Resolves aliases (e.g., `"shell"` → `"bash"`). Auto-downloads if needed.

**Parameters:**

- `name` (&str): Language name or alias

**Returns:** Result<Language, Error> - tree-sitter Language

**Errors:**

- Error::LanguageNotFound: If language not recognized
- Error::Download: If auto-download fails

**Example:**

```rust
use tree_sitter_language_pack::get_language;
use tree_sitter::Parser;

let language = get_language("python")?;

let mut parser = Parser::new();
parser.set_language(&language)?;
let tree = parser.parse("x = 1", None).unwrap();
assert_eq!(tree.root_node().kind(), "module");
```text

### `get_parser(name: &str) -> Result<Parser, Error>`

Get a pre-configured Parser for a language.

**Parameters:**

- `name` (&str): Language name or alias

**Returns:** Result<Parser, Error> - Pre-configured Parser

**Errors:**

- Error::LanguageNotFound: If language not recognized
- Error::ParserSetup: If parser setup fails

**Example:**

```rust
use tree_sitter_language_pack::get_parser;

let mut parser = get_parser("rust")?;
let tree = parser.parse("fn main() {}", None)?;
assert!(!tree.root_node().has_error());
```text

### `available_languages() -> Vec<String>`

List all available language names.

**Returns:** Vec<String> - Sorted, deduplicated names

**Example:**

```rust
use tree_sitter_language_pack::available_languages;

let langs = available_languages();
for lang in &langs {
    println!("{}", lang);
}
```text

### `has_language(name: &str) -> bool`

Check if a language is available.

**Parameters:**

- `name` (&str): Language name or alias

**Returns:** bool - True if available

**Example:**

```rust
use tree_sitter_language_pack::has_language;

if has_language("python") {
    println!("Python available");
}
assert!(has_language("shell")); // alias for bash
```text

### `language_count() -> usize`

Get total number of available languages.

**Returns:** usize - Language count

**Example:**

```rust
use tree_sitter_language_pack::language_count;

let count = language_count();
println!("{} languages", count);
```text

## Parsing

### `parse_string(source: &str, language: &Language) -> Result<Tree, Error>`

Parse source code into a syntax tree.

**Parameters:**

- `source` (&str): Source code
- `language` (&Language): tree-sitter Language

**Returns:** Result<Tree, Error> - Parsed tree

**Example:**

```rust
use tree_sitter_language_pack::{get_language, parse_string};

let language = get_language("python")?;
let tree = parse_string("x = 1", &language)?;
println!("{}", tree.root_node().sexp());
```text

### `tree_contains_node_type(tree: &Tree, node_type: &str) -> bool`

Check if tree contains a specific node type.

**Parameters:**

- `tree` (&Tree): Syntax tree
- `node_type` (&str): Node type name

**Returns:** bool - True if type exists

**Example:**

```rust
use tree_sitter_language_pack::{get_parser, tree_contains_node_type};

let mut parser = get_parser("python")?;
let tree = parser.parse("def foo(): pass", None)?;
assert!(tree_contains_node_type(&tree, "function_definition"));
```text

### `tree_to_sexp(tree: &Tree) -> String`

Get S-expression representation of tree.

**Parameters:**

- `tree` (&Tree): Syntax tree

**Returns:** String - S-expression

**Example:**

```rust
use tree_sitter_language_pack::{get_parser, tree_to_sexp};

let mut parser = get_parser("python")?;
let tree = parser.parse("x = 1", None)?;
println!("{}", tree_to_sexp(&tree));
```text

## Code Intelligence

### `process(source: &str, config: &ProcessConfig) -> Result<ProcessResult, Error>`

Extract code intelligence from source code.

**Parameters:**

- `source` (&str): Source code
- `config` (&ProcessConfig): Configuration

**Returns:** Result<ProcessResult, Error> - Analysis result

**Errors:**

- Error::LanguageNotFound: If language not found
- Error::Parse: If parsing fails

**Example:**

```rust
use tree_sitter_language_pack::{ProcessConfig, process};

let config = ProcessConfig::new("python").all();
let result = process("def hello(): pass", &config)?;
println!("Functions: {}", result.structure.len());
println!("Total lines: {}", result.metrics.total_lines);
```text

## Types

### `ProcessConfig`

Configuration for code intelligence analysis.

**Builder Pattern:**

```rust
let config = ProcessConfig::new("python")
    .with_structure(true)
    .with_imports(true)
    .with_exports(true)
    .with_chunks(2000, 400);
```text

**Methods:**

#### `new(language: &str) -> Self`

Create config for a language.

#### `with_structure(mut self, enabled: bool) -> Self`

Enable/disable structure extraction.

#### `with_imports(mut self, enabled: bool) -> Self`

Enable/disable import extraction.

#### `with_exports(mut self, enabled: bool) -> Self`

Enable/disable export extraction.

#### `with_comments(mut self, enabled: bool) -> Self`

Enable/disable comment extraction.

#### `with_docstrings(mut self, enabled: bool) -> Self`

Enable/disable docstring extraction.

#### `with_symbols(mut self, enabled: bool) -> Self`

Enable/disable symbol extraction.

#### `with_metrics(mut self, enabled: bool) -> Self`

Enable/disable metric extraction.

#### `with_diagnostics(mut self, enabled: bool) -> Self`

Enable/disable diagnostic extraction.

#### `with_chunks(mut self, max_size: usize, overlap: usize) -> Self`

Configure code chunking.

#### `all() -> Self`

Enable all features.

**Example:**

```rust
use tree_sitter_language_pack::ProcessConfig;

let config = ProcessConfig::new("python")
    .with_structure(true)
    .with_imports(true)
    .with_chunks(1024, 200)
    .all();
```text

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
```text

**Example:**

```rust
let result = process(source, &config)?;
println!("Language: {}", result.language);
println!("Structures: {}", result.structure.len());
println!("Imports: {}", result.imports.len());
```text

### `PackConfig`

Configuration for initialization and downloading.

**Fields:**

```rust
pub struct PackConfig {
    pub cache_dir: Option<PathBuf>,
    pub languages: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}
```text

**Example:**

```rust
use std::path::PathBuf;
use tree_sitter_language_pack::PackConfig;

let config = PackConfig {
    cache_dir: Some(PathBuf::from("/opt/ts-pack")),
    languages: Some(vec!["python".to_string(), "rust".to_string()]),
    groups: Some(vec!["web".to_string()]),
};
```text

### `Error`

Error type for all operations.

**Variants:**

```rust
pub enum Error {
    LanguageNotFound(String),
    ParserSetup(String),
    Parse(String),
    Download(String),
    LockPoisoned(String),
    Io(String),
    // ... other variants
}
```text

**Example:**

```rust
use tree_sitter_language_pack::{get_language, Error};

match get_language("python") {
    Ok(lang) => println!("Got Python"),
    Err(Error::LanguageNotFound(name)) => println!("Not found: {}", name),
    Err(e) => println!("Error: {:?}", e),
}
```text

## Feature Flags

### `download` (default)

Enable download API and automatic language retrieval.

```toml
[dependencies]
tree-sitter-language-pack = { version = "1.0", features = ["download"] }
```text

### Minimal Installation (no download)

```toml
[dependencies]
tree-sitter-language-pack = { version = "1.0", default-features = false }
```text

## Usage Patterns

### Pre-warming Cache

```rust
use tree_sitter_language_pack::{PackConfig, init};

let config = PackConfig {
    cache_dir: None,
    languages: Some(vec![
        "python".to_string(),
        "rust".to_string(),
        "typescript".to_string(),
    ]),
    groups: None,
};

init(&config)?;
```text

### Custom Cache Directory

```rust
use std::path::PathBuf;
use tree_sitter_language_pack::{PackConfig, configure};

let config = PackConfig {
    cache_dir: Some(PathBuf::from("/data/ts-pack")),
    languages: None,
    groups: None,
};

configure(&config)?;
```text

### Batch Processing

```rust
use tree_sitter_language_pack::{ProcessConfig, process};
use std::fs;

let config = ProcessConfig::new("python").all();

for entry in fs::read_dir("./src")? {
    let path = entry?.path();
    if path.extension().map_or(false, |ext| ext == "py") {
        let source = fs::read_to_string(&path)?;
        let result = process(&source, &config)?;
        println!("{}: {} items", path.display(), result.structure.len());
    }
}
```text

### Error Handling

```rust
use tree_sitter_language_pack::{get_language, Error};

match get_language("python") {
    Ok(lang) => {
        // Use language
    }
    Err(Error::LanguageNotFound(name)) => {
        eprintln!("Language not available: {}", name);
    }
    Err(Error::Download(msg)) => {
        eprintln!("Download failed: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {:?}", e);
    }
}
```
