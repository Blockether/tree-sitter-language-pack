---
description: "Python API reference for tree-sitter-language-pack"
---

# Python API Reference

## Installation

=== "pip"

    ```bash
    pip install tree-sitter-language-pack
    ```

=== "uv"

    ```bash
    uv add tree-sitter-language-pack
    ```

=== "poetry"

    ```bash
    poetry add tree-sitter-language-pack
    ```

## Quick Start

```python
from tree_sitter_language_pack import (
    init,
    get_language,
    get_parser,
    process,
    ProcessConfig,
)

# Pre-download languages for offline use
init(["python", "javascript"])

# Get a language
language = get_language("python")

# Get a pre-configured parser
parser = get_parser("python")
tree = parser.parse(b"def hello(): pass")
print(tree.root_node.sexp())

# Extract code intelligence
config = ProcessConfig(language="python", chunk_max_size=1000)
result = process("def hello(): pass", config)
print(f"Functions: {len(result['structure'])}")
```text

## Download Management

### `init(languages: list[str] | None = None, groups: list[str] | None = None) -> None`

Initialize the language pack with optional pre-downloads.

**Parameters:**

- `languages` (list[str] | None): Languages to download (e.g., `["python", "rust"]`)
- `groups` (list[str] | None): Language groups to download (e.g., `["web", "data"]`)

**Returns:** None

**Raises:**

- `DownloadError`: If downloads fail or network is unavailable

**Example:**

```python
from tree_sitter_language_pack import init

# Pre-download specific languages
init(languages=["python", "javascript", "rust"])

# Or download language groups
init(groups=["web", "data"])

# Or both
init(languages=["python"], groups=["web"])
```text

### `configure(cache_dir: str | None = None) -> None`

Apply download configuration without downloading.

Use this to set a custom cache directory before the first call to `get_language` or any download function.

**Parameters:**

- `cache_dir` (str | None): Custom cache directory path

**Returns:** None

**Raises:**

- `DownloadError`: If the lock cannot be acquired

**Example:**

```python
from tree_sitter_language_pack import configure

# Set custom cache directory
configure(cache_dir="/opt/ts-pack-cache")

# Now get_language will use this cache
from tree_sitter_language_pack import get_language
language = get_language("python")
```text

### `download(names: list[str]) -> int`

Download specific languages to the local cache.

Returns the number of newly downloaded languages. Languages already cached are not re-downloaded.

**Parameters:**

- `names` (list[str]): Language names to download

**Returns:** int - Number of newly downloaded languages

**Raises:**

- `DownloadError`: If any language is not available or download fails
- `LanguageNotFoundError`: If a language is not in the manifest

**Example:**

```python
from tree_sitter_language_pack import download

# Download specific languages
count = download(["python", "rust", "typescript"])
print(f"Downloaded {count} new languages")
```text

### `download_all() -> int`

Download all available languages from the remote manifest (170+).

Returns the number of newly downloaded languages.

**Returns:** int - Number of newly downloaded languages

**Raises:**

- `DownloadError`: If the manifest cannot be fetched or downloads fail

**Example:**

```python
from tree_sitter_language_pack import download_all

# Download all 170+ languages
count = download_all()
print(f"Downloaded {count} languages total")
```text

### `manifest_languages() -> list[str]`

Return all language names available in the remote manifest.

Fetches (and caches) the remote manifest to discover the full list of downloadable languages.

**Returns:** list[str] - Sorted list of available language names

**Raises:**

- `DownloadError`: If the manifest cannot be fetched

**Example:**

```python
from tree_sitter_language_pack import manifest_languages

# Get all available languages
languages = manifest_languages()
print(f"Available languages: {', '.join(languages[:10])}...")
```text

### `downloaded_languages() -> list[str]`

Return languages already downloaded and cached locally.

Does not perform any network requests.

**Returns:** list[str] - Sorted list of cached language names

**Example:**

```python
from tree_sitter_language_pack import downloaded_languages

# Check what's already cached
cached = downloaded_languages()
print(f"Cached languages: {', '.join(cached)}")
```text

### `clean_cache() -> None`

Delete all cached parser shared libraries.

Resets the cache registration so the next call to `get_language` will re-scan the (now empty) cache directory.

**Returns:** None

**Raises:**

- `DownloadError`: If the cache directory cannot be removed

**Example:**

```python
from tree_sitter_language_pack import clean_cache

# Clear all cached parsers
clean_cache()
print("Cache cleaned")
```text

### `cache_dir() -> str`

Get the current cache directory path.

**Returns:** str - Absolute path to cache directory

**Example:**

```python
from tree_sitter_language_pack import cache_dir

# Get cache location
cache_path = cache_dir()
print(f"Cached at: {cache_path}")
```text

## Language Discovery

### `get_language(name: str) -> Language`

Get a tree-sitter Language by name.

Resolves language aliases (e.g., `"shell"` maps to `"bash"`). When the download feature is enabled (default), automatically downloads the parser if not found locally.

**Parameters:**

- `name` (str): Language name or alias

**Returns:** Language - tree-sitter Language object

**Raises:**

- `LanguageNotFoundError`: If language is not recognized
- `DownloadError`: If auto-download fails

**Example:**

```python
from tree_sitter_language_pack import get_language

# Get a language
language = get_language("python")

# Use with tree-sitter Parser
import tree_sitter
parser = tree_sitter.Parser()
parser.set_language(language)
tree = parser.parse(b"x = 1")
print(tree.root_node.kind)  # "module"
```text

### `get_parser(name: str) -> Parser`

Get a tree-sitter Parser pre-configured for the given language.

Convenience function that calls `get_language` and configures a parser in one step.

**Parameters:**

- `name` (str): Language name or alias

**Returns:** Parser - Pre-configured tree-sitter Parser

**Raises:**

- `LanguageNotFoundError`: If language is not recognized
- `DownloadError`: If auto-download fails
- `ParseError`: If parser setup fails

**Example:**

```python
from tree_sitter_language_pack import get_parser

# Get pre-configured parser
parser = get_parser("rust")
tree = parser.parse(b"fn main() {}")
print(tree.root_node.has_error)  # False
```text

### `available_languages() -> list[str]`

List all available language names.

Returns names of both statically compiled and dynamically loadable languages, plus any configured aliases.

**Returns:** list[str] - Sorted, deduplicated list of language names

**Example:**

```python
from tree_sitter_language_pack import available_languages

# List all available languages
langs = available_languages()
for lang in langs:
    print(lang)
```text

### `has_language(name: str) -> bool`

Check if a language is available by name or alias.

Returns `True` if the language can be loaded (statically compiled, dynamically available, or a known alias).

**Parameters:**

- `name` (str): Language name or alias

**Returns:** bool - True if language is available

**Example:**

```python
from tree_sitter_language_pack import has_language

# Check availability
assert has_language("python")
assert has_language("shell")  # alias for "bash"
assert not has_language("nonexistent")
```text

### `language_count() -> int`

Return the number of available languages.

Includes statically compiled languages, dynamically loadable languages, and aliases.

**Returns:** int - Total number of languages

**Example:**

```python
from tree_sitter_language_pack import language_count

count = language_count()
print(f"{count} languages available")
```text

## Parsing

### `get_binding(name: str) -> TreeHandle`

Get the low-level tree-sitter binding for a language.

**Parameters:**

- `name` (str): Language name

**Returns:** TreeHandle - Low-level binding handle

**Raises:**

- `LanguageNotFoundError`: If language not found

**Example:**

```python
from tree_sitter_language_pack import get_binding

binding = get_binding("python")
# Use binding for advanced operations
```text

### `parse_string(source: str, language: str) -> Tree`

Parse source code string into a syntax tree.

Convenience function for quick parsing in a specific language.

**Parameters:**

- `source` (str): Source code to parse
- `language` (str): Language name

**Returns:** Tree - tree-sitter syntax tree

**Raises:**

- `LanguageNotFoundError`: If language not found
- `ParseError`: If parsing fails
- `DownloadError`: If auto-download fails

**Example:**

```python
from tree_sitter_language_pack import parse_string

tree = parse_string("def hello(): pass", "python")
print(tree.root_node.sexp())
```text

## Code Intelligence

### `process(source: str, config: ProcessConfig) -> dict`

Process source code and extract file intelligence.

Parses the source with tree-sitter and extracts metrics, structure, imports, exports, comments, docstrings, symbols, diagnostics, and/or chunks based on config flags.

**Parameters:**

- `source` (str): Source code to analyze
- `config` (ProcessConfig): Analysis configuration

**Returns:** dict - Result containing structure, imports, exports, comments, chunks, etc.

**Raises:**

- `LanguageNotFoundError`: If language not found
- `ParseError`: If parsing fails
- `ProcessError`: If analysis fails

**Example:**

```python
from tree_sitter_language_pack import process, ProcessConfig

# Extract all intelligence
config = ProcessConfig(language="python").all()
result = process("""
def hello(name):
    '''Say hello.'''
    print(f"Hello {name}")
""", config)

print(f"Functions: {len(result['structure'])}")
print(f"Docstrings: {len(result.get('docstrings', []))}")
print(f"Total lines: {result['metrics']['total_lines']}")
```text

## Types

### `ProcessConfig`

Configuration for the code intelligence analysis pipeline.

**Attributes:**

- `language` (str): Language name (required)
- `metrics` (bool): Extract file metrics (default: False)
- `structure` (bool): Extract code structure (default: False)
- `imports` (bool): Extract imports (default: False)
- `exports` (bool): Extract exports (default: False)
- `comments` (bool): Extract comments (default: False)
- `docstrings` (bool): Extract docstrings (default: False)
- `symbols` (bool): Extract symbols (default: False)
- `diagnostics` (bool): Extract diagnostics (default: False)
- `chunk_max_size` (int): Max size for chunks (default: 1024)
- `chunk_overlap` (int): Overlap between chunks (default: 200)

**Methods:**

#### `ProcessConfig(language: str)`

Create a new configuration for a language.

```python
from tree_sitter_language_pack import ProcessConfig

# Create config for Python
config = ProcessConfig(language="python")
```text

#### `.all()`

Enable all analysis flags.

```python
config = ProcessConfig(language="python").all()
# Enables: metrics, structure, imports, exports, comments, docstrings, symbols, diagnostics, chunks
```text

#### `.structure()`

Enable structure extraction only.

```python
config = ProcessConfig(language="python").structure()
```text

#### `.imports_exports()`

Enable imports/exports extraction.

```python
config = ProcessConfig(language="python").imports_exports()
```text

#### `.with_chunks(max_size: int, overlap: int)`

Configure code chunking.

```python
config = ProcessConfig(language="python").with_chunks(max_size=1000, overlap=200)
```text

**Example:**

```python
from tree_sitter_language_pack import ProcessConfig, process

# Configure analysis
config = (ProcessConfig(language="python")
    .structure()
    .imports_exports()
    .with_chunks(max_size=2000, overlap=400))

result = process("import os\ndef foo(): pass", config)
```text

### `TreeHandle`

Handle to a parsed syntax tree.

Provides access to tree-sitter Tree objects.

**Example:**

```python
from tree_sitter_language_pack import get_binding

handle = get_binding("python")
# Use handle to access tree-sitter functionality
```text

## Exceptions

### `DownloadError`

Raised when downloading languages fails.

```python
from tree_sitter_language_pack import download, DownloadError

try:
    download(["python"])
except DownloadError as e:
    print(f"Download failed: {e}")
```text

### `LanguageNotFoundError`

Raised when a language is not recognized.

```python
from tree_sitter_language_pack import get_language, LanguageNotFoundError

try:
    lang = get_language("nonexistent")
except LanguageNotFoundError as e:
    print(f"Language not found: {e}")
```text

### `ParseError`

Raised when parsing source code fails.

```python
from tree_sitter_language_pack import parse_string, ParseError

try:
    tree = parse_string(invalid_code, "python")
except ParseError as e:
    print(f"Parse error: {e}")
```text

### `QueryError`

Raised when query execution fails.

```python
from tree_sitter_language_pack import QueryError
```text

## Type Hints

All functions and methods are fully type-hinted for use with type checkers like `mypy`.

```python
from tree_sitter_language_pack import (
    init,
    download,
    get_parser,
    parse_string,
    process,
    ProcessConfig,
    DownloadError,
    LanguageNotFoundError,
    ParseError,
)

# All with proper type hints
def analyze_code(source: str, lang: str) -> dict:
    """Type-safe code analysis."""
    config: ProcessConfig = ProcessConfig(language=lang).all()
    result: dict = process(source, config)
    return result

try:
    result = analyze_code("def foo(): pass", "python")
except (DownloadError, LanguageNotFoundError, ParseError) as e:
    print(f"Error: {e}")
```
