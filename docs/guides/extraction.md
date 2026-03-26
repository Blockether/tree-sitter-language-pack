---
title: Extraction Queries
description: "Run custom tree-sitter queries against parsed code and retrieve structured results with text, node metadata, and child fields."
---

Extraction queries run arbitrary [tree-sitter S-expression queries](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html) against parsed source code and return structured results. Each match includes captured text, node metadata (type, position, byte offsets), and optionally the text of named child fields.

Use `extract()` when you need custom pattern matching beyond what `process()` provides. Use `process()` with its built-in analysis features (structure, imports, exports, etc.) for standard code intelligence. The two can also be combined: `ProcessConfig.extractions` runs extraction patterns alongside the standard analysis pass.

## Basic Usage

=== "Python"

    ```python
    from tree_sitter_language_pack import extract

    source = "def hello(): pass\ndef world(): pass\n"

    result = extract(source, {
        "language": "python",
        "patterns": {
            "functions": {
                "query": "(function_definition name: (identifier) @fn_name) @fn_def",
            }
        }
    })

    for match in result["functions"]["matches"]:
        for capture in match["captures"]:
            print(capture["name"], capture["text"])
    ```

=== "Node.js"

    ```typescript
    import { extract } from "@anthropic/tree-sitter-language-pack";

    const source = "def hello(): pass\ndef world(): pass\n";

    const result = extract(source, {
      language: "python",
      patterns: {
        functions: {
          query: "(function_definition name: (identifier) @fn_name) @fn_def",
        },
      },
    });

    for (const match of result.functions.matches) {
      for (const capture of match.captures) {
        console.log(capture.name, capture.text);
      }
    }
    ```

=== "Rust"

    ```rust
    use tree_sitter_language_pack::{
        ExtractionConfig, ExtractionPattern, extract_patterns,
    };
    use ahash::AHashMap;

    let mut patterns = AHashMap::new();
    patterns.insert("functions".to_string(), ExtractionPattern {
        query: "(function_definition name: (identifier) @fn_name) @fn_def"
            .to_string(),
        capture_output: Default::default(),
        child_fields: Vec::new(),
        max_results: None,
        byte_range: None,
    });

    let config = ExtractionConfig {
        language: "python".to_string(),
        patterns,
    };

    let result = extract_patterns(
        "def hello(): pass\ndef world(): pass\n",
        &config,
    )
    .unwrap();
    let fns = &result.results["functions"];
    assert_eq!(fns.total_count, 2);
    ```

## Configuration

### ExtractionConfig

| Field      | Type                             | Description                                                      |
|------------|----------------------------------|------------------------------------------------------------------|
| `language` | `str`                            | Language name (e.g., `"python"`, `"typescript"`, `"rust"`)       |
| `patterns` | `dict[str, ExtractionPattern]`   | Named patterns to run. Keys become the keys in the result object |

### ExtractionPattern

| Field            | Type                          | Default    | Description                                                    |
|------------------|-------------------------------|------------|----------------------------------------------------------------|
| `query`          | `str`                         | required   | Tree-sitter S-expression query                                 |
| `capture_output` | `"Text"` / `"Node"` / `"Full"` | `"Full"` | Controls what data is included per capture (see below)         |
| `child_fields`   | `list[str]`                   | `[]`       | Named child fields to extract from each captured node          |
| `max_results`    | `int` or `null`               | `null`     | Maximum number of matches to return; `null` means unlimited    |
| `byte_range`     | `[start, end]` or `null`      | `null`     | Restrict matches to a `(start, end)` byte range in the source |

## Capture Output Modes

The `capture_output` field controls what data each `CaptureResult` contains:

| Mode   | `text` field | `node` field | Use case                              |
|--------|:------------:|:------------:|---------------------------------------|
| `Text` | present      | `null`       | When you only need matched text       |
| `Node` | `null`       | present      | When you only need position/type info |
| `Full` | present      | present      | When you need both (default)          |

The `node` field is a `NodeInfo` object with `type`, `start_byte`, `end_byte`, `start_point` (row/column), and `end_point`.

=== "Python"

    ```python
    result = extract(source, {
        "language": "python",
        "patterns": {
            "names": {
                "query": "(function_definition name: (identifier) @fn_name)",
                "capture_output": "Text",
            }
        }
    })

    capture = result["names"]["matches"][0]["captures"][0]
    assert capture["text"] == "hello"
    assert capture["node"] is None
    ```

=== "Node.js"

    ```typescript
    const result = extract(source, {
      language: "python",
      patterns: {
        names: {
          query: "(function_definition name: (identifier) @fn_name)",
          captureOutput: "Text",
        },
      },
    });

    const capture = result.names.matches[0].captures[0];
    console.log(capture.text); // "hello"
    console.log(capture.node); // null
    ```

=== "Rust"

    ```rust
    use tree_sitter_language_pack::CaptureOutput;

    let pattern = ExtractionPattern {
        query: "(function_definition name: (identifier) @fn_name)".to_string(),
        capture_output: CaptureOutput::Text,
        child_fields: Vec::new(),
        max_results: None,
        byte_range: None,
    };
    ```

## Child Fields

Use `child_fields` to extract the text of named children from each captured node. Field names correspond to tree-sitter field names in the grammar (e.g., `name`, `parameters`, `body`, `return_type`).

=== "Python"

    ```python
    result = extract("def greet(name): pass\n", {
        "language": "python",
        "patterns": {
            "functions": {
                "query": "(function_definition) @fn_def",
                "child_fields": ["name", "parameters"],
            }
        }
    })

    capture = result["functions"]["matches"][0]["captures"][0]
    print(capture["child_fields"]["name"])        # "greet"
    print(capture["child_fields"]["parameters"])  # "(name)"
    ```

=== "Rust"

    ```rust
    let pattern = ExtractionPattern {
        query: "(function_definition) @fn_def".to_string(),
        capture_output: CaptureOutput::Full,
        child_fields: vec!["name".to_string(), "parameters".to_string()],
        max_results: None,
        byte_range: None,
    };
    ```

If a requested child field does not exist on a given node, its value is `None` / `null`.

## Byte Range

Restrict extraction to a portion of the source by setting `byte_range` to a `(start, end)` tuple. Only matches whose root node falls within the range are returned.

```python
source = "def a(): pass\ndef b(): pass\ndef c(): pass\n"

result = extract(source, {
    "language": "python",
    "patterns": {
        "fns": {
            "query": "(function_definition name: (identifier) @fn_name)",
            "byte_range": [14, 28],
        }
    }
})

assert len(result["fns"]["matches"]) == 1
assert result["fns"]["matches"][0]["captures"][0]["text"] == "b"
```

## Result Truncation

When `max_results` is set, the returned `matches` list is capped at that number. The `total_count` field always reflects the true number of matches in the source, so you can detect truncation:

```python
result = extract(source_with_many_functions, {
    "language": "python",
    "patterns": {
        "fns": {
            "query": "(function_definition name: (identifier) @fn_name)",
            "max_results": 5,
        }
    }
})

pattern = result["fns"]
print(len(pattern["matches"]))  # at most 5
print(pattern["total_count"])   # actual count, e.g. 42
```

## Validation

Use `validate_extraction()` to check query syntax without running extraction. Returns per-pattern diagnostics including capture names, pattern count, warnings, and errors.

=== "Python"

    ```python
    from tree_sitter_language_pack import validate_extraction

    result = validate_extraction({
        "language": "python",
        "patterns": {
            "good": {
                "query": "(function_definition name: (identifier) @fn_name)",
            },
            "bad": {
                "query": "((((not valid syntax",
            }
        }
    })

    print(result["valid"])  # False

    good = result["patterns"]["good"]
    print(good["valid"])          # True
    print(good["capture_names"]) # ["fn_name"]
    print(good["pattern_count"]) # 1

    bad = result["patterns"]["bad"]
    print(bad["valid"])   # False
    print(bad["errors"])  # ["<query syntax error>"]
    ```

=== "Node.js"

    ```typescript
    import { validateExtraction } from "@anthropic/tree-sitter-language-pack";

    const result = validateExtraction({
      language: "python",
      patterns: {
        fns: {
          query: "(function_definition name: (identifier) @fn_name)",
        },
      },
    });

    console.log(result.valid);                     // true
    console.log(result.patterns.fns.captureNames); // ["fn_name"]
    ```

=== "Rust"

    ```rust
    use tree_sitter_language_pack::validate_extraction;

    let result = validate_extraction(&config).unwrap();
    assert!(result.valid);
    assert!(result.patterns["fns"]
        .capture_names
        .contains(&"fn_name".to_string()));
    ```

The `PatternValidation` struct returned per pattern contains:

| Field           | Type         | Description                                     |
|-----------------|--------------|--------------------------------------------------|
| `valid`         | `bool`       | Whether the query compiled                       |
| `capture_names` | `list[str]`  | Capture names defined in the query               |
| `pattern_count` | `int`        | Number of patterns in the query                  |
| `warnings`      | `list[str]`  | Non-fatal warnings (e.g., empty child field name)|
| `errors`        | `list[str]`  | Fatal errors (e.g., query syntax errors)         |

## Compiled Extraction (Rust only)

`CompiledExtraction` pre-compiles query patterns so they can be reused across multiple source inputs without recompilation overhead. This is relevant when processing many files with the same set of patterns.

```rust
use tree_sitter_language_pack::{CompiledExtraction, ExtractionConfig};

let config = ExtractionConfig {
    language: "python".to_string(),
    patterns, // AHashMap<String, ExtractionPattern>
};

let compiled = CompiledExtraction::compile(&config).unwrap();

// Reuse across multiple inputs
let r1 = compiled.extract("def a(): pass\n").unwrap();
let r2 = compiled.extract("def x(): pass\ndef y(): pass\n").unwrap();

assert_eq!(r1.results["fns"].total_count, 1);
assert_eq!(r2.results["fns"].total_count, 2);
```

`CompiledExtraction` is `Send + Sync`, so it can be shared across threads. A new `QueryCursor` is created per `extract()` call.

To extract from an already-parsed tree (avoiding a re-parse):

```rust
let tree = tree_sitter_language_pack::parse_string("python", source.as_bytes())
    .unwrap();
let result = compiled
    .extract_from_tree(&tree, source.as_bytes())
    .unwrap();
```

## Integration with process()

`ProcessConfig` has an `extractions` field that runs extraction patterns alongside the standard analysis pass. Results appear in `ProcessResult.extractions`, keyed by pattern name.

=== "Python"

    ```python
    from tree_sitter_language_pack import process

    result = process(source, {
        "language": "python",
        "structure": True,
        "extractions": {
            "decorators": {
                "query": "(decorator) @dec",
                "capture_output": "Text",
            }
        }
    })

    # Standard analysis results
    print(result["structure"])

    # Custom extraction results
    print(result["extractions"]["decorators"]["matches"])
    ```

=== "Node.js"

    ```typescript
    import { process } from "@anthropic/tree-sitter-language-pack";

    const result = process(source, {
      language: "python",
      structure: true,
      extractions: {
        decorators: {
          query: "(decorator) @dec",
          captureOutput: "Text",
        },
      },
    });
    ```

=== "Rust"

    ```rust
    use tree_sitter_language_pack::{ProcessConfig, ExtractionPattern, CaptureOutput};

    let mut config = ProcessConfig::new("python");
    let mut extractions = ahash::AHashMap::new();
    extractions.insert("decorators".to_string(), ExtractionPattern {
        query: "(decorator) @dec".to_string(),
        capture_output: CaptureOutput::Text,
        child_fields: Vec::new(),
        max_results: None,
        byte_range: None,
    });
    config.extractions = Some(extractions);
    ```

## Available Bindings

| Binding    | `extract()` | `validate_extraction()` |
|------------|:-----------:|:-----------------------:|
| Python     | yes         | yes                     |
| Node.js    | yes         | yes                     |
| Rust       | yes         | yes                     |
| Ruby       | yes         | yes                     |
| Elixir     | yes         | yes                     |
| PHP        | yes         | yes                     |
| WASM       | yes         | yes                     |
| C FFI      | yes         | yes                     |
| Go         | not yet     | not yet                 |
| C#         | not yet     | not yet                 |
| Java       | not yet     | not yet                 |
