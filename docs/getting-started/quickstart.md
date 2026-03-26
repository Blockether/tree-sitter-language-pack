---
title: Quick Start
description: "Parse your first file with tree-sitter-language-pack in under 5 minutes."
---

This guide walks from install to parsing code in 5 minutes.

## Step 1 — Install

=== "Python"

    ```bash
    pip install tree-sitter-language-pack
    ```

=== "Node.js"

    ```bash
    npm install @kreuzberg/tree-sitter-language-pack
    ```

=== "Rust"

    ```bash
    cargo add ts-pack-core
    ```

=== "CLI"

    ```bash
    brew install kreuzberg-dev/tap/ts-pack
    ```

## Step 2 — Get a Parser

Parsers are downloaded automatically on first use. You can also pre-download for offline use.

=== "Python"

    ```python
    from tree_sitter_language_pack import get_parser, download

    # Auto-downloads on first call
    parser = get_parser("python")

    # Or pre-download explicitly
    download(["python", "javascript", "rust"])
    ```

=== "Node.js"

    ```typescript
    import { parseString, download } from "@kreuzberg/tree-sitter-language-pack";

    // Auto-downloads on first call
    const tree = parseString("python", "print('hello')");

    // Or pre-download explicitly
    download(["python", "javascript", "rust"]);
    ```

=== "Rust"

    ```rust
    use ts_pack_core::{get_parser, download};

    // Auto-downloads on first call
    let mut parser = get_parser("python")?;

    // Or pre-download explicitly
    download(&["python", "javascript", "rust"])?;
    ```

=== "CLI"

    ```bash
    # Download for offline use
    ts-pack download python javascript rust

    # Or skip — parsing auto-downloads
    ```

## Step 3 — Parse Code

With a parser in hand, build a concrete syntax tree from source code.

=== "Python"

    ```python
    from tree_sitter_language_pack import get_parser

    parser = get_parser("python")

    source = b"""
    def greet(name: str) -> str:
        return f"Hello, {name}!"

    result = greet("world")
    """

    tree = parser.parse(source)
    root = tree.root_node

    print(root.type)           # module
    print(root.child_count)    # 2
    print(root.sexp()[:120])   # S-expression of the tree
    ```

=== "Node.js"

    ```typescript
    import { parseString, treeRootNodeType, treeRootChildCount } from "@kreuzberg/tree-sitter-language-pack";

    const source = `
    function greet(name) {
      return \`Hello, \${name}!\`;
    }
    greet("world");
    `;

    const tree = parseString("javascript", source);

    console.log(treeRootNodeType(tree));       // program
    console.log(treeRootChildCount(tree));     // 2
    ```

=== "Rust"

    ```rust
    use ts_pack_core::get_parser;

    fn main() -> anyhow::Result<()> {
        let mut parser = get_parser("rust")?;

        let source = r#"
    fn greet(name: &str) -> String {
        format!("Hello, {}!", name)
    }
    "#;

        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();

        println!("{}", root.kind());        // source_file
        println!("{}", root.child_count()); // 1
        println!("{}", root.to_sexp());
        Ok(())
    }
    ```

=== "CLI"

    ```bash
    # Parse a file and display the syntax tree
    ts-pack parse src/main.py

    # Output as JSON
    ts-pack parse src/main.py --format json

    # Parse inline code
    echo "def hello(): pass" | ts-pack parse --language python
    ```

## Step 4 — Extract Code Intelligence

Go beyond the raw syntax tree. Extract functions, classes, imports, and more with `process`.

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    source = """
    import os
    from pathlib import Path

    def read_file(path: str) -> str:
        \"\"\"Read and return the contents of a file.\"\"\"
        return Path(path).read_text()

    class FileManager:
        def __init__(self, base_dir: str):
            self.base_dir = base_dir

        def get(self, name: str) -> str:
            return read_file(os.path.join(self.base_dir, name))
    """

    config = ProcessConfig(
        language="python",
        structure=True,   # functions and classes
        imports=True,     # import statements
        comments=True,    # inline comments
        docstrings=True,  # docstring extraction
    )
    result = process(source, config)

    print(f"Imports:  {[i['name'] for i in result['imports']]}")
    print(f"Symbols:  {[s['name'] for s in result['structure']]}")
    print(f"Docstring: {result['structure'][0]['docstring']}")
    ```

=== "Node.js"

    ```typescript
    import { process } from "@kreuzberg/tree-sitter-language-pack";

    const source = `
    import fs from "fs";
    import { join } from "path";

    /**
     * Read and return the contents of a file.
     */
    function readFile(path: string): string {
      return fs.readFileSync(path, "utf8");
    }

    class FileManager {
      constructor(private baseDir: string) {}

      get(name: string): string {
        return readFile(join(this.baseDir, name));
      }
    }
    `;

    const result = await process(source, {
      language: "typescript",
      structure: true,
      imports: true,
      docstrings: true,
    });

    console.log("Imports:", result.imports.map(i => i.name));
    console.log("Symbols:", result.structure.map(s => s.name));
    ```

=== "Rust"

    ```rust
    use ts_pack_core::{process, ProcessConfig};

    fn main() -> anyhow::Result<()> {
        let source = r#"
    use std::fs;
    use std::path::Path;

    /// Read and return the contents of a file.
    fn read_file(path: &str) -> String {
        fs::read_to_string(path).unwrap()
    }

    struct FileManager {
        base_dir: String,
    }
    "#;

        let config = ProcessConfig::new("rust")
            .structure(true)
            .imports(true)
            .docstrings(true);

        let result = process(source, &config)?;

        println!("Imports: {:?}", result.imports.iter().map(|i| &i.name).collect::<Vec<_>>());
        println!("Symbols: {:?}", result.structure.iter().map(|s| &s.name).collect::<Vec<_>>());
        Ok(())
    }
    ```

=== "CLI"

    ```bash
    # Run full code intelligence on a file
    ts-pack process src/main.py --structure --imports --docstrings

    # Output as JSON for piping
    ts-pack process src/main.py --all --format json | jq '.structure[].name'
    ```

## Step 5 — Run Extraction Queries

Use `extract` to run custom tree-sitter queries and get structured results with captured text and metadata.

=== "Python"

    ```python
    import tree_sitter_language_pack as tslp

    source = """
    def greet(name: str) -> str:
        return f"Hello, {name}!"

    def farewell(name: str) -> str:
        return f"Goodbye, {name}!"
    """

    result = tslp.extract(source, {
        "language": "python",
        "patterns": {
            "functions": {
                "query": "(function_definition name: (identifier) @name)",
                "capture_output": "Text",
            }
        }
    })
    for match in result["results"]["functions"]["matches"]:
        print(match["captures"][0]["text"])
    # greet
    # farewell
    ```

## Step 6 — Chunk for LLMs

Split code at natural boundaries so language models receive coherent, complete units.

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    with open("large_module.py") as f:
        source = f.read()

    config = ProcessConfig(
        language="python",
        chunk_max_size=1500,  # max tokens per chunk
        structure=True,
    )
    result = process(source, config)

    for i, chunk in enumerate(result["chunks"]):
        print(f"Chunk {i}: {chunk['start_line']}-{chunk['end_line']} "
              f"({chunk['token_count']} tokens)")
    ```

=== "Node.js"

    ```typescript
    import { process } from "@kreuzberg/tree-sitter-language-pack";
    import { readFileSync } from "fs";

    const source = readFileSync("large_module.ts", "utf8");

    const result = await process(source, {
      language: "typescript",
      chunkMaxSize: 1500,
      structure: true,
    });

    result.chunks.forEach((chunk, i) => {
      console.log(`Chunk ${i}: lines ${chunk.startLine}-${chunk.endLine} (${chunk.tokenCount} tokens)`);
    });
    ```

=== "CLI"

    ```bash
    # Chunk a file for LLM ingestion
    ts-pack process large_module.py --chunk-size 1500 --format json \
      | jq '.chunks[] | {start: .start_line, end: .end_line, tokens: .token_count}'
    ```

## What's Next

<div class="grid cards" markdown>

- :material-book-open-outline: **Concepts**

    ---

    Understand the architecture, download model, and what code intelligence extracts.

    [:material-arrow-right: Architecture](../concepts/architecture.md) ·
    [:material-arrow-right: Download Model](../concepts/download-model.md) ·
    [:material-arrow-right: Code Intelligence](../concepts/code-intelligence.md)

- :material-wrench-outline: **Guides**

    ---

    Deep dives on specific features and real-world use cases.

    [:material-arrow-right: Chunking for LLMs](../guides/chunking.md) ·
    [:material-arrow-right: CLI Reference](../guides/cli.md)

- :material-api: **API Reference**

    ---

    Full API documentation for every language binding.

    [:material-arrow-right: Python](../api/python.md) ·
    [:material-arrow-right: Node.js](../api/typescript.md) ·
    [:material-arrow-right: Rust](../api/rust.md)

</div>
