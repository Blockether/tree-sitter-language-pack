---
title: Extraction queries
description: "Custom extraction queries are not part of the v1.9 public API — use process(), bundled query sources, or manual AST traversal."
---

## Extraction queries

Custom query execution helpers are not exported by the Rust crate or generated language bindings in v1.9.

Use [`process()`](intelligence.md) for supported code intelligence fields such as structure, imports, exports, comments, docstrings, symbols, diagnostics, metrics, and chunks. The implementation extracts these fields with manual AST traversal in the Rust core.

Bundled query helper functions return query source strings only; they do not execute queries:

| Helper | What it returns |
| ------ | --------------- |
| `get_highlights_query(language)` | `highlights.scm` source, when bundled |
| `get_injections_query(language)` | `injections.scm` source, when bundled |
| `get_locals_query(language)` | `locals.scm` source, when bundled |
| `get_tags_query(language)` <span class="version-badge">Available by v1.9</span> | `tags.scm` source, when bundled |

If you need custom extraction, call [`get_parser()`](parsing.md), parse the source with `Parser.parse(&str)` or `Parser.parse_bytes(&[u8])`, then walk the tree manually or run tree-sitter query APIs in your host language.

### Next steps

- [Code intelligence](intelligence.md) — built-in extraction for common patterns
- [Parsing code](parsing.md) — raw syntax trees and low-level node traversal
