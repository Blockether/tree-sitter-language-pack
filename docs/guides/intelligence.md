---
description: "Code intelligence extraction — structure, imports, exports, symbols, docstrings, diagnostics, and metrics from source code."
---

# Code Intelligence Guide

Beyond raw syntax trees, tree-sitter-language-pack can extract **semantic information** useful for code analysis, search, documentation generation, and LLM ingestion. This guide covers the `process()` function, `ProcessConfig` options, and working with intelligence results.

## Overview

The `process()` function runs tree-sitter queries against a parsed AST to extract structured data about code. Enable only the features you need:

- **structure**: Functions, classes, methods, interfaces, and other declarations
- **imports**: Import statements and their sources
- **exports**: Exported symbols and public API surface
- **comments**: Inline and block comments
- **docstrings**: Documentation strings attached to declarations
- **symbols**: All identifiers in the code (for search indexing)
- **diagnostics**: Syntax errors and malformed code regions
- **chunks**: Syntax-aware splitting for LLM token budgets (see [Chunking](chunking.md))
- **metrics**: File-level statistics (line count, complexity, etc.)

## Quick Start

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    source = """
    def greet(name: str) -> str:
        '''Say hello to a user.'''
        return f"Hello {name}"

    class Greeter:
        def __init__(self, prefix: str = ""):
            self.prefix = prefix
    """

    config = ProcessConfig(
        language="python",
        structure=True,      # extract classes, functions, methods
        docstrings=True,     # attach docstrings to declarations
        metrics=True,        # file statistics
    )
    result = process(source, config)

    # Access results
    for item in result["structure"]:
        print(f"{item['kind']:10} {item['name']:20} lines {item['start_line']}-{item['end_line']}")

    print(f"\nFile metrics: {result['metrics']['total_lines']} lines, "
          f"{result['metrics']['code_lines']} code")
    ```

=== "Node.js"

    ```typescript
    import { process } from "@kreuzberg/tree-sitter-language-pack";

    const source = `
    function greet(name: string): string {
      return \`Hello \${name}\`;
    }

    class Greeter {
      constructor(public prefix: string = "") {}
    }
    `;

    const result = await process(source, {
      language: "typescript",
      structure: true,
      docstrings: true,
      metrics: true,
    });

    // Access results
    result.structure.forEach(item => {
      console.log(`${item.kind.padEnd(10)} ${item.name.padEnd(20)} lines ${item.startLine}-${item.endLine}`);
    });

    console.log(`\nFile metrics: ${result.metrics.totalLines} lines, ${result.metrics.codeLines} code`);
    ```

=== "Rust"

    ```rust
    use ts_pack_core::{process, ProcessConfig};

    let source = r#"
    /// Greet a user.
    pub fn greet(name: &str) -> String {
        format!("Hello {}", name)
    }

    pub struct Greeter {
        prefix: String,
    }
    "#;

    let config = ProcessConfig::new("rust")
        .structure(true)
        .docstrings(true)
        .metrics(true);

    let result = process(source, &config)?;

    // Access results
    for item in &result.structure {
        println!("{:10} {:20} lines {}-{}",
            item.kind, item.name, item.start_line, item.end_line);
    }

    println!("\nFile metrics: {} lines, {} code",
        result.metrics.total_lines, result.metrics.code_lines);
    ```

=== "CLI"

    ```bash
    # Extract structure and docstrings
    ts-pack process src/app.py --structure --docstrings

    # Enable all features and output as JSON
    ts-pack process src/app.py --all --format json | jq '.structure'

    # Count functions
    ts-pack process src/lib.rs --structure --format json | jq '.structure | map(select(.kind == "function")) | length'
    ```

## ProcessConfig Reference

### Creating Configuration

=== "Python"

    ```python
    from tree_sitter_language_pack import ProcessConfig

    # Create config for a language
    config = ProcessConfig(language="python")

    # Enable specific features
    config = ProcessConfig(
        language="python",
        structure=True,       # functions, classes, methods
        imports=True,         # import statements
        exports=True,         # exported symbols
        comments=True,        # inline comments
        docstrings=True,      # docstrings attached to declarations
        symbols=True,         # all identifiers
        diagnostics=True,     # syntax errors
        metrics=True,         # file statistics
        chunk_max_size=0,     # 0 = no chunking
        chunk_overlap=0,      # tokens to overlap between chunks
    )

    # Or enable everything at once
    config = ProcessConfig(language="python").all()
    ```

=== "Node.js"

    ```typescript
    import { process } from "@kreuzberg/tree-sitter-language-pack";

    const result = await process(source, {
      language: "typescript",
      structure: true,
      imports: true,
      exports: true,
      comments: true,
      docstrings: true,
      symbols: true,
      diagnostics: true,
      metrics: true,
      chunkMaxSize: 0,      // 0 = no chunking
      chunkOverlap: 0,      // tokens to overlap
    });
    ```

=== "Rust"

    ```rust
    use ts_pack_core::{ProcessConfig, process};

    let config = ProcessConfig::new("rust")
        .structure(true)
        .imports(true)
        .exports(true)
        .comments(true)
        .docstrings(true)
        .symbols(true)
        .diagnostics(true)
        .metrics(true)
        .all();  // Enable all at once

    let result = process(source, &config)?;
    ```

## ProcessResult Fields

### `structure` — Code Declarations

List of top-level and nested code constructs: functions, classes, methods, interfaces, traits, etc.

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    source = """
    def process_data(input_file: str):
        '''Read and process data from a file.'''
        data = load_file(input_file)
        return transform(data)

    class DataProcessor:
        '''A class for processing data.'''

        def __init__(self, name: str):
            self.name = name

        def process(self, data):
            '''Process data and return result.'''
            return data
    """

    config = ProcessConfig(language="python", structure=True, docstrings=True)
    result = process(source, config)

    for item in result["structure"]:
        print(f"Kind:       {item['kind']}")           # "function", "class", "method"
        print(f"Name:       {item['name']}")           # "process_data", "DataProcessor"
        print(f"Start line: {item['start_line']}")     # 1-indexed
        print(f"End line:   {item['end_line']}")
        print(f"Docstring:  {item.get('docstring', '(none)')}")
        print()
    ```

=== "Node.js"

    ```typescript
    import { process } from "@kreuzberg/tree-sitter-language-pack";

    const source = `
    function processData(inputFile: string): any {
      const data = loadFile(inputFile);
      return transform(data);
    }

    class DataProcessor {
      name: string;

      constructor(name: string) {
        this.name = name;
      }

      process(data: any): any {
        return data;
      }
    }
    `;

    const result = await process(source, {
      language: "typescript",
      structure: true,
      docstrings: true,
    });

    result.structure.forEach(item => {
      console.log(`Kind:       ${item.kind}`);             // "function", "class", "method"
      console.log(`Name:       ${item.name}`);             // "processData", "DataProcessor"
      console.log(`Start line: ${item.startLine}`);
      console.log(`End line:   ${item.endLine}`);
      console.log(`Docstring:  ${item.docstring || "(none)"}`);
      console.log();
    });
    ```

**Structure item fields:**

| Field | Type | Description |
|-------|------|-------------|
| `kind` | string | One of: `function`, `class`, `method`, `interface`, `struct`, `trait`, `enum`, `module`, etc. |
| `name` | string | Name of the declaration |
| `start_line` | int | First line (1-indexed) |
| `end_line` | int | Last line (1-indexed) |
| `docstring` | string \| null | Docstring if `docstrings=True` |

**Supported kinds by language:**

| Language | Kinds |
|----------|-------|
| Python | `function`, `class`, `method`, `async_function` |
| JavaScript/TypeScript | `function`, `class`, `method`, `async_function`, `interface`, `enum` |
| Rust | `function`, `struct`, `impl`, `trait`, `enum`, `type_alias`, `mod` |
| Java | `class`, `interface`, `method`, `constructor`, `enum` |
| Go | `function`, `struct`, `interface`, `method` |
| Ruby | `def`, `class`, `module`, `singleton_method` |

### `imports` — Import Statements

All import and require declarations with their source modules.

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    source = """
    import os
    from pathlib import Path
    from typing import Optional, Dict
    from .utils import helper
    """

    config = ProcessConfig(language="python", imports=True)
    result = process(source, config)

    for imp in result["imports"]:
        print(f"Source: {imp['source']}")          # "os", "pathlib", "typing", "./utils"
        print(f"Names:  {imp['names']}")           # ["Path"], ["Optional", "Dict"], ["helper"]
        print(f"Line:   {imp['start_line']}")
        print()
    ```

Output:

```text
Source: os
Names:  []

Source: pathlib
Names:  ['Path']

Source: typing
Names:  ['Optional', 'Dict']

Source: ./utils
Names:  ['helper']
```text

=== "Node.js"

    ```typescript
    import { process } from "@kreuzberg/tree-sitter-language-pack";

    const source = `
    import os from "os";
    import { readFile, writeFile } from "fs";
    import * as path from "path";
    import utils from "./utils.js";
    `;

    const result = await process(source, {
      language: "javascript",
      imports: true,
    });

    result.imports.forEach(imp => {
      console.log(`Source: ${imp.source}`);
      console.log(`Names:  ${imp.names.join(", ") || "(all)"}`);
      console.log();
    });
    ```

**Import item fields:**

| Field | Type | Description |
|-------|------|-------------|
| `source` | string | Module path or name |
| `names` | list[string] | Imported identifiers (empty = wildcard or bare import) |
| `start_line` | int | Line where import appears |

### `exports` — Exported Symbols

Symbols that are part of the module's public API.

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    source = """
    def public_function():
        pass

    def _private_function():
        pass

    class PublicClass:
        pass

    __all__ = ["public_function", "PublicClass"]
    """

    config = ProcessConfig(language="python", exports=True)
    result = process(source, config)

    for exp in result["exports"]:
        print(f"Name: {exp['name']}")        # "public_function", "PublicClass"
        print(f"Kind: {exp['kind']}")        # "function", "class"
        print()
    ```

=== "JavaScript/TypeScript"

    ```typescript
    import { process } from "@kreuzberg/tree-sitter-language-pack";

    const source = `
    export function helper() {}
    export const API_KEY = "secret";
    export class Logger {}
    function internal() {}
    `;

    const result = await process(source, {
      language: "typescript",
      exports: true,
    });

    result.exports.forEach(exp => {
      console.log(`${exp.kind.padEnd(10)} ${exp.name}`);
    });
    // Output:
    // function   helper
    // const      API_KEY
    // class      Logger
    ```

!!! note "Language Differences"
    Export detection varies:
    - **Python**: Module-level items not prefixed with `_`, or listed in `__all__`
    - **JavaScript/TypeScript**: Only explicit `export` declarations
    - **Rust**: Public items with `pub` visibility

### `comments` — Comments

All comments in the source, with their text and location.

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    source = """
    # Top-level comment
    def process():
        # Inline comment
        return 42  # End-of-line comment
    """

    config = ProcessConfig(language="python", comments=True)
    result = process(source, config)

    for comment in result["comments"]:
        print(f"Text:        {comment['text']}")
        print(f"Start line:  {comment['start_line']}")
        print(f"Is block:    {comment['is_block']}")
        print()
    ```

Output:

```text
Text:        # Top-level comment
Start line:  1
Is block:    False

Text:        # Inline comment
Start line:  3
Is block:    False

Text:        # End-of-line comment
Start line:  4
Is block:    False
```text

### `docstrings` — Documentation Strings

Docstrings are automatically attached to their parent constructs in the `structure` field when `docstrings=True`.

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    source = '''
    def read_file(path: str) -> str:
        """Read and return the contents of a file.

        Args:
            path: Path to the file to read.

        Returns:
            The file contents as a string.

        Raises:
            FileNotFoundError: If the file does not exist.
        """
        with open(path) as f:
            return f.read()

    class FileCache:
        """In-memory cache for file contents."""
        pass
    '''

    config = ProcessConfig(language="python", structure=True, docstrings=True)
    result = process(source, config)

    for item in result["structure"]:
        if item.get("docstring"):
            print(f"{item['kind']} {item['name']}:")
            print(f"  {item['docstring'][:80]}...")
            print()
    ```

**Docstring conventions by language:**

| Language | Convention |
|----------|-----------|
| Python | `"""..."""` triple-quoted strings after `def`/`class` |
| Rust | `///` or `//!` doc comments above item |
| JavaScript/TypeScript | `/** ... */` JSDoc above function |
| Java | `/** ... */` Javadoc above method/class |
| Ruby | `# ...` lines immediately before `def`/`class` |
| Go | `// FuncName ...` comment block above function |
| Elixir | `@doc "..."` or `@moduledoc "..."` |

### `symbols` — All Identifiers

A deduplicated list of all identifiers referenced in the file, useful for search indexing.

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    source = """
    from os import path
    from typing import List

    def process_files(directory: str) -> List[str]:
        results = []
        for file in path.listdir(directory):
            if file.endswith(".txt"):
                results.append(file)
        return results
    """

    config = ProcessConfig(language="python", symbols=True)
    result = process(source, config)

    print("Symbols found:")
    for symbol in sorted(result["symbols"]):
        print(f"  - {symbol}")
    ```

Output:

```text
Symbols found:
  - List
  - append
  - directory
  - endswith
  - file
  - file
  - listdir
  - os
  - path
  - process_files
  - results
  - txt
  - typing
```text

### `diagnostics` — Syntax Errors

Syntax errors and error nodes detected during parsing.

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    source = """
    def broken_function(
        # missing closing paren
        print("hello")
    """

    config = ProcessConfig(language="python", diagnostics=True)
    result = process(source, config)

    if result["diagnostics"]:
        for error in result["diagnostics"]:
            print(f"Error at line {error['start_line']}, col {error['start_col']}")
            print(f"  {error['message']}")
    else:
        print("No syntax errors")
    ```

!!! tip
    A non-empty `diagnostics` list does not mean the file is unparsable—tree-sitter recovers and produces a partial tree. Use diagnostics to detect and report broken syntax.

### `metrics` — File Statistics

Basic metrics about the file.

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    source = """
    # This is a comment

    def hello():
        print("world")

    # Another comment
    x = 1
    """

    config = ProcessConfig(language="python", metrics=True)
    result = process(source, config)

    m = result["metrics"]
    print(f"Total lines:       {m['total_lines']}")
    print(f"Code lines:        {m['code_lines']}")
    print(f"Comment lines:     {m['comment_lines']}")
    print(f"Blank lines:       {m['blank_lines']}")
    print(f"Complexity:        {m.get('complexity', 'N/A')}")
    ```

**Metric fields:**

| Field | Type | Description |
|-------|------|-------------|
| `total_lines` | int | Total lines in file |
| `code_lines` | int | Non-blank, non-comment lines |
| `comment_lines` | int | Lines that are comments |
| `blank_lines` | int | Empty lines |
| `complexity` | int \| null | Cyclomatic complexity (if supported) |

## Chunking for LLMs

When `chunk_max_size > 0`, the result includes a `chunks` field with syntax-aware splits optimized for LLM token budgets. See [Chunking for LLMs](chunking.md) for full documentation.

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    config = ProcessConfig(
        language="python",
        chunk_max_size=1000,    # target 1000 tokens per chunk
        structure=True,
        imports=True,
    )
    result = process(source, config)

    for i, chunk in enumerate(result["chunks"]):
        print(f"Chunk {i+1}: lines {chunk['start_line']}-{chunk['end_line']} "
              f"({chunk['token_count']} tokens)")
    ```

## Full Example

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    source = '''
    """Module for file operations."""
    import os
    from pathlib import Path

    def read_file(path: str) -> str:
        """Read and return file contents.

        Args:
            path: Path to file.

        Returns:
            File contents.
        """
        # TODO: add error handling
        return Path(path).read_text()

    class FileManager:
        """Manage file operations."""

        def __init__(self, root: str):
            self.root = Path(root)

        def get(self, name: str) -> str:
            """Get file contents."""
            return read_file(os.path.join(self.root, name))
    '''

    # Extract everything
    config = ProcessConfig(
        language="python",
        structure=True,
        imports=True,
        exports=True,
        comments=True,
        docstrings=True,
        symbols=True,
        metrics=True,
    )
    result = process(source, config)

    # Print structure
    print("=== Structure ===")
    for item in result["structure"]:
        print(f"{item['kind']:12} {item['name']:20} ({item['start_line']}-{item['end_line']})")

    # Print imports
    print("\n=== Imports ===")
    for imp in result["imports"]:
        names = ", ".join(imp["names"]) if imp["names"] else "*"
        print(f"from {imp['source']} import {names}")

    # Print exports
    print("\n=== Exports ===")
    for exp in result["exports"]:
        print(f"{exp['kind']:12} {exp['name']}")

    # Print comments
    print("\n=== Comments ===")
    for comment in result["comments"]:
        print(f"Line {comment['start_line']:3} {comment['text']}")

    # Print metrics
    print("\n=== Metrics ===")
    m = result["metrics"]
    print(f"Lines: {m['total_lines']:3} total, {m['code_lines']:3} code, "
          f"{m['comment_lines']:3} comments, {m['blank_lines']:3} blank")

    # Print symbols
    print(f"\n=== Symbols ({len(result['symbols'])}) ===")
    print(", ".join(sorted(result["symbols"])[:20]))  # first 20
    ```

## Working with Language-Specific Results

Different languages produce different `structure` kinds, imports patterns, and docstring conventions. Always check what's available:

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    config = ProcessConfig(language="python", structure=True)
    result = process("...", config)

    # Group structure by kind
    by_kind = {}
    for item in result["structure"]:
        kind = item["kind"]
        if kind not in by_kind:
            by_kind[kind] = []
        by_kind[kind].append(item["name"])

    for kind in sorted(by_kind.keys()):
        print(f"{kind}: {by_kind[kind]}")
    ```

=== "JavaScript/TypeScript"

    ```typescript
    import { process } from "@kreuzberg/tree-sitter-language-pack";

    const config = {
      language: "typescript",
      structure: true,
    };
    const result = await process("...", config);

    // Group structure by kind
    const byKind = {};
    result.structure.forEach(item => {
      byKind[item.kind] ??= [];
      byKind[item.kind].push(item.name);
    });

    for (const kind of Object.keys(byKind).sort()) {
      console.log(`${kind}: ${byKind[kind]}`);
    }
    ```

## Performance Considerations

!!! tip "Enable Only What You Need"
    Enabling more features increases processing time. Start with just `structure=True` and add features as needed.

!!! tip "Reuse Parsers"
    The `process()` function internally uses a parser. For multiple files in the same language, consider lower-level APIs for better control.

!!! tip "Chunking Overhead"
    Chunking (`chunk_max_size > 0`) adds computational cost. Only enable if you plan to split code for LLM ingestion.

## Next Steps

- **Parse without intelligence**: Use [Parsing](parsing.md) for raw syntax trees and low-level tree navigation.
- **Split for LLMs**: See [Chunking for LLMs](chunking.md) for syntax-aware code splitting.
- **Configure cache**: Use [Configuration](configuration.md) to set cache directories and pre-download languages.
