---
description: "TypeScript/Node.js API reference for tree-sitter-language-pack"
---

# TypeScript / Node.js API Reference

## Installation

=== "npm"

    ```bash
    npm install @kreuzberg/tree-sitter-language-pack
    ```

=== "pnpm"

    ```bash
    pnpm add @kreuzberg/tree-sitter-language-pack
    ```

=== "yarn"

    ```bash
    yarn add @kreuzberg/tree-sitter-language-pack
    ```

## Quick Start

```typescript
import {
  init,
  getLanguage,
  getParser,
  process,
  ProcessConfig,
} from "@kreuzberg/tree-sitter-language-pack";

// Pre-download languages
await init({
  languages: ["python", "javascript"],
});

// Get a language
const language = await getLanguage("python");

// Get a pre-configured parser
const parser = await getParser("python");
const tree = parser.parse("def hello(): pass");
console.log(tree.rootNode.sexp());

// Extract code intelligence
const config = new ProcessConfig("python")
  .structure()
  .importExports();
const result = process("def hello(): pass", config);
console.log(`Functions: ${result.structure.length}`);
```text

## Download Management

### `init(options?: InitOptions): Promise<void>`

Initialize the language pack with optional pre-downloads.

**Parameters:**

- `options` (InitOptions | undefined):
    - `languages` (string[] | undefined): Languages to download
    - `groups` (string[] | undefined): Language groups to download
    - `cacheDir` (string | undefined): Custom cache directory

**Returns:** Promise<void>

**Throws:** DownloadError if downloads fail

**Example:**

```typescript
import { init } from "@kreuzberg/tree-sitter-language-pack";

// Pre-download specific languages
await init({
  languages: ["python", "rust", "typescript"],
});

// Or download language groups
await init({
  groups: ["web", "data"],
});

// With custom cache directory
await init({
  languages: ["python"],
  cacheDir: "/opt/ts-pack-cache",
});
```text

### `configure(options?: ConfigureOptions): Promise<void>`

Apply configuration without downloading.

**Parameters:**

- `options` (ConfigureOptions | undefined):
    - `cacheDir` (string | undefined): Custom cache directory

**Returns:** Promise<void>

**Throws:** DownloadError if lock cannot be acquired

**Example:**

```typescript
import { configure, getLanguage } from "@kreuzberg/tree-sitter-language-pack";

// Set custom cache before first use
await configure({ cacheDir: "/opt/ts-pack" });

// Now getLanguage uses this cache
const lang = await getLanguage("python");
```text

### `download(names: string[]): Promise<number>`

Download specific languages to cache.

Returns the number of newly downloaded languages.

**Parameters:**

- `names` (string[]): Language names to download

**Returns:** Promise<number> - Count of newly downloaded languages

**Throws:** DownloadError or LanguageNotFoundError

**Example:**

```typescript
import { download } from "@kreuzberg/tree-sitter-language-pack";

const count = await download(["python", "rust", "typescript"]);
console.log(`Downloaded ${count} new languages`);
```text

### `downloadAll(): Promise<number>`

Download all available languages (170+).

Returns the number of newly downloaded languages.

**Returns:** Promise<number> - Count of newly downloaded languages

**Throws:** DownloadError if manifest fetch fails

**Example:**

```typescript
import { downloadAll } from "@kreuzberg/tree-sitter-language-pack";

const count = await downloadAll();
console.log(`Downloaded ${count} languages total`);
```text

### `manifestLanguages(): Promise<string[]>`

Get all available languages from the remote manifest.

Fetches (and caches) the manifest.

**Returns:** Promise<string[]> - Sorted list of available languages

**Throws:** DownloadError if manifest fetch fails

**Example:**

```typescript
import { manifestLanguages } from "@kreuzberg/tree-sitter-language-pack";

const languages = await manifestLanguages();
console.log(`${languages.length} languages available`);
```text

### `downloadedLanguages(): Promise<string[]>`

Get languages already cached locally.

Does not perform network requests.

**Returns:** Promise<string[]> - Sorted list of cached languages

**Example:**

```typescript
import { downloadedLanguages } from "@kreuzberg/tree-sitter-language-pack";

const cached = await downloadedLanguages();
console.log(`Cached: ${cached.join(", ")}`);
```text

### `cleanCache(): Promise<void>`

Delete all cached parser shared libraries.

**Returns:** Promise<void>

**Throws:** DownloadError if cache cannot be removed

**Example:**

```typescript
import { cleanCache } from "@kreuzberg/tree-sitter-language-pack";

await cleanCache();
console.log("Cache cleaned");
```text

### `cacheDir(): Promise<string>`

Get the current cache directory path.

**Returns:** Promise<string> - Absolute path to cache directory

**Example:**

```typescript
import { cacheDir } from "@kreuzberg/tree-sitter-language-pack";

const dir = await cacheDir();
console.log(`Cached at: ${dir}`);
```text

## Language Discovery

### `getLanguage(name: string): Promise<Language>`

Get a tree-sitter Language by name.

Resolves aliases (e.g., `"shell"` → `"bash"`). Auto-downloads if needed.

**Parameters:**

- `name` (string): Language name or alias

**Returns:** Promise<Language> - tree-sitter Language object

**Throws:** LanguageNotFoundError or DownloadError

**Example:**

```typescript
import { getLanguage } from "@kreuzberg/tree-sitter-language-pack";
import Parser from "tree-sitter";

const language = await getLanguage("python");

const parser = new Parser();
parser.setLanguage(language);
const tree = parser.parse("x = 1");
console.log(tree.rootNode.type); // "module"
```text

### `getParser(name: string): Promise<Parser>`

Get a pre-configured Parser for a language.

Calls `getLanguage` and configures a parser in one step.

**Parameters:**

- `name` (string): Language name or alias

**Returns:** Promise<Parser> - Pre-configured tree-sitter Parser

**Throws:** LanguageNotFoundError, DownloadError, or ParseError

**Example:**

```typescript
import { getParser } from "@kreuzberg/tree-sitter-language-pack";

const parser = await getParser("rust");
const tree = parser.parse("fn main() {}");
console.log(tree.rootNode.hasError); // false
```text

### `availableLanguages(): Promise<string[]>`

List all available language names.

**Returns:** Promise<string[]> - Sorted list of language names

**Example:**

```typescript
import { availableLanguages } from "@kreuzberg/tree-sitter-language-pack";

const langs = await availableLanguages();
for (const lang of langs) {
  console.log(lang);
}
```text

### `hasLanguage(name: string): Promise<boolean>`

Check if a language is available.

**Parameters:**

- `name` (string): Language name or alias

**Returns:** Promise<boolean> - True if available

**Example:**

```typescript
import { hasLanguage } from "@kreuzberg/tree-sitter-language-pack";

if (await hasLanguage("python")) {
  console.log("Python is available");
}
```text

### `languageCount(): Promise<number>`

Get total number of available languages.

**Returns:** Promise<number> - Language count

**Example:**

```typescript
import { languageCount } from "@kreuzberg/tree-sitter-language-pack";

const count = await languageCount();
console.log(`${count} languages available`);
```text

## Parsing

### `getBinding(name: string): Promise<TreeHandle>`

Get the low-level binding for a language.

**Parameters:**

- `name` (string): Language name

**Returns:** Promise<TreeHandle> - Low-level binding handle

**Throws:** LanguageNotFoundError

**Example:**

```typescript
import { getBinding } from "@kreuzberg/tree-sitter-language-pack";

const binding = await getBinding("python");
// Use binding for advanced operations
```text

### `parseString(source: string, language: string): Promise<Tree>`

Parse source code into a syntax tree.

**Parameters:**

- `source` (string): Source code to parse
- `language` (string): Language name

**Returns:** Promise<Tree> - Parsed tree

**Throws:** LanguageNotFoundError, ParseError, or DownloadError

**Example:**

```typescript
import { parseString } from "@kreuzberg/tree-sitter-language-pack";

const tree = await parseString("def hello(): pass", "python");
console.log(tree.rootNode.sexp());
```text

## Code Intelligence

### `process(source: string, config: ProcessConfig): Result`

Extract code intelligence from source code.

**Parameters:**

- `source` (string): Source code to analyze
- `config` (ProcessConfig): Analysis configuration

**Returns:** Result - Analysis result object

**Throws:** LanguageNotFoundError, ParseError, or ProcessError

**Example:**

```typescript
import { process, ProcessConfig } from "@kreuzberg/tree-sitter-language-pack";

const config = new ProcessConfig("python")
  .structure()
  .importExports()
  .withChunks(1000, 200);

const result = process("def hello(): pass", config);
console.log(`Functions: ${result.structure.length}`);
console.log(`Total lines: ${result.metrics.totalLines}`);
```text

## Types

### `ProcessConfig`

Configuration for code intelligence analysis.

**Constructor:**

```typescript
new ProcessConfig(language: string)
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

#### `withChunks(maxSize: number, overlap: number): ProcessConfig`

Configure code chunking.

#### `all(): ProcessConfig`

Enable all analysis features.

**Example:**

```typescript
import { ProcessConfig } from "@kreuzberg/tree-sitter-language-pack";

const config = new ProcessConfig("python")
  .structure()
  .importExports()
  .comments()
  .withChunks(2000, 400);
```text

### `Result`

Analysis result from `process()`.

**Properties:**

```typescript
interface Result {
  language: string;
  metrics: FileMetrics;
  structure: StructureItem[];
  imports: ImportInfo[];
  exports: ExportInfo[];
  comments: CommentInfo[];
  docstrings: DocstringInfo[];
  symbols: SymbolInfo[];
  diagnostics: Diagnostic[];
  chunks: CodeChunk[];
  parseErrors: number;
}
```text

**Example:**

```typescript
const result = process(source, config);

// Access different parts
console.log(result.language);
console.log(result.structure[0].kind);
console.log(result.imports.map(i => i.module));
console.log(result.chunks.map(c => c.content));
```text

### `Language`

tree-sitter Language object.

### `Parser`

tree-sitter Parser object.

### `Tree`

Parsed syntax tree with root node.

### `TreeHandle`

Low-level language binding handle.

## Exceptions

### `DownloadError`

Raised when download operations fail.

```typescript
import { download, DownloadError } from "@kreuzberg/tree-sitter-language-pack";

try {
  await download(["python"]);
} catch (e) {
  if (e instanceof DownloadError) {
    console.error(`Download failed: ${e.message}`);
  }
}
```text

### `LanguageNotFoundError`

Raised when language is not recognized.

```typescript
import { getLanguage, LanguageNotFoundError } from "@kreuzberg/tree-sitter-language-pack";

try {
  const lang = await getLanguage("nonexistent");
} catch (e) {
  if (e instanceof LanguageNotFoundError) {
    console.error(`Language not found: ${e.message}`);
  }
}
```text

### `ParseError`

Raised when parsing fails.

```typescript
import { parseString, ParseError } from "@kreuzberg/tree-sitter-language-pack";

try {
  const tree = await parseString("invalid code", "python");
} catch (e) {
  if (e instanceof ParseError) {
    console.error(`Parse error: ${e.message}`);
  }
}
```text

## Usage Patterns

### Pre-download and Cache

```typescript
import { init, getParser } from "@kreuzberg/tree-sitter-language-pack";

// Initialize and pre-download
await init({
  languages: ["python", "rust", "typescript"],
});

// Later, fast access (no network)
const parser = await getParser("python");
```text

### Custom Cache Directory

```typescript
import { configure, getLanguage } from "@kreuzberg/tree-sitter-language-pack";

// Configure before first use
await configure({
  cacheDir: "/data/ts-pack-cache",
});

// Use normally
const lang = await getLanguage("python");
```text

### Analyze Multiple Files

```typescript
import { process, ProcessConfig } from "@kreuzberg/tree-sitter-language-pack";

const config = new ProcessConfig("python").all();

const files = [
  "file1.py",
  "file2.py",
  "file3.py",
];

for (const file of files) {
  const source = await fs.readFile(file, "utf-8");
  const result = process(source, config);
  console.log(`${file}: ${result.structure.length} items`);
}
```text

### Type-Safe Analysis

```typescript
import {
  process,
  ProcessConfig,
  type Result,
} from "@kreuzberg/tree-sitter-language-pack";

function analyzeCode(source: string, lang: string): Result {
  const config = new ProcessConfig(lang).all();
  return process(source, config);
}

const result: Result = analyzeCode("def foo(): pass", "python");
console.log(result.structure);
```
