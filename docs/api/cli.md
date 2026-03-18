---
description: "CLI command reference for tree-sitter-language-pack"
---

# CLI Command Reference

## Installation

### From Homebrew

```bash
brew install kreuzberg-dev/tap/ts-pack
```text

### From Source

```bash
git clone https://github.com/kreuzberg-dev/tree-sitter-language-pack
cd tree-sitter-language-pack
cargo install --path crates/ts-pack-cli
```text

## Global Options

All commands support these options:

- `--help` / `-h` - Show help message
- `--version` / `-V` - Show version
- `--cache-dir <PATH>` - Custom cache directory (default: platform-specific)
- `--verbose` / `-v` - Enable verbose output
- `--quiet` / `-q` - Suppress output

**Example:**

```bash
ts-pack --cache-dir /opt/ts-pack download python
ts-pack -v get-language python
```text

## Commands

### `ts-pack download`

Download specific languages to cache.

**Usage:**

```bash
ts-pack download [OPTIONS] <LANGUAGES>...
```text

**Arguments:**

- `<LANGUAGES>...` - Language names to download (space-separated)

**Options:**

- `--all` - Download all 170+ languages
- `--groups <GROUPS>...` - Download language groups instead
- `--force` - Re-download even if cached
- `--verbose` - Show download progress

**Examples:**

```bash
# Download specific languages
ts-pack download python rust typescript

# Download all languages
ts-pack download --all

# Download language groups
ts-pack download --groups web data

# Download with progress
ts-pack download -v python javascript
```text

### `ts-pack list`

List available or downloaded languages.

**Usage:**

```bash
ts-pack list [OPTIONS]
```text

**Options:**

- `--all` / `-a` - Show all available languages (default: cached only)
- `--groups` - Show available language groups
- `--count` - Show count only
- `--json` - Output as JSON
- `--filter <PATTERN>` - Filter by name pattern

**Examples:**

```bash
# List cached languages
ts-pack list

# List all available
ts-pack list --all

# Count cached languages
ts-pack list --count

# Show as JSON
ts-pack list --json

# Filter languages
ts-pack list --all --filter ".*script"
```text

**Output:**

```text
Available languages:
  python
  rust
  typescript
  ...

Cached: 15 of 170 languages
```text

### `ts-pack parse`

Parse source code and display syntax tree.

**Usage:**

```bash
ts-pack parse [OPTIONS] <FILE> <LANGUAGE>
```text

**Arguments:**

- `<FILE>` - Source code file to parse (use `-` for stdin)
- `<LANGUAGE>` - Language name

**Options:**

- `--sexp` - Output S-expression (default)
- `--json` - Output as JSON
- `--dot` - Output as Graphviz DOT format
- `--pretty` - Pretty-print output
- `--depth <N>` - Maximum tree depth to display
- `--first-error` - Stop at first parse error

**Examples:**

```bash
# Parse file
ts-pack parse code.py python

# Parse from stdin
echo "x = 1" | ts-pack parse - python

# Output as JSON
ts-pack parse code.py python --json

# Output as Graphviz
ts-pack parse code.py python --dot | dot -Tpng -o tree.png

# Show only first 3 levels
ts-pack parse code.py python --depth 3
```text

### `ts-pack analyze`

Extract code intelligence from source code.

**Usage:**

```bash
ts-pack analyze [OPTIONS] <FILE> <LANGUAGE>
```text

**Arguments:**

- `<FILE>` - Source code file to analyze (use `-` for stdin)
- `<LANGUAGE>` - Language name

**Options:**

- `--structure` - Extract code structure (functions, classes, etc.)
- `--imports` - Extract imports
- `--exports` - Extract exports
- `--comments` - Extract comments
- `--docstrings` - Extract docstrings
- `--symbols` - Extract symbols
- `--diagnostics` - Extract diagnostics
- `--metrics` - Show file metrics
- `--chunks <SIZE>` - Enable code chunking with max size
- `--chunk-overlap <SIZE>` - Chunk overlap size (default: 200)
- `--all` - Enable all features
- `--json` - Output as JSON
- `--pretty` - Pretty-print output

**Examples:**

```bash
# Extract everything
ts-pack analyze code.py python --all

# Extract structure only
ts-pack analyze code.py python --structure

# Extract with chunking
ts-pack analyze code.py python --all --chunks 1000 --chunk-overlap 200

# Output as JSON
ts-pack analyze code.py python --all --json

# Analyze from stdin
cat code.py | ts-pack analyze - python --structure
```text

**Output (default):**

```text
File: code.py
Language: python
Metrics:
  Total lines: 42
  Code lines: 38
  Comment lines: 2
  Blank lines: 2

Structure:
  function: hello (line 1)
    - parameter: name
    - parameter: age
  class: Person (line 8)
    - method: __init__ (line 9)
    - method: greet (line 12)

Imports:
  os
  sys
  typing.Optional

Exports:
  hello
  Person
```text

### `ts-pack check`

Check syntax of source code.

**Usage:**

```bash
ts-pack check [OPTIONS] <FILE> <LANGUAGE>
```text

**Arguments:**

- `<FILE>` - Source file to check
- `<LANGUAGE>` - Language name

**Options:**

- `--strict` - Fail on warnings
- `--json` - Output as JSON
- `--explain` - Show detailed error explanations

**Examples:**

```bash
# Check syntax
ts-pack check code.py python

# Check with detailed output
ts-pack check code.py python --explain

# Output as JSON
ts-pack check code.py python --json
```text

**Output:**

```text
✓ code.py (syntax OK)
  No errors found
```text

Or on error:

```text
✗ code.py (syntax errors found)
  Line 5, col 3: Unexpected EOF in function definition
```text

### `ts-pack info`

Show information about a language.

**Usage:**

```bash
ts-pack info [OPTIONS] <LANGUAGE>
```text

**Arguments:**

- `<LANGUAGE>` - Language name

**Options:**

- `--json` - Output as JSON
- `--stats` - Show statistics

**Examples:**

```bash
# Get language info
ts-pack info python

# Show as JSON
ts-pack info python --json

# Show statistics
ts-pack info python --stats
```text

**Output:**

```text
Language: python
Aliases: py, python3

Status: ready
Cached: yes
Path: /home/user/.cache/ts-pack/python.so
Version: 1.0.0

Features:
  ✓ Parsing
  ✓ Structure extraction
  ✓ Import/export detection
  ✓ Comment extraction
  ✓ Docstring extraction
  ✓ Symbol extraction
```text

### `ts-pack config`

Show or modify configuration.

**Usage:**

```bash
ts-pack config [OPTIONS] [KEY] [VALUE]
```text

**Arguments:**

- `[KEY]` - Configuration key to get/set
- `[VALUE]` - Value to set (if omitted, shows current value)

**Options:**

- `--get <KEY>` - Get config value
- `--set <KEY> <VALUE>` - Set config value
- `--list` - List all configuration
- `--reset` - Reset to defaults
- `--json` - Output as JSON

**Examples:**

```bash
# List configuration
ts-pack config --list

# Get cache directory
ts-pack config --get cache-dir

# Set cache directory
ts-pack config --set cache-dir /opt/ts-pack

# Show as JSON
ts-pack config --list --json

# Reset to defaults
ts-pack config --reset
```text

### `ts-pack clean`

Clean cache directory.

**Usage:**

```bash
ts-pack clean [OPTIONS]
```text

**Options:**

- `--all` - Delete entire cache directory
- `--dry-run` - Show what would be deleted
- `--confirm` / `-y` - Skip confirmation prompt
- `--language <NAME>` - Delete specific language cache only

**Examples:**

```bash
# Remove unused cache files
ts-pack clean

# Preview cleanup
ts-pack clean --dry-run

# Delete everything
ts-pack clean --all --confirm

# Remove specific language
ts-pack clean --language python
```text

**Output:**

```text
Cleaning cache...
  Removed 5 unused parsers (42 MB)
Cache size: 128 MB
```text

### `ts-pack cache`

Show cache information.

**Usage:**

```bash
ts-pack cache [OPTIONS]
```text

**Options:**

- `--path` - Show cache directory path
- `--size` - Show cache size
- `--breakdown` - Show size per language
- `--json` - Output as JSON

**Examples:**

```bash
# Show cache info
ts-pack cache

# Show path only
ts-pack cache --path

# Show size breakdown
ts-pack cache --breakdown --json
```text

**Output:**

```text
Cache Information
Location: /home/user/.cache/ts-pack

Total size: 256 MB

Breakdown:
  python:     45 MB (20%)
  rust:       38 MB (15%)
  typescript: 32 MB (12%)
  ... (17 more languages)

Cached languages: 20/170
```text

### `ts-pack version`

Show version information.

**Usage:**

```bash
ts-pack version [OPTIONS]
```text

**Options:**

- `--json` - Output as JSON
- `--check` - Check for updates

**Examples:**

```bash
# Show version
ts-pack version

# Check for updates
ts-pack version --check

# Output as JSON
ts-pack version --json
```text

**Output:**

```text
ts-pack version 1.0.0 (tree-sitter-language-pack)
  Built: 2026-03-18
  Grammars: 170+
```text

### `ts-pack help`

Show help information.

**Usage:**

```bash
ts-pack help [COMMAND]
```text

**Arguments:**

- `[COMMAND]` - Command to get help for (optional)

**Examples:**

```bash
# General help
ts-pack help

# Command help
ts-pack help download

# Also works with --help
ts-pack download --help
```text

## Common Workflows

### Download and Parse

```bash
# Download language
ts-pack download python

# Parse file
ts-pack parse code.py python --sexp
```text

### Analyze Project

```bash
# Download languages needed for project
ts-pack download python rust typescript

# Analyze Python files
for file in src/**/*.py; do
  ts-pack analyze "$file" python --structure
done

# Analyze Rust files
for file in src/**/*.rs; do
  ts-pack analyze "$file" rust --structure
done
```text

### Check Multiple Files

```bash
# Check all Python files
for file in src/**/*.py; do
  ts-pack check "$file" python || exit 1
done
echo "All files valid"
```text

### Export Tree Structure

```bash
# Export as JSON
ts-pack parse code.py python --json > tree.json

# Export as Graphviz
ts-pack parse code.py python --dot | dot -Tpng -o tree.png

# Export as S-expression
ts-pack parse code.py python --sexp > tree.sexp
```text

### Batch Operations

```bash
# Download all web languages
ts-pack download --groups web

# Analyze all files
find . -name "*.py" -exec ts-pack analyze {} python --all \;

# Generate reports
for lang in python rust typescript; do
  ts-pack list --filter "$lang" --json > "$lang-info.json"
done
```text

## Exit Codes

- `0` - Success
- `1` - Command error (file not found, invalid language, etc.)
- `2` - Parse error (syntax error in source)
- `3` - Download error (network issue, permission denied)
- `4` - Configuration error
- `5` - Cache error

## Environment Variables

- `TS_PACK_CACHE` - Override cache directory
- `TS_PACK_VERBOSE` - Enable verbose output (set to `1`)
- `TS_PACK_NO_COLOR` - Disable colored output

**Example:**

```bash
export TS_PACK_CACHE=/opt/ts-pack
export TS_PACK_VERBOSE=1
ts-pack list --all
```text

## Configuration File

Configuration stored in platform-specific locations:

- **Linux/macOS**: `~/.config/ts-pack/config.toml`
- **Windows**: `%APPDATA%\ts-pack\config.toml`

**Example:**

```toml
[cache]
directory = "/opt/ts-pack"

[features]
auto-download = true
check-updates = false

[languages]
default = "python"
```text

## Troubleshooting

### Clear cache and restart

```bash
ts-pack clean --all --confirm
ts-pack download python
ts-pack parse code.py python
```text

### Check configuration

```bash
ts-pack config --list
ts-pack cache --path
```text

### Verbose output for debugging

```bash
ts-pack -v download python 2>&1 | less
ts-pack -v analyze code.py python --all 2>&1 | less
```text

### Check language availability

```bash
ts-pack list --all --filter "python"
ts-pack info python
```
