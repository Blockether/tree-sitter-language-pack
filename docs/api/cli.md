---
description: "CLI command reference for tree-sitter-language-pack"
---

# CLI Command Reference

## Installation

### From Source

```bash
git clone https://github.com/kreuzberg-dev/tree-sitter-language-pack
cd tree-sitter-language-pack
cargo install --path crates/ts-pack-cli
```

## Global Options

All commands support these options:

- `--help` / `-h` -- Show help message
- `--version` / `-V` -- Show version

## Commands

### `ts-pack download`

Download parser libraries to the local cache.

**Usage:**

```bash
ts-pack download [OPTIONS] [LANGUAGES]...
```

**Arguments:**

- `[LANGUAGES]...` -- Language names to download (space-separated). If omitted, looks for a `language-pack.toml` config file.

**Options:**

- `--all` -- Download all available languages
- `--groups <GROUPS>` -- Download language groups (comma-separated values: web, systems, scripting, data, jvm, functional)
- `--fresh` -- Clean cache before downloading (fresh download)

**Examples:**

```bash
# Download specific languages
ts-pack download python rust typescript

# Download all languages
ts-pack download --all

# Download language groups
ts-pack download --groups web,data

# Fresh download (clears cache first)
ts-pack download --fresh python rust

# Use discovered language-pack.toml config
ts-pack download
```

### `ts-pack clean`

Remove all cached parser libraries.

**Usage:**

```bash
ts-pack clean [OPTIONS]
```

**Options:**

- `--force` -- Skip the confirmation prompt

**Examples:**

```bash
# Clean with confirmation prompt
ts-pack clean

# Clean without confirmation
ts-pack clean --force
```

### `ts-pack list`

List available languages.

**Usage:**

```bash
ts-pack list [OPTIONS]
```

**Options:**

- `--downloaded` -- Show only downloaded/cached languages
- `--manifest` -- Show all languages from the remote manifest
- `--filter <SUBSTRING>` -- Filter languages by substring match

By default (no flags), lists languages from the remote manifest.

**Examples:**

```bash
# List all available languages (from manifest)
ts-pack list

# List only downloaded languages
ts-pack list --downloaded

# Filter by name
ts-pack list --filter python
```

### `ts-pack info`

Show details about a specific language.

**Usage:**

```bash
ts-pack info <LANGUAGE>
```

**Arguments:**

- `<LANGUAGE>` -- Language name

**Output fields:**

- Language name
- Whether the language is known (compiled-in or in manifest)
- Whether it is downloaded
- Cache path (if downloaded) or cache directory

**Example:**

```bash
ts-pack info python
```

**Output:**

```text
Language:    python
Known:       true
Downloaded:  true
Cache path:  /home/user/.cache/ts-pack/libtree_sitter_python.so
```

### `ts-pack parse`

Parse a file and output the syntax tree.

**Usage:**

```bash
ts-pack parse [OPTIONS] <FILE>
```

**Arguments:**

- `<FILE>` -- Source file to parse. Use `-` for stdin.

**Options:**

- `--language <LANG>` / `-l <LANG>` -- Language name. Auto-detected from extension if omitted.
- `--format <FORMAT>` / `-f <FORMAT>` -- Output format: `sexp` (default) or `json`.

**Examples:**

```bash
# Parse file with auto-detected language
ts-pack parse code.py

# Parse with explicit language
ts-pack parse code.py --language python

# Output as JSON
ts-pack parse code.py --format json

# Parse from stdin
echo "x = 1" | ts-pack parse - --language python
```

**JSON output format:**

```json
{
  "language": "python",
  "sexp": "(module (expression_statement (assignment ...)))",
  "has_errors": false
}
```

### `ts-pack process`

Run the code intelligence pipeline on a file.

**Usage:**

```bash
ts-pack process [OPTIONS] <FILE>
```

**Arguments:**

- `<FILE>` -- Source file to process. Use `-` for stdin.

**Options:**

- `--language <LANG>` / `-l <LANG>` -- Language name. Auto-detected from extension if omitted (required for stdin).
- `--all` -- Enable all analysis features
- `--structure` -- Extract code structure (functions, classes)
- `--imports` -- Extract imports
- `--exports` -- Extract exports
- `--comments` -- Extract comments
- `--symbols` -- Extract symbols
- `--docstrings` -- Extract docstrings
- `--diagnostics` -- Include diagnostics
- `--chunk-size <BYTES>` -- Maximum chunk size in bytes

When no feature flags are given, defaults apply (structure, imports, and exports enabled).

**Examples:**

```bash
# Process with all features
ts-pack process code.py --all

# Extract only structure
ts-pack process code.py --structure

# Extract with chunking
ts-pack process code.py --all --chunk-size 1000

# Process from stdin
cat code.py | ts-pack process - --language python --structure
```

Output is always JSON printed to stdout.

### `ts-pack cache-dir`

Print the effective cache directory path.

**Usage:**

```bash
ts-pack cache-dir
```

**Example:**

```bash
ts-pack cache-dir
# /home/user/.cache/ts-pack
```

### `ts-pack init`

Create a `language-pack.toml` configuration file in the current directory.

**Usage:**

```bash
ts-pack init [OPTIONS]
```

**Options:**

- `--cache-dir <PATH>` -- Set cache directory in the config
- `--languages <LANGS>` -- Languages to include (comma-separated)

If languages are specified, they are also downloaded immediately.

**Examples:**

```bash
# Create a config with specific languages
ts-pack init --languages python,rust,typescript

# Create with custom cache directory
ts-pack init --cache-dir /opt/ts-pack --languages python

# Create a blank template
ts-pack init
```

**Generated file (`language-pack.toml`):**

```toml
languages = ["python", "rust", "typescript"]
```

### `ts-pack completions`

Generate shell completions for the given shell.

**Usage:**

```bash
ts-pack completions <SHELL>
```

**Arguments:**

- `<SHELL>` -- Shell to generate completions for: `bash`, `zsh`, `fish`, `elvish`, `powershell`

**Examples:**

```bash
# Generate Bash completions
ts-pack completions bash > ~/.local/share/bash-completion/completions/ts-pack

# Generate Zsh completions
ts-pack completions zsh > ~/.zfunc/_ts-pack

# Generate Fish completions
ts-pack completions fish > ~/.config/fish/completions/ts-pack.fish
```

## Exit Codes

- `0` -- Success
- `1` -- Error (parse failure, missing file, invalid language, network error, etc.)

All error messages are printed to stderr.

## Extraction Queries

Extraction queries are available through the library API but are not yet exposed as a CLI subcommand. See the [Extraction Queries guide](../guides/extraction.md) for usage via the Rust, Python, TypeScript, and C FFI APIs.

## Common Workflows

### Download and Parse

```bash
ts-pack download python
ts-pack parse code.py
```

### Set Up a Project

```bash
ts-pack init --languages python,rust,typescript
ts-pack download
```

### Process Multiple Files

```bash
ts-pack download python
for file in src/**/*.py; do
    ts-pack process "$file" --structure --imports
done
```
