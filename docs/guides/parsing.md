---
description: "Parse source code with the Rust API or CLI and inspect tree-sitter syntax trees."
---

# Parsing code

The stable public API for low-level syntax-tree parsing is `get_parser()`. It returns a tree-sitter parser configured for one of the bundled languages.

Language bindings expose higher-level APIs such as `process()` and `extract()` for structured analysis. Use those APIs unless you need a raw tree-sitter syntax tree.

=== "Rust"

    ```rust
    use tree_sitter_language_pack::get_parser;

    let mut parser = get_parser("python")?;
    let tree = parser
        .parse(
            b"def greet(name: str) -> str:\n    return f\"Hello {name}\"\n",
            None,
        )
        .ok_or("failed to parse source")?;

    let root = tree.root_node();
    println!("{}", root.to_sexp());
    # Ok::<(), Box<dyn std::error::Error>>(())
    ```

=== "CLI"

    ```bash
    # Print the syntax tree for a file
    ts-pack parse main.py

    # Output as JSON
    ts-pack parse main.py --format json
    ```

For batch processing in Rust, reuse the parser object. Creating one parser per parse works at small scale but adds avoidable setup overhead when processing a large file set for the same language.

!!! Tip "Language names" Names are case-insensitive. Aliases exist: `shell` -> `bash`, `makefile` -> `make`, `bazel` -> `starlark`. See [Languages](../languages.md) for the full list.

## The syntax tree

Every parse returns a `Tree` with a single root node. Its kind is the top-level grammar node for the language: `module` for Python, `program` for JavaScript, `source_file` for Rust and Go.

```rust
let mut parser = get_parser("python")?;
let tree = parser
    .parse(b"def foo(): pass\ndef bar(): pass", None)
    .ok_or("failed to parse source")?;
let root = tree.root_node();

println!("{}", root.kind());          // "module"
println!("{:?}", root.start_position());
println!("{:?}", root.end_position());
println!("{}", root.child_count());   // 2
println!("{}", root.has_error());     // false
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Field names

Grammars assign named fields to semantically meaningful children. A Python `function_definition` has `name`, `parameters`, `return_type`, and `body`. Use `child_by_field_name` to reach them directly:

```rust
let mut parser = get_parser("python")?;
let tree = parser
    .parse(b"def add(a, b):\n    return a + b", None)
    .ok_or("failed to parse source")?;
let root = tree.root_node();
let func = root.child(0).ok_or("missing function")?;

let name = func.child_by_field_name("name").ok_or("missing function name")?;
let params = func
    .child_by_field_name("parameters")
    .ok_or("missing parameters")?;

println!("{}", name.utf8_text(b"def add(a, b):\n    return a + b")?);   // "add"
println!("{}", params.utf8_text(b"def add(a, b):\n    return a + b")?); // "(a, b)"
# Ok::<(), Box<dyn std::error::Error>>(())
```

Field names are grammar-specific. To discover them, run `ts-pack parse file.py` and read the labelled S-expression output. Field names appear as `name:`, `parameters:`, `body:` before each child.

## Named vs. anonymous nodes

Named nodes carry semantic meaning (`identifier`, `call_expression`, `string`). Anonymous nodes are punctuation and keywords (`(`, `)`, `def`, `:`). Use `named_children` when you want semantic nodes and no punctuation tokens.

```rust
let cursor = &mut root.walk();
for child in root.named_children(cursor) {
    println!("{}", child.kind());
}
```

## Syntax errors

Tree-sitter does not raise on malformed syntax. It marks problem areas with `ERROR` or `MISSING` nodes and keeps parsing.

```rust
let mut parser = get_parser("python")?;
let tree = parser
    .parse(b"def broken(", None)
    .ok_or("failed to parse source")?;

println!("{}", tree.root_node().has_error()); // true
```

`has_error` on the root is a fast way to check whether any errors exist before walking. For structured diagnostics, use `process()` with diagnostics enabled.

## Node properties

| Property              | Description                                          |
| --------------------- | ---------------------------------------------------- |
| `kind()`              | Grammar node type, for example `function_definition` |
| `start_position()`    | `(row, column)`, zero-indexed                        |
| `end_position()`      | `(row, column)`, zero-indexed                        |
| `start_byte()`        | Byte offset in source                                |
| `end_byte()`          | Byte offset in source                                |
| `child_count()`       | All children including anonymous nodes               |
| `named_child_count()` | Named children only                                  |
| `utf8_text(source)`   | Raw source text for this node decoded as UTF-8       |
| `has_error()`         | True if any error nodes exist in this subtree        |
| `is_named()`          | False for anonymous nodes like `(` or `def`          |
| `parent()`            | Enclosing node, or `None` for the root               |

## Next steps

- [Code intelligence](intelligence.md) — extract functions, imports, docstrings, and symbols without writing a tree walker
- [Extraction queries](extraction.md) — public API status for custom query helpers
- [Languages](../languages.md) — all 306 supported languages and their aliases
