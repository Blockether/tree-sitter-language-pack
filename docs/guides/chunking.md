---
description: "Syntax-aware code chunking for LLMs — split code at natural boundaries, never mid-function."
---

# Chunking for LLMs

When feeding source code to a language model, naive line-count or character-count splitting produces broken, incoherent fragments. A function split across two chunks loses its signature. A class split mid-method gives the model half a definition. tree-sitter-language-pack solves this with **syntax-aware chunking**: it walks the concrete syntax tree and splits only at natural boundaries.

## Why Syntax-Aware Chunking Matters

Consider this Python file:

```python
def process_order(order_id: str, quantity: int) -> dict:
    """Process an order and return the result."""
    # validate input
    if quantity <= 0:
        raise ValueError("quantity must be positive")
    item = fetch_item(order_id)
    price = item["price"] * quantity
    return {"order_id": order_id, "total": price, "status": "pending"}
```text

Naive chunking at 100 tokens might split after `raise ValueError(...)`, leaving the return statement in the next chunk. The model sees an incomplete function in both chunks, with no way to understand the full intent.

Syntax-aware chunking keeps `process_order` together as one unit. Only when a single function exceeds the token budget does the chunker split inside it — and it marks this clearly.

## Basic Usage

=== "Python"

    ```python
    from tree_sitter_language_pack import process, ProcessConfig

    with open("src/service.py") as f:
        source = f.read()

    config = ProcessConfig(
        language="python",
        chunk_max_size=1000,  # target tokens per chunk
        structure=True,       # optionally include structure info
    )
    result = process(source, config)

    for i, chunk in enumerate(result["chunks"]):
        print(f"Chunk {i + 1}: lines {chunk['start_line']}-{chunk['end_line']} "
              f"({chunk['token_count']} tokens)")
        print(chunk["content"][:80] + "...")
        print()
    ```

=== "Node.js"

    ```typescript
    import { process } from "@kreuzberg/tree-sitter-language-pack";
    import { readFileSync } from "fs";

    const source = readFileSync("src/service.ts", "utf8");

    const result = await process(source, {
      language: "typescript",
      chunkMaxSize: 1000,
      structure: true,
    });

    result.chunks.forEach((chunk, i) => {
      console.log(`Chunk ${i + 1}: lines ${chunk.startLine}-${chunk.endLine} (${chunk.tokenCount} tokens)`);
    });
    ```

=== "Rust"

    ```rust
    use ts_pack_core::{process, ProcessConfig};
    use std::fs;

    let source = fs::read_to_string("src/service.rs")?;

    let config = ProcessConfig::new("rust")
        .chunk_max_size(1000)
        .structure(true);

    let result = process(&source, &config)?;

    for (i, chunk) in result.chunks.iter().enumerate() {
        println!("Chunk {}: lines {}-{} ({} tokens)",
            i + 1, chunk.start_line, chunk.end_line, chunk.token_count);
    }
    ```

=== "CLI"

    ```bash
    ts-pack process src/service.py --chunk-size 1000 --format json \
      | jq '.chunks[] | {lines: "\(.start_line)-\(.end_line)", tokens: .token_count}'
    ```

## Chunk Structure

Each chunk contains:

| Field | Type | Description |
|-------|------|-------------|
| `content` | string | The source code text for this chunk |
| `start_line` | int | First line of the chunk (1-indexed) |
| `end_line` | int | Last line of the chunk (1-indexed) |
| `token_count` | int | Estimated token count (cl100k approximation) |
| `node_types` | list[str] | Tree-sitter node types at the top of this chunk |
| `is_partial` | bool | `True` if a single construct was split across chunks |

## How the Chunker Works

The chunker operates in three passes:

**Pass 1: Collect leaf units.** Walk the syntax tree and collect all top-level declarations (functions, classes, methods, etc.) as atomic units. Comments and docstrings above a declaration are attached to it.

**Pass 2: Pack units into chunks.** Greedily pack units into chunks without exceeding `chunk_max_size`. When the current chunk would overflow, close it and start a new one.

**Pass 3: Split oversized units.** If a single unit (e.g., a very large function) exceeds `chunk_max_size` on its own, split it at the next logical sub-boundary (e.g., between methods in a class, or between statement blocks in a function).

This strategy ensures:

- Functions are never split unless they are individually too large.
- A decorator or docstring is always in the same chunk as the function it belongs to.
- Class definitions keep their method list together where possible.
- Imports are grouped into a single chunk at the top.

## Token Budget

The `chunk_max_size` parameter is an **upper bound** on tokens per chunk, not a fixed size. The chunker may produce smaller chunks when a natural boundary falls before the limit, and may slightly exceed the limit when the only split point is past it.

Token counting uses the `cl100k_base` approximation (4 characters ≈ 1 token), which is a close match for GPT-4, Claude, and Llama-family models. You can override this:

=== "Python"

    ```python
    config = ProcessConfig(
        language="python",
        chunk_max_size=1000,
        chunk_overlap=100,    # overlap tokens between adjacent chunks
    )
    ```

=== "Node.js"

    ```typescript
    const result = await process(source, {
      language: "python",
      chunkMaxSize: 1000,
      chunkOverlap: 100,   // repeat last N tokens of previous chunk
    });
    ```

## Chunk Overlap

For retrieval use cases, you may want adjacent chunks to share some context. Set `chunk_overlap` to repeat the last N tokens of the previous chunk at the start of the next:

```python
config = ProcessConfig(
    language="python",
    chunk_max_size=800,
    chunk_overlap=150,  # repeat ~150 tokens of context
)
```text

!!! warning "Overlap increases storage"
    Overlap causes chunks to share content. For storage in a vector database, account for the increased total token count across chunks when planning your embedding budget.

## Including Structure Metadata

When `structure=True` is also set, each chunk's `node_types` field tells you what kind of code it contains, which is useful for metadata-enriched vector store ingestion:

```python
config = ProcessConfig(
    language="python",
    chunk_max_size=1000,
    structure=True,
    docstrings=True,
)
result = process(source, config)

# Build vector store documents
documents = []
for chunk in result["chunks"]:
    documents.append({
        "content": chunk["content"],
        "metadata": {
            "language": "python",
            "start_line": chunk["start_line"],
            "end_line": chunk["end_line"],
            "node_types": chunk["node_types"],
            "token_count": chunk["token_count"],
        }
    })
```text

## Real-World Example: Indexing a Repository

```python
import os
from pathlib import Path
from tree_sitter_language_pack import process, ProcessConfig, has_language

LANGUAGE_MAP = {
    ".py": "python",
    ".js": "javascript",
    ".ts": "typescript",
    ".rs": "rust",
    ".go": "go",
    ".java": "java",
    ".rb": "ruby",
    ".ex": "elixir",
    ".exs": "elixir",
    ".php": "php",
    ".cs": "csharp",
    ".cpp": "cpp",
    ".c": "c",
    ".kt": "kotlin",
    ".swift": "swift",
}

def chunk_repository(repo_path: str, chunk_size: int = 1000) -> list[dict]:
    chunks = []
    for root, _, files in os.walk(repo_path):
        for filename in files:
            ext = Path(filename).suffix
            language = LANGUAGE_MAP.get(ext)
            if not language or not has_language(language):
                continue

            filepath = os.path.join(root, filename)
            try:
                source = Path(filepath).read_text(encoding="utf-8", errors="ignore")
            except OSError:
                continue

            config = ProcessConfig(
                language=language,
                chunk_max_size=chunk_size,
                structure=True,
                imports=True,
                docstrings=True,
            )
            result = process(source, config)

            for chunk in result["chunks"]:
                chunks.append({
                    "content": chunk["content"],
                    "file": filepath,
                    "start_line": chunk["start_line"],
                    "end_line": chunk["end_line"],
                    "language": language,
                    "node_types": chunk["node_types"],
                    "token_count": chunk["token_count"],
                })
    return chunks

# Index a repository
docs = chunk_repository("./my-project", chunk_size=800)
print(f"Generated {len(docs)} chunks from {len(set(d['file'] for d in docs))} files")
```text

## Chunking vs. Splitting by File

For large codebases, you might consider sending entire small files as single chunks and only chunking large files. Here is a pattern:

```python
MAX_FILE_TOKENS = 600   # treat files under this as one chunk
CHUNK_SIZE = 800

config_full = ProcessConfig(language=language, structure=True, imports=True)
config_chunked = ProcessConfig(language=language, chunk_max_size=CHUNK_SIZE, structure=True, imports=True)

result = process(source, config_full)
file_tokens = result["metrics"].get("total_tokens", len(source) // 4)

if file_tokens <= MAX_FILE_TOKENS:
    # Use the whole file as one chunk
    chunks = [{"content": source, "start_line": 1, "end_line": result["metrics"]["total_lines"]}]
else:
    result = process(source, config_chunked)
    chunks = result["chunks"]
```
