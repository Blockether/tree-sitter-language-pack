---
description: "Java API reference for tree-sitter-language-pack"
---

# Java API Reference

## Installation

Add to `pom.xml`:

```xml
<dependency>
    <groupId>dev.kreuzberg</groupId>
    <artifactId>tree-sitter-language-pack</artifactId>
    <version>1.0.0</version>
</dependency>
```text

## Quick Start

```java
import dev.kreuzberg.treesitter.*;

public class Main {
    public static void main(String[] args) throws Exception {
        // Pre-download languages
        TsPackRegistry.download(new String[]{"python", "rust"});

        // Get a language
        Language language = TsPackRegistry.getLanguage("python");

        // Get a pre-configured parser
        Parser parser = TsPackRegistry.getParser("python");
        Tree tree = parser.parse("def hello(): pass".getBytes());
        System.out.println(tree.getRootNode().sexp());

        // Extract code intelligence
        ProcessConfig config = new ProcessConfig("python").all();
        ProcessResult result = TsPackRegistry.process("def hello(): pass", config);
        System.out.printf("Functions: %d%n", result.getStructure().size());
    }
}
```text

## Download Management

### `TsPackRegistry.download(String[] names): void`

Download specific languages to cache.

**Parameters:**

- `names` (String[]): Language names to download

**Throws:**

- `DownloadException`: If language not found or download fails

**Example:**

```java
try {
    TsPackRegistry.download(new String[]{"python", "rust", "typescript"});
} catch (DownloadException e) {
    System.err.println("Download failed: " + e.getMessage());
}
```text

### `TsPackRegistry.downloadAll(): void`

Download all available languages (170+).

**Throws:**

- `DownloadException`: If manifest fetch or download fails

**Example:**

```java
try {
    TsPackRegistry.downloadAll();
    System.out.println("All languages downloaded");
} catch (DownloadException e) {
    e.printStackTrace();
}
```text

### `TsPackRegistry.manifestLanguages(): List<String>`

Get all available languages from remote manifest.

**Returns:** List<String> - Available language names

**Throws:**

- `DownloadException`: If manifest fetch fails

**Example:**

```java
try {
    List<String> languages = TsPackRegistry.manifestLanguages();
    System.out.printf("Available: %d languages%n", languages.size());
} catch (DownloadException e) {
    e.printStackTrace();
}
```text

### `TsPackRegistry.downloadedLanguages(): List<String>`

Get languages already cached locally.

Does not perform network requests.

**Returns:** List<String> - Cached language names

**Example:**

```java
List<String> cached = TsPackRegistry.downloadedLanguages();
for (String lang : cached) {
    System.out.println(lang);
}
```text

### `TsPackRegistry.cleanCache(): void`

Delete all cached parser shared libraries.

**Throws:**

- `DownloadException`: If cache cannot be removed

**Example:**

```java
try {
    TsPackRegistry.cleanCache();
    System.out.println("Cache cleaned");
} catch (DownloadException e) {
    e.printStackTrace();
}
```text

### `TsPackRegistry.cacheDir(): String`

Get the current cache directory path.

**Returns:** String - Absolute cache directory path

**Example:**

```java
String cacheDir = TsPackRegistry.cacheDir();
System.out.printf("Cache at: %s%n", cacheDir);
```text

### `TsPackRegistry.init(String[] languages, String cacheDir): void`

Initialize with optional pre-downloads and cache directory.

**Parameters:**

- `languages` (String[]): Languages to download (null = skip)
- `cacheDir` (String): Custom cache directory (null = default)

**Throws:**

- `DownloadException`: If configuration or download fails

**Example:**

```java
try {
    TsPackRegistry.init(
        new String[]{"python", "javascript"},
        "/opt/ts-pack"
    );
} catch (DownloadException e) {
    e.printStackTrace();
}
```text

### `TsPackRegistry.configure(String cacheDir): void`

Apply configuration without downloading.

**Parameters:**

- `cacheDir` (String): Custom cache directory (null = default)

**Throws:**

- `DownloadException`: If lock cannot be acquired

**Example:**

```java
try {
    TsPackRegistry.configure("/data/ts-pack");
} catch (DownloadException e) {
    e.printStackTrace();
}
```text

## Language Discovery

### `TsPackRegistry.getLanguage(String name): Language`

Get a tree-sitter Language by name.

Resolves aliases (e.g., `"shell"` → `"bash"`). Auto-downloads if needed.

**Parameters:**

- `name` (String): Language name or alias

**Returns:** Language - tree-sitter Language object

**Throws:**

- `LanguageNotFoundException`: If language not recognized
- `DownloadException`: If auto-download fails

**Example:**

```java
try {
    Language language = TsPackRegistry.getLanguage("python");
    Parser parser = language.createParser();
    Tree tree = parser.parse("x = 1".getBytes());
    System.out.println(tree.getRootNode().getType()); // "module"
} catch (LanguageNotFoundException e) {
    System.err.println("Language not found: " + e.getMessage());
}
```text

### `TsPackRegistry.getParser(String name): Parser`

Get a pre-configured Parser for a language.

**Parameters:**

- `name` (String): Language name or alias

**Returns:** Parser - Pre-configured tree-sitter Parser

**Throws:**

- `LanguageNotFoundException`: If language not recognized
- `DownloadException`: If auto-download fails
- `ParserException`: If parser setup fails

**Example:**

```java
try {
    Parser parser = TsPackRegistry.getParser("rust");
    Tree tree = parser.parse("fn main() {}".getBytes());
    System.out.println(!tree.hasErrors()); // true
} catch (Exception e) {
    e.printStackTrace();
}
```text

### `TsPackRegistry.availableLanguages(): List<String>`

List all available language names.

**Returns:** List<String> - Sorted language names

**Example:**

```java
List<String> languages = TsPackRegistry.availableLanguages();
for (String lang : languages) {
    System.out.println(lang);
}
```text

### `TsPackRegistry.hasLanguage(String name): boolean`

Check if a language is available.

**Parameters:**

- `name` (String): Language name or alias

**Returns:** boolean - True if available

**Example:**

```java
if (TsPackRegistry.hasLanguage("python")) {
    System.out.println("Python available");
}
assert TsPackRegistry.hasLanguage("shell"); // alias for bash
```text

### `TsPackRegistry.languageCount(): int`

Get total number of available languages.

**Returns:** int - Language count

**Example:**

```java
int count = TsPackRegistry.languageCount();
System.out.printf("%d languages available%n", count);
```text

## Parsing

### `TsPackRegistry.parse(byte[] source, String language): Tree`

Parse source code into a syntax tree.

**Parameters:**

- `source` (byte[]): Source code bytes
- `language` (String): Language name

**Returns:** Tree - Parsed syntax tree

**Throws:**

- `LanguageNotFoundException`: If language not found
- `ParseException`: If parsing fails

**Example:**

```java
try {
    Tree tree = TsPackRegistry.parse("def foo(): pass".getBytes(), "python");
    System.out.println(tree.getRootNode().sexp());
} catch (Exception e) {
    e.printStackTrace();
}
```text

## Code Intelligence

### `TsPackRegistry.process(String source, ProcessConfig config): ProcessResult`

Extract code intelligence from source code.

**Parameters:**

- `source` (String): Source code
- `config` (ProcessConfig): Configuration

**Returns:** ProcessResult - Analysis result

**Throws:**

- `LanguageNotFoundException`: If language not found
- `ParseException`: If parsing fails
- `ProcessException`: If analysis fails

**Example:**

```java
try {
    ProcessConfig config = new ProcessConfig("python").all();
    ProcessResult result = TsPackRegistry.process(
        "def hello(): pass",
        config
    );

    System.out.printf("Functions: %d%n", result.getStructure().size());
    System.out.printf("Lines: %d%n", result.getMetrics().getTotalLines());
} catch (Exception e) {
    e.printStackTrace();
}
```text

## Types

### `ProcessConfig`

Configuration for code intelligence analysis.

**Constructor:**

```java
ProcessConfig config = new ProcessConfig("python");
```text

**Methods:**

#### `structure(): ProcessConfig`

Enable structure extraction.

#### `importExports(): ProcessConfig`

Enable imports/exports extraction.

#### `comments(): ProcessConfig`

Enable comment extraction.

#### `docstrings(): ProcessConfig`

Enable docstring extraction.

#### `symbols(): ProcessConfig`

Enable symbol extraction.

#### `metrics(): ProcessConfig`

Enable metric extraction.

#### `diagnostics(): ProcessConfig`

Enable diagnostic extraction.

#### `withChunks(int maxSize, int overlap): ProcessConfig`

Configure code chunking.

#### `all(): ProcessConfig`

Enable all features.

**Example:**

```java
ProcessConfig config = new ProcessConfig("python")
    .structure()
    .importExports()
    .comments()
    .withChunks(2000, 400);
```text

### `ProcessResult`

Result from code intelligence analysis.

**Methods:**

- `String getLanguage()` - Get language name
- `FileMetrics getMetrics()` - Get file metrics
- `List<StructureItem> getStructure()` - Get code structure
- `List<ImportInfo> getImports()` - Get imports
- `List<ExportInfo> getExports()` - Get exports
- `List<CommentInfo> getComments()` - Get comments
- `List<DocstringInfo> getDocstrings()` - Get docstrings
- `List<SymbolInfo> getSymbols()` - Get symbols
- `List<Diagnostic> getDiagnostics()` - Get diagnostics
- `List<CodeChunk> getChunks()` - Get code chunks

**Example:**

```java
ProcessResult result = TsPackRegistry.process(source, config);

System.out.println("Language: " + result.getLanguage());
for (StructureItem item : result.getStructure()) {
    System.out.printf("  %s: %s%n", item.getKind(), item.getName());
}
```text

### `Language`

tree-sitter Language object.

**Methods:**

- `Parser createParser()` - Create a new parser for this language
- `String getName()` - Get language name

### `Parser`

tree-sitter Parser object.

**Methods:**

- `Tree parse(byte[] source)` - Parse source code
- `Tree parse(byte[] source, Tree oldTree)` - Parse with incremental update
- `void setTimeoutMicros(long micros)` - Set parse timeout

### `Tree`

Parsed syntax tree.

**Methods:**

- `Node getRootNode()` - Get root node
- `Tree copy()` - Copy tree
- `void close()` - Close tree

### `Node`

Syntax tree node.

**Methods:**

- `String getType()` - Get node type
- `String getKind()` - Get node kind
- `Point getStartPoint()` - Get start position
- `Point getEndPoint()` - Get end position
- `String getText(byte[] source)` - Get node text
- `int getChildCount()` - Get number of children
- `Node getChild(int index)` - Get child node
- `String sexp()` - Get S-expression

## Exception Handling

```java
import dev.kreuzberg.treesitter.*;

try {
    Language language = TsPackRegistry.getLanguage("python");
    Parser parser = language.createParser();
    Tree tree = parser.parse("x = 1".getBytes());
} catch (LanguageNotFoundException e) {
    System.err.println("Language not available");
} catch (DownloadException e) {
    System.err.println("Download failed");
} catch (ParseException e) {
    System.err.println("Parse error");
} catch (Exception e) {
    System.err.println("Unexpected error");
}
```text

## Usage Patterns

### Pre-download Languages

```java
public class App {
    static {
        try {
            TsPackRegistry.download(new String[]{
                "python", "rust", "typescript"
            });
        } catch (DownloadException e) {
            throw new RuntimeException("Failed to download languages", e);
        }
    }

    public static void main(String[] args) throws Exception {
        // Fast, no network required
        Parser parser = TsPackRegistry.getParser("python");
        // ...
    }
}
```text

### Custom Cache Directory

```java
public class App {
    static {
        try {
            TsPackRegistry.configure("/opt/ts-pack-cache");
        } catch (DownloadException e) {
            throw new RuntimeException("Configuration failed", e);
        }
    }
}
```text

### Batch Processing

```java
public class Analyzer {
    private final Parser parser;

    public Analyzer(String language) throws Exception {
        this.parser = TsPackRegistry.getParser(language);
    }

    public void analyzeFiles(List<String> files) {
        for (String file : files) {
            try {
                byte[] source = Files.readAllBytes(Path.of(file));
                Tree tree = parser.parse(source);
                System.out.printf("%s: %d nodes%n",
                    file,
                    tree.getRootNode().getChildCount());
            } catch (Exception e) {
                System.err.printf("Error: %s%n", e.getMessage());
            }
        }
    }
}
```text

### Multi-threaded Usage

Each thread should get its own parser instance:

```java
ExecutorService executor = Executors.newFixedThreadPool(4);

for (String file : files) {
    executor.submit(() -> {
        try {
            Parser parser = TsPackRegistry.getParser("python");
            byte[] source = Files.readAllBytes(Path.of(file));
            Tree tree = parser.parse(source);
            // Process tree
        } catch (Exception e) {
            e.printStackTrace();
        }
    });
}

executor.shutdown();
executor.awaitTermination(1, TimeUnit.MINUTES);
```
