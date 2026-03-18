---
description: "C FFI API reference for tree-sitter-language-pack"
---

# C / FFI API Reference

## Overview

The C FFI layer provides a stable C API for tree-sitter-language-pack, enabling integration with languages like Go, Java (Panama FFM), and C#.

Headers are automatically generated from Rust source using `cbindgen` and are located in `crates/ffi/include/html_to_markdown.h`.

## Installation

### C Header

```c
#include "html_to_markdown.h"
```text

Link against the compiled FFI library:

```bash
gcc -o program program.c -L. -lts_pack_ffi
```text

## Version Query

### `const char* ts_pack_version(void)`

Get the version string of the library.

**Returns:** const char* - Version string (e.g., "1.0.0")

**Note:** Do NOT free the returned pointer; it points to static data.

**Example:**

```c
#include <stdio.h>
#include "html_to_markdown.h"

int main() {
    const char* version = ts_pack_version();
    printf("Version: %s\n", version);
    return 0;
}
```text

## Language Discovery

### `const char** ts_pack_available_languages(size_t* out_count)`

Get all available language names.

**Parameters:**

- `out_count` (size_t*): Pointer to store count of languages

**Returns:** const char** - Array of language name strings

**Note:** Free the returned array with `ts_pack_free_strings()`. Strings themselves must NOT be freed.

**Example:**

```c
size_t count;
const char** languages = ts_pack_available_languages(&count);
for (size_t i = 0; i < count; i++) {
    printf("%s\n", languages[i]);
}
ts_pack_free_strings(languages);
```text

### `bool ts_pack_has_language(const char* name)`

Check if a language is available.

**Parameters:**

- `name` (const char*): Language name or alias

**Returns:** bool - True if available

**Example:**

```c
if (ts_pack_has_language("python")) {
    printf("Python available\n");
}
```text

### `size_t ts_pack_language_count(void)`

Get total number of available languages.

**Returns:** size_t - Language count

**Example:**

```c
size_t count = ts_pack_language_count();
printf("%zu languages available\n", count);
```text

## Parsing

### `TSLanguage* ts_pack_get_language(const char* name, TSPackError* out_error)`

Get a tree-sitter Language by name.

**Parameters:**

- `name` (const char*): Language name or alias
- `out_error` (TSPackError*): Error output (NULL-safe)

**Returns:** TSLanguage* - Language object, NULL on error

**Example:**

```c
TSPackError error = {0};
TSLanguage* language = ts_pack_get_language("python", &error);
if (!language) {
    fprintf(stderr, "Error: %s\n", error.message);
    return;
}
// Use language
```text

### `TSParser* ts_pack_get_parser(const char* name, TSPackError* out_error)`

Get a pre-configured Parser for a language.

**Parameters:**

- `name` (const char*): Language name
- `out_error` (TSPackError*): Error output

**Returns:** TSParser* - Parser object, NULL on error

**Note:** Caller must free with `ts_parser_delete()`.

**Example:**

```c
TSPackError error = {0};
TSParser* parser = ts_pack_get_parser("python", &error);
if (!parser) {
    fprintf(stderr, "Error: %s\n", error.message);
    return;
}

TSTree* tree = ts_parser_parse(parser, "x = 1", 5);
// ... use tree ...
ts_tree_delete(tree);
ts_parser_delete(parser);
```text

### `TSTree* ts_pack_parse(const char* source, size_t source_len, const char* language, TSPackError* out_error)`

Parse source code into a syntax tree.

**Parameters:**

- `source` (const char*): Source code bytes
- `source_len` (size_t): Length of source
- `language` (const char*): Language name
- `out_error` (TSPackError*): Error output

**Returns:** TSTree* - Parsed tree, NULL on error

**Note:** Caller must free with `ts_tree_delete()`.

**Example:**

```c
const char* code = "def hello(): pass";
TSPackError error = {0};
TSTree* tree = ts_pack_parse(code, strlen(code), "python", &error);
if (!tree) {
    fprintf(stderr, "Parse error: %s\n", error.message);
    return;
}

TSNode root = ts_tree_root_node(tree);
// ... use tree ...
ts_tree_delete(tree);
```text

## Tree Navigation

### `TSNode ts_tree_root_node(TSTree* tree)`

Get the root node of a tree.

**Parameters:**

- `tree` (TSTree*): Syntax tree

**Returns:** TSNode - Root node

**Example:**

```c
TSNode root = ts_tree_root_node(tree);
printf("Root type: %s\n", ts_node_type(root));
```text

### `const char* ts_node_type(TSNode node)`

Get the type name of a node.

**Parameters:**

- `node` (TSNode): Syntax tree node

**Returns:** const char* - Type name string

**Example:**

```c
TSNode node = ts_tree_root_node(tree);
printf("Type: %s\n", ts_node_type(node));
```text

### `uint32_t ts_node_child_count(TSNode node)`

Get number of child nodes.

**Parameters:**

- `node` (TSNode): Syntax tree node

**Returns:** uint32_t - Child count

**Example:**

```c
uint32_t count = ts_node_child_count(node);
for (uint32_t i = 0; i < count; i++) {
    TSNode child = ts_node_child(node, i);
    printf("Child %u: %s\n", i, ts_node_type(child));
}
```text

### `TSNode ts_node_child(TSNode node, uint32_t index)`

Get a child node by index.

**Parameters:**

- `node` (TSNode): Parent node
- `index` (uint32_t): Child index

**Returns:** TSNode - Child node

**Example:**

```c
TSNode first_child = ts_node_child(node, 0);
```text

## Code Intelligence

### `TSPackProcessConfig* ts_pack_process_config_new(const char* language)`

Create a new process configuration.

**Parameters:**

- `language` (const char*): Language name

**Returns:** TSPackProcessConfig* - Configuration object

**Note:** Free with `ts_pack_process_config_delete()`.

**Example:**

```c
TSPackProcessConfig* config = ts_pack_process_config_new("python");
ts_pack_process_config_set_structure(config, true);
ts_pack_process_config_set_imports(config, true);
// ... configure options ...
TSPackProcessResult* result = ts_pack_process(code, config, NULL);
ts_pack_process_config_delete(config);
```text

### Configuration Methods

- `void ts_pack_process_config_set_structure(TSPackProcessConfig* config, bool enabled)`
- `void ts_pack_process_config_set_imports(TSPackProcessConfig* config, bool enabled)`
- `void ts_pack_process_config_set_exports(TSPackProcessConfig* config, bool enabled)`
- `void ts_pack_process_config_set_comments(TSPackProcessConfig* config, bool enabled)`
- `void ts_pack_process_config_set_docstrings(TSPackProcessConfig* config, bool enabled)`
- `void ts_pack_process_config_set_symbols(TSPackProcessConfig* config, bool enabled)`
- `void ts_pack_process_config_set_diagnostics(TSPackProcessConfig* config, bool enabled)`
- `void ts_pack_process_config_set_metrics(TSPackProcessConfig* config, bool enabled)`
- `void ts_pack_process_config_set_chunks(TSPackProcessConfig* config, size_t max_size, size_t overlap)`
- `void ts_pack_process_config_set_all(TSPackProcessConfig* config, bool enabled)`

### `TSPackProcessResult* ts_pack_process(const char* source, TSPackProcessConfig* config, TSPackError* out_error)`

Extract code intelligence from source code.

**Parameters:**

- `source` (const char*): Source code
- `config` (TSPackProcessConfig*): Configuration
- `out_error` (TSPackError*): Error output

**Returns:** TSPackProcessResult* - Analysis result, NULL on error

**Note:** Free with `ts_pack_process_result_delete()`.

**Example:**

```c
TSPackProcessConfig* config = ts_pack_process_config_new("python");
ts_pack_process_config_set_all(config, true);

TSPackError error = {0};
TSPackProcessResult* result = ts_pack_process("def foo(): pass", config, &error);

if (!result) {
    fprintf(stderr, "Process error: %s\n", error.message);
    ts_pack_process_config_delete(config);
    return;
}

// Access result data
printf("Functions: %zu\n", result->structure_count);

ts_pack_process_result_delete(result);
ts_pack_process_config_delete(config);
```text

## Memory Management

### `void ts_pack_free_strings(const char** strings)`

Free a string array returned by C API.

**Parameters:**

- `strings` (const char**): String array to free

**Example:**

```c
const char** languages = ts_pack_available_languages(&count);
// ... use languages ...
ts_pack_free_strings(languages);
```text

### `void ts_pack_process_config_delete(TSPackProcessConfig* config)`

Free a process configuration.

**Parameters:**

- `config` (TSPackProcessConfig*): Configuration to free

### `void ts_pack_process_result_delete(TSPackProcessResult* result)`

Free a process result.

**Parameters:**

- `result` (TSPackProcessResult*): Result to free

## Error Handling

### `TSPackError`

Error information structure.

**Fields:**

```c
typedef struct {
    int code;              // Error code (1000+)
    const char* message;   // Error message string
    const char* context;   // Additional context (nullable)
} TSPackError;
```text

**Example:**

```c
TSPackError error = {0};
TSLanguage* lang = ts_pack_get_language("python", &error);

if (!lang) {
    switch (error.code) {
        case 1001:
            printf("Language not found\n");
            break;
        case 1002:
            printf("Download failed\n");
            break;
        default:
            printf("Error %d: %s\n", error.code, error.message);
    }
    if (error.context) {
        printf("Context: %s\n", error.context);
    }
}
```text

## Complete Example

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "html_to_markdown.h"

int main(int argc, char** argv) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <language>\n", argv[0]);
        return 1;
    }

    const char* language = argv[1];
    const char* code = "def hello(name):\n    print(f'Hello {name}')";

    // Get parser
    TSPackError error = {0};
    TSParser* parser = ts_pack_get_parser(language, &error);
    if (!parser) {
        fprintf(stderr, "Error: %s\n", error.message);
        return 1;
    }

    // Parse code
    TSTree* tree = ts_parser_parse(parser, code, strlen(code));
    if (!tree) {
        fprintf(stderr, "Parse failed\n");
        ts_parser_delete(parser);
        return 1;
    }

    // Navigate tree
    TSNode root = ts_tree_root_node(tree);
    printf("Root: %s\n", ts_node_type(root));
    printf("Children: %u\n", ts_node_child_count(root));

    for (uint32_t i = 0; i < ts_node_child_count(root); i++) {
        TSNode child = ts_node_child(root, i);
        printf("  %u: %s\n", i, ts_node_type(child));
    }

    // Process for intelligence
    TSPackProcessConfig* config = ts_pack_process_config_new(language);
    ts_pack_process_config_set_all(config, true);

    TSPackProcessResult* result = ts_pack_process(code, config, &error);
    if (!result) {
        fprintf(stderr, "Process error: %s\n", error.message);
    } else {
        printf("Structure items: %zu\n", result->structure_count);
        ts_pack_process_result_delete(result);
    }

    ts_pack_process_config_delete(config);
    ts_tree_delete(tree);
    ts_parser_delete(parser);

    return 0;
}
```text

## Linking

### Static Library

```bash
gcc -o program program.c -L. -l:libts_pack_ffi.a
```text

### Dynamic Library

```bash
gcc -o program program.c -L. -lts_pack_ffi
export LD_LIBRARY_PATH=.:$LD_LIBRARY_PATH
./program
```text

### CMake Integration

```cmake
find_library(TS_PACK_FFI ts_pack_ffi REQUIRED)
add_executable(program program.c)
target_link_libraries(program ${TS_PACK_FFI})
target_include_directories(program PRIVATE /path/to/crates/ffi/include)
```text

## ABI Stability

The C FFI is governed by semantic versioning:

- **MAJOR**: Breaking ABI changes (function signature, struct layout changes)
- **MINOR**: New functions or optional fields at struct end
- **PATCH**: Bug fixes, internal optimizations

Current ABI version: See `HTML_TO_MARKDOWN_VERSION_MAJOR` and `_MINOR` constants in header.
