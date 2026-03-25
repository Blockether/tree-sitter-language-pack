---
description: "Ruby API reference for tree-sitter-language-pack"
---

# Ruby API Reference

## Installation

Add to `Gemfile`:

```ruby
gem "tree_sitter_language_pack"
```text

Then run:

```bash
bundle install
```text

Or install directly:

```bash
gem install tree_sitter_language_pack
```text

## Quick Start

```ruby
require "tree_sitter_language_pack"

# Pre-download languages
TreeSitterLanguagePack.init(["python", "rust"])

# Get a language
language = TreeSitterLanguagePack.get_language("python")

# Get a pre-configured parser
parser = TreeSitterLanguagePack.get_parser("python")
tree = parser.parse("def hello(): pass")
puts tree.root_node.sexp

# Extract code intelligence
config = TreeSitterLanguagePack::ProcessConfig.new("python").all
result = TreeSitterLanguagePack.process("def hello(): pass", config)
puts "Functions: #{result["structure"].length}"
```text

## Download Management

### `TreeSitterLanguagePack.init(languages = nil, groups = nil, cache_dir = nil)`

Initialize the language pack with optional pre-downloads.

**Parameters:**

- `languages` (Array<String> | nil): Languages to download
- `groups` (Array<String> | nil): Language groups to download
- `cache_dir` (String | nil): Custom cache directory

**Returns:** nil

**Raises:**

- `DownloadError`: If downloads fail or network unavailable

**Example:**

```ruby
# Pre-download specific languages
TreeSitterLanguagePack.init(["python", "javascript", "rust"])

# Or download language groups
TreeSitterLanguagePack.init(groups: ["web", "data"])

# With custom cache directory
TreeSitterLanguagePack.init(
  languages: ["python"],
  cache_dir: "/opt/ts-pack"
)
```text

### `TreeSitterLanguagePack.configure(cache_dir = nil)`

Apply configuration without downloading.

Use to set custom cache directory before first `get_language` call.

**Parameters:**

- `cache_dir` (String | nil): Custom cache directory

**Returns:** nil

**Raises:**

- `DownloadError`: If lock cannot be acquired

**Example:**

```ruby
TreeSitterLanguagePack.configure(cache_dir: "/data/ts-pack")

language = TreeSitterLanguagePack.get_language("python")
```text

### `TreeSitterLanguagePack.download(names)`

Download specific languages to cache.

**Parameters:**

- `names` (Array<String>): Language names to download

**Returns:** Integer - Count of newly downloaded languages

**Raises:**

- `DownloadError`: If language not found or download fails
- `LanguageNotFoundError`: If language not in manifest

**Example:**

```ruby
count = TreeSitterLanguagePack.download(
  ["python", "rust", "typescript"]
)
puts "Downloaded #{count} new languages"
```text

### `TreeSitterLanguagePack.download_all`

Download all available languages (248).

**Returns:** Integer - Count of newly downloaded languages

**Raises:**

- `DownloadError`: If manifest fetch fails

**Example:**

```ruby
count = TreeSitterLanguagePack.download_all
puts "Downloaded #{count} languages total"
```text

### `TreeSitterLanguagePack.manifest_languages`

Get all available languages from remote manifest.

Fetches and caches the manifest.

**Returns:** Array<String> - Sorted language names

**Raises:**

- `DownloadError`: If manifest fetch fails

**Example:**

```ruby
languages = TreeSitterLanguagePack.manifest_languages
puts "Available: #{languages.length} languages"
```text

### `TreeSitterLanguagePack.downloaded_languages`

Get languages already cached locally.

Does not perform network requests.

**Returns:** Array<String> - Cached language names

**Example:**

```ruby
cached = TreeSitterLanguagePack.downloaded_languages
cached.each { |lang| puts lang }
```text

### `TreeSitterLanguagePack.clean_cache`

Delete all cached parser shared libraries.

**Returns:** nil

**Raises:**

- `DownloadError`: If cache cannot be removed

**Example:**

```ruby
TreeSitterLanguagePack.clean_cache
puts "Cache cleaned"
```text

### `TreeSitterLanguagePack.cache_dir`

Get the current cache directory path.

**Returns:** String - Absolute cache directory path

**Example:**

```ruby
dir = TreeSitterLanguagePack.cache_dir
puts "Cache at: #{dir}"
```text

## Language Discovery

### `TreeSitterLanguagePack.get_language(name)`

Get a tree-sitter Language by name.

Resolves aliases (e.g., `"shell"` → `"bash"`). Auto-downloads if needed.

**Parameters:**

- `name` (String): Language name or alias

**Returns:** Language - tree-sitter Language object

**Raises:**

- `LanguageNotFoundError`: If language not recognized
- `DownloadError`: If auto-download fails

**Example:**

```ruby
language = TreeSitterLanguagePack.get_language("python")

parser = TreeSitter::Parser.new
parser.set_language(language)
tree = parser.parse("x = 1")
puts tree.root_node.type # "module"
```text

### `TreeSitterLanguagePack.get_parser(name)`

Get a pre-configured Parser for a language.

**Parameters:**

- `name` (String): Language name or alias

**Returns:** Parser - Pre-configured tree-sitter Parser

**Raises:**

- `LanguageNotFoundError`: If language not recognized
- `DownloadError`: If auto-download fails
- `ParserError`: If parser setup fails

**Example:**

```ruby
parser = TreeSitterLanguagePack.get_parser("rust")
tree = parser.parse("fn main() {}")
puts tree.root_node.has_error? # false
```text

### `TreeSitterLanguagePack.available_languages`

List all available language names.

**Returns:** Array<String> - Sorted language names

**Example:**

```ruby
langs = TreeSitterLanguagePack.available_languages
langs.each { |lang| puts lang }
```text

### `TreeSitterLanguagePack.has_language?(name)`

Check if a language is available.

**Parameters:**

- `name` (String): Language name or alias

**Returns:** Boolean - True if available

**Example:**

```ruby
if TreeSitterLanguagePack.has_language?("python")
  puts "Python available"
end

raise "Shell not available" unless TreeSitterLanguagePack.has_language?("shell")
```text

### `TreeSitterLanguagePack.language_count`

Get total number of available languages.

**Returns:** Integer - Language count

**Example:**

```ruby
count = TreeSitterLanguagePack.language_count
puts "#{count} languages available"
```text

## Parsing

### `TreeSitterLanguagePack.parse_string(source, language)`

Parse source code into a syntax tree.

**Parameters:**

- `source` (String): Source code
- `language` (String): Language name

**Returns:** Tree - Parsed syntax tree

**Raises:**

- `LanguageNotFoundError`: If language not found
- `ParseError`: If parsing fails
- `DownloadError`: If auto-download fails

**Example:**

```ruby
tree = TreeSitterLanguagePack.parse_string(
  "def foo(): pass",
  "python"
)
puts tree.root_node.sexp
```text

## Code Intelligence

### `TreeSitterLanguagePack.process(source, config)`

Extract code intelligence from source code.

**Parameters:**

- `source` (String): Source code
- `config` (ProcessConfig): Configuration

**Returns:** Hash - Result with structure, imports, exports, etc.

**Raises:**

- `LanguageNotFoundError`: If language not found
- `ParseError`: If parsing fails
- `ProcessError`: If analysis fails

**Example:**

```ruby
config = TreeSitterLanguagePack::ProcessConfig.new("python")
  .structure
  .import_exports
  .with_chunks(2000, 400)

result = TreeSitterLanguagePack.process(
  "def hello(): pass",
  config
)

puts "Functions: #{result["structure"].length}"
puts "Lines: #{result["metrics"]["total_lines"]}"
```text

## Types

### `ProcessConfig`

Configuration for code intelligence analysis.

**Constructor:**

```ruby
config = TreeSitterLanguagePack::ProcessConfig.new("python")
```text

**Methods:**

#### `#structure`

Enable structure extraction.

#### `#import_exports`

Enable imports/exports extraction.

#### `#comments`

Enable comment extraction.

#### `#docstrings`

Enable docstring extraction.

#### `#symbols`

Enable symbol extraction.

#### `#metrics`

Enable metric extraction.

#### `#diagnostics`

Enable diagnostic extraction.

#### `#with_chunks(max_size, overlap)`

Configure code chunking.

#### `#all`

Enable all features.

**Example:**

```ruby
config = TreeSitterLanguagePack::ProcessConfig.new("python")
  .structure
  .import_exports
  .comments
  .with_chunks(2000, 400)
```text

### Result Hash

Result from `process` method.

**Keys:**

- `"language"` (String) - Language name
- `"metrics"` (Hash) - File metrics
    - `"total_lines"` (Integer)
    - `"code_lines"` (Integer)
    - `"comment_lines"` (Integer)
    - `"blank_lines"` (Integer)
- `"structure"` (Array) - Code structure items
    - Each item has `"kind"`, `"name"`, `"line"`, `"column"`, etc.
- `"imports"` (Array) - Import statements
- `"exports"` (Array) - Export statements
- `"comments"` (Array) - Comments
- `"docstrings"` (Array) - Docstrings
- `"symbols"` (Array) - Symbols
- `"diagnostics"` (Array) - Diagnostics
- `"chunks"` (Array) - Code chunks
- `"parse_errors"` (Integer) - Number of parse errors

**Example:**

```ruby
result = TreeSitterLanguagePack.process(source, config)

puts result["language"]
result["structure"].each do |item|
  puts "  #{item["kind"]}: #{item["name"]}"
end
```text

## Exception Handling

```ruby
require "tree_sitter_language_pack"

begin
  language = TreeSitterLanguagePack.get_language("python")
  parser = TreeSitter::Parser.new
  parser.set_language(language)
  tree = parser.parse("x = 1")
rescue TreeSitterLanguagePack::LanguageNotFoundError => e
  puts "Language not found: #{e.message}"
rescue TreeSitterLanguagePack::DownloadError => e
  puts "Download failed: #{e.message}"
rescue TreeSitterLanguagePack::ParseError => e
  puts "Parse error: #{e.message}"
rescue => e
  puts "Unexpected error: #{e.message}"
end
```text

## Usage Patterns

### Pre-download Languages

```ruby
# config/initializers/tree_sitter.rb
TreeSitterLanguagePack.init(
  languages: %w[python rust typescript javascript]
)
```text

Then use in your application:

```ruby
require "tree_sitter_language_pack"

# Fast, no network required
parser = TreeSitterLanguagePack.get_parser("python")
```text

### Custom Cache Directory

```ruby
TreeSitterLanguagePack.configure(
  cache_dir: "/data/ts-pack-cache"
)

language = TreeSitterLanguagePack.get_language("python")
```text

### Batch Processing

```ruby
def analyze_files(dir, language)
  Dir.glob("#{dir}/**/*.#{language}").each do |file|
    begin
      source = File.read(file)
      config = TreeSitterLanguagePack::ProcessConfig.new(language).all
      result = TreeSitterLanguagePack.process(source, config)

      puts "#{file}: #{result["structure"].length} items"
    rescue => e
      puts "Error: #{e.message}"
    end
  end
end

analyze_files("./src", "py")
```text

### Parse and Walk Tree

```ruby
parser = TreeSitterLanguagePack.get_parser("python")
tree = parser.parse("def hello(): pass")

def walk_tree(node, depth = 0)
  indent = "  " * depth
  puts "#{indent}#{node.type}"

  node.children.each { |child| walk_tree(child, depth + 1) }
end

walk_tree(tree.root_node)
```text

### Extract Specific Patterns

```ruby
config = TreeSitterLanguagePack::ProcessConfig.new("python")
  .structure

result = TreeSitterLanguagePack.process(File.read("code.py"), config)

# Find all functions
functions = result["structure"].select { |item| item["kind"] == "function" }
functions.each do |func|
  puts func["name"]
end
```text

### Concurrent Processing (with Mutex)

```ruby
require "concurrent"

parser_pool = Concurrent::Array.new

def get_or_create_parser(pool, language)
  # Ensure thread safety
  pool.find { |p| p.language == language } ||
    pool << TreeSitterLanguagePack.get_parser(language)
end

# Use pool in threads
(1..10).map do |i|
  Thread.new do
    parser = get_or_create_parser(parser_pool, "python")
    source = File.read("file#{i}.py")
    tree = parser.parse(source)
    puts "Parsed file #{i}"
  end
end.each(&:join)
```
