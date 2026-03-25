---
description: "Go API reference for tree-sitter-language-pack"
---

# Go API Reference

## Installation

```bash
go get github.com/kreuzberg-dev/tree-sitter-language-pack/packages/go/v2
```text

## Quick Start

```go
package main

import (
    "fmt"
    "log"

    tsp "github.com/kreuzberg-dev/tree-sitter-language-pack/packages/go/v2"
)

func main() {
    // Pre-download languages
    if err := tsp.Download([]string{"python", "rust"}); err != nil {
        log.Fatal(err)
    }

    // Get a language
    lang, err := tsp.GetLanguage("python")
    if err != nil {
        log.Fatal(err)
    }

    // Get a pre-configured parser
    parser, err := tsp.GetParser("python")
    if err != nil {
        log.Fatal(err)
    }

    // Parse source code
    tree := parser.Parse([]byte("def hello(): pass"), nil)
    fmt.Println(tree.RootNode().String())

    // Extract code intelligence
    config := tsp.NewProcessConfig("python").All()
    result, err := tsp.Process("def hello(): pass", config)
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Functions: %d\n", len(result.Structure))
}
```text

## Download Management

### `Download(names []string) error`

Download specific languages to cache.

**Parameters:**

- `names` ([]string): Language names to download

**Returns:** error - nil on success

**Example:**

```go
err := tsp.Download([]string{"python", "rust", "typescript"})
if err != nil {
    log.Printf("Download failed: %v", err)
}
```text

### `DownloadAll() error`

Download all available languages (248).

**Returns:** error - nil on success

**Example:**

```go
err := tsp.DownloadAll()
if err != nil {
    log.Fatal(err)
}
```text

### `ManifestLanguages() ([]string, error)`

Get all available languages from remote manifest.

**Returns:** []string - Language names, error

**Example:**

```go
langs, err := tsp.ManifestLanguages()
if err != nil {
    log.Fatal(err)
}
fmt.Printf("Available: %d languages\n", len(langs))
```text

### `DownloadedLanguages() []string`

Get languages already cached locally.

No error return. Returns empty slice if unavailable.

**Returns:** []string - Cached language names

**Example:**

```go
cached := tsp.DownloadedLanguages()
for _, lang := range cached {
    fmt.Println(lang)
}
```text

### `CleanCache() error`

Delete all cached parser shared libraries.

**Returns:** error - nil on success

**Example:**

```go
if err := tsp.CleanCache(); err != nil {
    log.Fatal(err)
}
```text

### `CacheDir() string`

Get the current cache directory path.

**Returns:** string - Absolute cache directory path

**Example:**

```go
dir := tsp.CacheDir()
fmt.Printf("Cache at: %s\n", dir)
```text

### `Init(languages []string, cacheDir string) error`

Initialize with optional pre-downloads and cache directory.

**Parameters:**

- `languages` ([]string): Languages to download
- `cacheDir` (string): Custom cache directory (empty = default)

**Returns:** error - nil on success

**Example:**

```go
err := tsp.Init([]string{"python", "javascript"}, "/opt/ts-pack")
if err != nil {
    log.Fatal(err)
}
```text

### `Configure(cacheDir string) error`

Apply configuration without downloading.

**Parameters:**

- `cacheDir` (string): Custom cache directory (empty = default)

**Returns:** error - nil on success

**Example:**

```go
if err := tsp.Configure("/data/ts-pack"); err != nil {
    log.Fatal(err)
}
```text

## Language Discovery

### `GetLanguage(name string) (*Language, error)`

Get a tree-sitter Language by name.

Resolves aliases (e.g., `"shell"` → `"bash"`). Auto-downloads if needed.

**Parameters:**

- `name` (string): Language name or alias

**Returns:** *Language, error

**Example:**

```go
lang, err := tsp.GetLanguage("python")
if err != nil {
    log.Fatal(err)
}
defer lang.Close()

parser := language.NewParser()
tree := parser.Parse([]byte("x = 1"), nil)
```text

### `GetParser(name string) (*Parser, error)`

Get a pre-configured Parser for a language.

**Parameters:**

- `name` (string): Language name or alias

**Returns:** *Parser, error

**Example:**

```go
parser, err := tsp.GetParser("rust")
if err != nil {
    log.Fatal(err)
}
defer parser.Close()

tree := parser.Parse([]byte("fn main() {}"), nil)
```text

### `AvailableLanguages() []string`

List all available language names.

**Returns:** []string - Sorted language names

**Example:**

```go
langs := tsp.AvailableLanguages()
for _, lang := range langs {
    fmt.Println(lang)
}
```text

### `HasLanguage(name string) bool`

Check if a language is available.

**Parameters:**

- `name` (string): Language name or alias

**Returns:** bool - True if available

**Example:**

```go
if tsp.HasLanguage("python") {
    fmt.Println("Python available")
}
if tsp.HasLanguage("shell") {
    fmt.Println("Shell (alias for bash) available")
}
```text

### `LanguageCount() int`

Get total number of available languages.

**Returns:** int - Language count

**Example:**

```go
count := tsp.LanguageCount()
fmt.Printf("%d languages available\n", count)
```text

## Parsing

### `Parse(source []byte, language *Language) (*Tree, error)`

Parse source code into a syntax tree.

**Parameters:**

- `source` ([]byte): Source code
- `language` (*Language): tree-sitter Language

**Returns:** *Tree, error

**Example:**

```go
lang, _ := tsp.GetLanguage("python")
defer lang.Close()

tree, err := tsp.Parse([]byte("x = 1"), lang)
if err != nil {
    log.Fatal(err)
}
defer tree.Close()

fmt.Println(tree.RootNode().String())
```text

### `ParseString(source, language string) (*Tree, error)`

Parse source code string with language name.

**Parameters:**

- `source` (string): Source code
- `language` (string): Language name

**Returns:** *Tree, error

**Example:**

```go
tree, err := tsp.ParseString("def foo(): pass", "python")
if err != nil {
    log.Fatal(err)
}
defer tree.Close()
```text

## Code Intelligence

### `Process(source string, config *ProcessConfig) (*ProcessResult, error)`

Extract code intelligence from source code.

**Parameters:**

- `source` (string): Source code
- `config` (*ProcessConfig): Configuration

**Returns:** *ProcessResult, error

**Example:**

```go
config := tsp.NewProcessConfig("python").All()
result, err := tsp.Process("def hello(): pass", config)
if err != nil {
    log.Fatal(err)
}

fmt.Printf("Functions: %d\n", len(result.Structure))
fmt.Printf("Imports: %d\n", len(result.Imports))
fmt.Printf("Lines: %d\n", result.Metrics.TotalLines)
```text

## Types

### `ProcessConfig`

Configuration for code intelligence analysis.

**Constructor:**

```go
config := tsp.NewProcessConfig("python")
```text

**Methods:**

#### `Structure() *ProcessConfig`

Enable structure extraction.

#### `ImportExports() *ProcessConfig`

Enable imports/exports extraction.

#### `Comments() *ProcessConfig`

Enable comment extraction.

#### `Docstrings() *ProcessConfig`

Enable docstring extraction.

#### `Symbols() *ProcessConfig`

Enable symbol extraction.

#### `Metrics() *ProcessConfig`

Enable metric extraction.

#### `Diagnostics() *ProcessConfig`

Enable diagnostic extraction.

#### `WithChunks(maxSize, overlap int) *ProcessConfig`

Configure code chunking.

#### `All() *ProcessConfig`

Enable all features.

**Example:**

```go
config := tsp.NewProcessConfig("python").
    Structure().
    ImportExports().
    Comments().
    WithChunks(2000, 400)
```text

### `ProcessResult`

Result from code intelligence analysis.

**Fields:**

```go
type ProcessResult struct {
    Language    string
    Metrics     FileMetrics
    Structure   []StructureItem
    Imports     []ImportInfo
    Exports     []ExportInfo
    Comments    []CommentInfo
    Docstrings  []DocstringInfo
    Symbols     []SymbolInfo
    Diagnostics []Diagnostic
    Chunks      []CodeChunk
    ParseErrors int
}
```text

**Example:**

```go
result, _ := tsp.Process(source, config)
fmt.Printf("Language: %s\n", result.Language)
for _, item := range result.Structure {
    fmt.Printf("  %s: %s\n", item.Kind, item.Name)
}
```text

### `Language`

tree-sitter Language object.

**Methods:**

- `Close()` - Release language resources
- `Name() string` - Get language name

### `Parser`

tree-sitter Parser object.

**Methods:**

- `Close()` - Release parser resources
- `Parse(source []byte, oldTree *Tree) *Tree` - Parse source code
- `SetTimeoutMicros(micros uint64)` - Set parse timeout

### `Tree`

Parsed syntax tree.

**Methods:**

- `Close()` - Release tree resources
- `RootNode() *Node` - Get root node
- `Copy() *Tree` - Copy tree

### `Node`

Syntax tree node.

**Methods:**

- `Type() string` - Node type name
- `Kind() string` - Node kind
- `StartPoint() Point` - Start position
- `EndPoint() Point` - End position
- `Text(source []byte) string` - Get node text
- `ChildCount() int` - Number of children
- `Child(i int) *Node` - Get child node
- `String() string` - S-expression representation

## Error Handling

Always check and handle errors:

```go
lang, err := tsp.GetLanguage("python")
if err != nil {
    switch err {
    case tsp.ErrLanguageNotFound:
        log.Printf("Language not available")
    case tsp.ErrDownloadFailed:
        log.Printf("Download failed (network error?)")
    default:
        log.Printf("Error: %v", err)
    }
    return
}
defer lang.Close()
```text

## Usage Patterns

### Pre-download Languages

```go
package main

import (
    "log"
    tsp "github.com/kreuzberg-dev/tree-sitter-language-pack/packages/go/v2"
)

func init() {
    if err := tsp.Download([]string{
        "python", "rust", "typescript",
    }); err != nil {
        log.Fatal(err)
    }
}

func main() {
    // Fast, no network required
    parser, _ := tsp.GetParser("python")
    defer parser.Close()
}
```text

### Custom Cache Directory

```go
import tsp "github.com/kreuzberg-dev/tree-sitter-language-pack/packages/go/v2"

func init() {
    if err := tsp.Configure("/opt/ts-pack-cache"); err != nil {
        log.Fatal(err)
    }
}
```text

### Process Multiple Files

```go
func analyzeFiles(dir string, lang string) error {
    entries, err := os.ReadDir(dir)
    if err != nil {
        return err
    }

    config := tsp.NewProcessConfig(lang).All()

    for _, entry := range entries {
        if !entry.IsDir() {
            path := filepath.Join(dir, entry.Name())
            data, _ := os.ReadFile(path)
            source := string(data)

            result, err := tsp.Process(source, config)
            if err != nil {
                log.Printf("Error processing %s: %v", path, err)
                continue
            }

            fmt.Printf("%s: %d items\n", path, len(result.Structure))
        }
    }

    return nil
}
```text

### Concurrent Parsing

```go
import (
    "sync"
    tsp "github.com/kreuzberg-dev/tree-sitter-language-pack/packages/go/v2"
)

func parseFiles(files []string, lang string) {
    parser, _ := tsp.GetParser(lang)
    defer parser.Close()

    var wg sync.WaitGroup
    for _, file := range files {
        wg.Add(1)
        go func(f string) {
            defer wg.Done()
            data, _ := os.ReadFile(f)
            tree := parser.Parse(data, nil)
            defer tree.Close()
            // Process tree
        }(file)
    }
    wg.Wait()
}
```text

## Thread Safety

All public functions are thread-safe. Create separate Parser instances for concurrent use:

```go
// Safe: each goroutine gets its own parser
for i := 0; i < 10; i++ {
    go func() {
        parser, _ := tsp.GetParser("python")
        defer parser.Close()
        // Use parser
    }()
}
```
