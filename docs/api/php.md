---
description: "PHP API reference for tree-sitter-language-pack"
---

# PHP API Reference

## Installation

Install via Composer:

```bash
composer require kreuzberg/tree-sitter-language-pack
```text

## Quick Start

```php
<?php
require_once 'vendor/autoload.php';

use Kreuzberg\TreeSitterLanguagePack\LanguagePack;
use Kreuzberg\TreeSitterLanguagePack\ProcessConfig;

// Pre-download languages
LanguagePack::init(['python', 'rust']);

// Get a language
$language = LanguagePack::getLanguage('python');

// Get a pre-configured parser
$parser = LanguagePack::getParser('python');
$tree = $parser->parse('def hello(): pass');
echo $tree->rootNode()->sexp();

// Extract code intelligence
$config = (new ProcessConfig('python'))->all();
$result = LanguagePack::process('def hello(): pass', $config);
echo count($result['structure']) . ' functions';
```text

## Download Management

### `LanguagePack::download(string[] $names): int`

Download specific languages to cache.

**Parameters:**

- `$names` (string[]): Language names to download

**Returns:** int - Count of newly downloaded languages

**Throws:**

- `DownloadException`: If language not found or download fails
- `LanguageNotFoundError`: If language not in manifest

**Example:**

```php
try {
    $count = LanguagePack::download(['python', 'rust', 'typescript']);
    echo "Downloaded $count new languages";
} catch (DownloadException $e) {
    echo "Download failed: " . $e->getMessage();
}
```text

### `LanguagePack::downloadAll(): int`

Download all available languages (248).

**Returns:** int - Count of newly downloaded languages

**Throws:**

- `DownloadException`: If manifest fetch fails

**Example:**

```php
$count = LanguagePack::downloadAll();
echo "Downloaded $count languages total";
```text

### `LanguagePack::manifestLanguages(): string[]`

Get all available languages from remote manifest.

**Returns:** string[] - Sorted language names

**Throws:**

- `DownloadException`: If manifest fetch fails

**Example:**

```php
$languages = LanguagePack::manifestLanguages();
echo 'Available: ' . count($languages) . ' languages';
```text

### `LanguagePack::downloadedLanguages(): string[]`

Get languages already cached locally.

**Returns:** string[] - Cached language names

**Example:**

```php
$cached = LanguagePack::downloadedLanguages();
foreach ($cached as $lang) {
    echo $lang . PHP_EOL;
}
```text

### `LanguagePack::cleanCache(): void`

Delete all cached parser shared libraries.

**Throws:**

- `DownloadException`: If cache cannot be removed

**Example:**

```php
LanguagePack::cleanCache();
echo 'Cache cleaned';
```text

### `LanguagePack::cacheDir(): string`

Get the current cache directory path.

**Returns:** string - Absolute cache directory path

**Example:**

```php
$dir = LanguagePack::cacheDir();
echo "Cache at: $dir";
```text

### `LanguagePack::init(?string[] $languages = null, ?string $cacheDir = null): void`

Initialize with optional pre-downloads and cache directory.

**Parameters:**

- `$languages` (string[] | null): Languages to download
- `$cacheDir` (string | null): Custom cache directory

**Throws:**

- `DownloadException`: If configuration or download fails

**Example:**

```php
LanguagePack::init(
    languages: ['python', 'javascript'],
    cacheDir: '/opt/ts-pack'
);
```text

### `LanguagePack::configure(?string $cacheDir = null): void`

Apply configuration without downloading.

**Parameters:**

- `$cacheDir` (string | null): Custom cache directory

**Example:**

```php
LanguagePack::configure(cacheDir: '/data/ts-pack');
$language = LanguagePack::getLanguage('python');
```text

## Language Discovery

### `LanguagePack::getLanguage(string $name): Language`

Get a tree-sitter Language by name.

Resolves aliases. Auto-downloads if needed.

**Parameters:**

- `$name` (string): Language name or alias

**Returns:** Language - tree-sitter Language object

**Throws:**

- `LanguageNotFoundError`: If language not recognized
- `DownloadException`: If auto-download fails

**Example:**

```php
try {
    $language = LanguagePack::getLanguage('python');
    $parser = new TreeSitter\Parser();
    $parser->setLanguage($language);
    $tree = $parser->parse('x = 1');
    echo $tree->rootNode()->type(); // 'module'
} catch (LanguageNotFoundError $e) {
    echo "Language not found: " . $e->getMessage();
}
```text

### `LanguagePack::getParser(string $name): Parser`

Get a pre-configured Parser for a language.

**Parameters:**

- `$name` (string): Language name or alias

**Returns:** Parser - Pre-configured tree-sitter Parser

**Example:**

```php
$parser = LanguagePack::getParser('rust');
$tree = $parser->parse('fn main() {}');
echo !$tree->rootNode()->hasError(); // true
```text

### `LanguagePack::availableLanguages(): string[]`

List all available language names.

**Returns:** string[] - Sorted language names

**Example:**

```php
$langs = LanguagePack::availableLanguages();
foreach ($langs as $lang) {
    echo $lang . PHP_EOL;
}
```text

### `LanguagePack::hasLanguage(string $name): bool`

Check if a language is available.

**Parameters:**

- `$name` (string): Language name or alias

**Returns:** bool - True if available

**Example:**

```php
if (LanguagePack::hasLanguage('python')) {
    echo 'Python available';
}
assert(LanguagePack::hasLanguage('shell')); // alias for bash
```text

### `LanguagePack::languageCount(): int`

Get total number of available languages.

**Returns:** int - Language count

**Example:**

```php
$count = LanguagePack::languageCount();
echo "$count languages available";
```text

## Parsing

### `LanguagePack::parseString(string $source, string $language): Tree`

Parse source code into a syntax tree.

**Parameters:**

- `$source` (string): Source code
- `$language` (string): Language name

**Returns:** Tree - Parsed syntax tree

**Example:**

```php
$tree = LanguagePack::parseString('def foo(): pass', 'python');
echo $tree->rootNode()->sexp();
```text

## Code Intelligence

### `LanguagePack::process(string $source, ProcessConfig $config): array`

Extract code intelligence from source code.

**Parameters:**

- `$source` (string): Source code
- `$config` (ProcessConfig): Configuration

**Returns:** array - Result with structure, imports, exports, etc.

**Example:**

```php
$config = (new ProcessConfig('python'))
    ->structure()
    ->importExports()
    ->withChunks(2000, 400);

$result = LanguagePack::process('def hello(): pass', $config);
echo count($result['structure']) . ' functions';
echo $result['metrics']['total_lines'] . ' lines';
```text

## Types

### `ProcessConfig`

Configuration for code intelligence analysis.

**Constructor:**

```php
$config = new ProcessConfig('python');
```text

**Methods:**

- `structure(): self` - Enable structure extraction
- `importExports(): self` - Enable imports/exports extraction
- `comments(): self` - Enable comment extraction
- `docstrings(): self` - Enable docstring extraction
- `symbols(): self` - Enable symbol extraction
- `metrics(): self` - Enable metric extraction
- `diagnostics(): self` - Enable diagnostic extraction
- `withChunks(int $maxSize, int $overlap): self` - Configure chunking
- `all(): self` - Enable all features

**Example:**

```php
$config = (new ProcessConfig('python'))
    ->structure()
    ->importExports()
    ->comments()
    ->withChunks(2000, 400);
```text

### Result Array

**Keys:**

- `'language'` (string) - Language name
- `'metrics'` (array) - File metrics
    - `'total_lines'` (int)
    - `'code_lines'` (int)
    - `'comment_lines'` (int)
    - `'blank_lines'` (int)
- `'structure'` (array) - Code structure items
- `'imports'` (array) - Import statements
- `'exports'` (array) - Export statements
- `'comments'` (array) - Comments
- `'docstrings'` (array) - Docstrings
- `'symbols'` (array) - Symbols
- `'diagnostics'` (array) - Diagnostics
- `'chunks'` (array) - Code chunks
- `'parse_errors'` (int) - Parse error count

## Exception Handling

```php
use Kreuzberg\TreeSitterLanguagePack\LanguagePack;
use Kreuzberg\TreeSitterLanguagePack\Exception\LanguageNotFoundError;
use Kreuzberg\TreeSitterLanguagePack\Exception\DownloadException;

try {
    $parser = LanguagePack::getParser('python');
    $tree = $parser->parse('x = 1');
} catch (LanguageNotFoundError $e) {
    echo 'Language not found';
} catch (DownloadException $e) {
    echo 'Download failed';
} catch (Exception $e) {
    echo 'Unexpected error';
}
```text

## Usage Patterns

### Pre-download Languages

```php
// config/bootstrap.php
use Kreuzberg\TreeSitterLanguagePack\LanguagePack;

LanguagePack::init(['python', 'rust', 'typescript', 'javascript']);
```text

Then use in your application:

```php
require_once 'config/bootstrap.php';

// Fast, no network required
$parser = LanguagePack::getParser('python');
```text

### Custom Cache Directory

```php
LanguagePack::configure(cacheDir: '/data/ts-pack-cache');
$language = LanguagePack::getLanguage('python');
```text

### Batch Processing

```php
function analyzeFiles(string $dir, string $language): void {
    $config = (new ProcessConfig($language))->all();

    $files = glob("$dir/**/*." . (
        $language === 'python' ? 'py' : $language
    ));

    foreach ($files as $file) {
        try {
            $source = file_get_contents($file);
            $result = LanguagePack::process($source, $config);
            echo "$file: " . count($result['structure']) . " items\n";
        } catch (Exception $e) {
            echo "Error: {$e->getMessage()}\n";
        }
    }
}

analyzeFiles('./src', 'python');
```text

### Laravel Integration

```php
// app/Services/CodeAnalyzer.php
namespace App\Services;

use Kreuzberg\TreeSitterLanguagePack\LanguagePack;
use Kreuzberg\TreeSitterLanguagePack\ProcessConfig;

class CodeAnalyzer {
    public function analyze(string $source, string $language): array {
        $config = (new ProcessConfig($language))->all();
        return LanguagePack::process($source, $config);
    }
}

// In controller
$analyzer = new \App\Services\CodeAnalyzer();
$result = $analyzer->analyze($source, 'python');
```text

### Streaming Large Files

```php
function analyzeStreamedFile(string $path, string $language): void {
    $config = (new ProcessConfig($language))->all();

    $chunks = str_split(file_get_contents($path), 1024 * 100);
    $accumulated = '';

    foreach ($chunks as $chunk) {
        $accumulated .= $chunk;

        try {
            $result = LanguagePack::process($accumulated, $config);
            echo "Chunk: " . count($result['structure']) . " items\n";
        } catch (Exception $e) {
            echo "Error: {$e->getMessage()}\n";
        }
    }
}
```text

### Extract Specific Patterns

```php
function findFunctions(string $source, string $language): array {
    $config = (new ProcessConfig($language))->structure();
    $result = LanguagePack::process($source, $config);

    return array_filter(
        $result['structure'],
        fn($item) => $item['kind'] === 'function'
    );
}

$functions = findFunctions(file_get_contents('code.py'), 'python');
foreach ($functions as $func) {
    echo $func['name'] . PHP_EOL;
}
```text

### Type Hints

```php
/**
 * @param string $source Source code to analyze
 * @param string $language Language name
 * @return array Analysis result with structure, imports, etc.
 */
function analyzeCode(string $source, string $language): array {
    $config = (new ProcessConfig($language))->all();
    return LanguagePack::process($source, $config);
}
```
