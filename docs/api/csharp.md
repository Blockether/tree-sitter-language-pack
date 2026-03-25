---
description: "C# / .NET API reference for tree-sitter-language-pack"
---

# C# / .NET API Reference

## Installation

Add to `.csproj`:

```xml
<PackageReference Include="TreeSitterLanguagePack" Version="1.0.0" />
```text

Or via dotnet CLI:

```bash
dotnet add package TreeSitterLanguagePack
```text

## Quick Start

```csharp
using TreeSitterLanguagePack;
using System.Collections.Generic;

class Program
{
    static async Task Main(string[] args)
    {
        // Pre-download languages
        await TsPackClient.Download(new[] { "python", "rust" });

        // Get a language
        var language = await TsPackClient.GetLanguage("python");

        // Get a pre-configured parser
        var parser = await TsPackClient.GetParser("python");
        var tree = parser.Parse("def hello(): pass");
        Console.WriteLine(tree.RootNode.Sexp);

        // Extract code intelligence
        var config = new ProcessConfig("python").All();
        var result = TsPackClient.Process("def hello(): pass", config);
        Console.WriteLine($"Functions: {result.Structure.Count}");
    }
}
```text

## Download Management

### `TsPackClient.Download(string[] names): Task`

Download specific languages to cache.

**Parameters:**

- `names` (string[]): Language names to download

**Returns:** Task

**Throws:**

- `DownloadException`: If language not found or download fails

**Example:**

```csharp
try
{
    await TsPackClient.Download(new[] { "python", "rust", "typescript" });
}
catch (DownloadException ex)
{
    Console.WriteLine($"Download failed: {ex.Message}");
}
```text

### `TsPackClient.DownloadAll(): Task`

Download all available languages (248).

**Returns:** Task

**Throws:**

- `DownloadException`: If manifest fetch or download fails

**Example:**

```csharp
try
{
    await TsPackClient.DownloadAll();
    Console.WriteLine("All languages downloaded");
}
catch (DownloadException ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}
```text

### `TsPackClient.ManifestLanguages(): Task<List<string>>`

Get all available languages from remote manifest.

**Returns:** Task<List<string>> - Available language names

**Throws:**

- `DownloadException`: If manifest fetch fails

**Example:**

```csharp
try
{
    var languages = await TsPackClient.ManifestLanguages();
    Console.WriteLine($"Available: {languages.Count} languages");
}
catch (DownloadException ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}
```text

### `TsPackClient.DownloadedLanguages(): List<string>`

Get languages already cached locally.

Does not perform network requests.

**Returns:** List<string> - Cached language names

**Example:**

```csharp
var cached = TsPackClient.DownloadedLanguages();
foreach (var lang in cached)
{
    Console.WriteLine(lang);
}
```text

### `TsPackClient.CleanCache(): Task`

Delete all cached parser shared libraries.

**Returns:** Task

**Throws:**

- `DownloadException`: If cache cannot be removed

**Example:**

```csharp
try
{
    await TsPackClient.CleanCache();
    Console.WriteLine("Cache cleaned");
}
catch (DownloadException ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}
```text

### `TsPackClient.CacheDir(): string`

Get the current cache directory path.

**Returns:** string - Absolute cache directory path

**Example:**

```csharp
var dir = TsPackClient.CacheDir();
Console.WriteLine($"Cache at: {dir}");
```text

### `TsPackClient.Init(string[]? languages, string? cacheDir): Task`

Initialize with optional pre-downloads and cache directory.

**Parameters:**

- `languages` (string[]?): Languages to download (null = skip)
- `cacheDir` (string?): Custom cache directory (null = default)

**Returns:** Task

**Throws:**

- `DownloadException`: If configuration or download fails

**Example:**

```csharp
try
{
    await TsPackClient.Init(
        languages: new[] { "python", "javascript" },
        cacheDir: "/opt/ts-pack"
    );
}
catch (DownloadException ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}
```text

### `TsPackClient.Configure(string? cacheDir): Task`

Apply configuration without downloading.

**Parameters:**

- `cacheDir` (string?): Custom cache directory (null = default)

**Returns:** Task

**Throws:**

- `DownloadException`: If lock cannot be acquired

**Example:**

```csharp
try
{
    await TsPackClient.Configure(cacheDir: "/data/ts-pack");
}
catch (DownloadException ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}
```text

## Language Discovery

### `TsPackClient.GetLanguage(string name): Task<Language>`

Get a tree-sitter Language by name.

Resolves aliases (e.g., `"shell"` → `"bash"`). Auto-downloads if needed.

**Parameters:**

- `name` (string): Language name or alias

**Returns:** Task<Language> - tree-sitter Language object

**Throws:**

- `LanguageNotFoundException`: If language not recognized
- `DownloadException`: If auto-download fails

**Example:**

```csharp
try
{
    var language = await TsPackClient.GetLanguage("python");
    var parser = new Parser();
    parser.SetLanguage(language);
    var tree = parser.Parse("x = 1");
    Console.WriteLine(tree.RootNode.Type); // "module"
}
catch (LanguageNotFoundException ex)
{
    Console.WriteLine($"Language not found: {ex.Message}");
}
```text

### `TsPackClient.GetParser(string name): Task<Parser>`

Get a pre-configured Parser for a language.

**Parameters:**

- `name` (string): Language name or alias

**Returns:** Task<Parser> - Pre-configured tree-sitter Parser

**Throws:**

- `LanguageNotFoundException`: If language not recognized
- `DownloadException`: If auto-download fails
- `ParserException`: If parser setup fails

**Example:**

```csharp
try
{
    var parser = await TsPackClient.GetParser("rust");
    var tree = parser.Parse("fn main() {}");
    Console.WriteLine(!tree.HasError); // true
}
catch (Exception ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}
```text

### `TsPackClient.AvailableLanguages(): List<string>`

List all available language names.

**Returns:** List<string> - Sorted language names

**Example:**

```csharp
var languages = TsPackClient.AvailableLanguages();
foreach (var lang in languages)
{
    Console.WriteLine(lang);
}
```text

### `TsPackClient.HasLanguage(string name): bool`

Check if a language is available.

**Parameters:**

- `name` (string): Language name or alias

**Returns:** bool - True if available

**Example:**

```csharp
if (TsPackClient.HasLanguage("python"))
{
    Console.WriteLine("Python available");
}
Debug.Assert(TsPackClient.HasLanguage("shell")); // alias for bash
```text

### `TsPackClient.LanguageCount(): int`

Get total number of available languages.

**Returns:** int - Language count

**Example:**

```csharp
int count = TsPackClient.LanguageCount();
Console.WriteLine($"{count} languages available");
```text

## Parsing

### `TsPackClient.Parse(byte[] source, string language): Tree`

Parse source code into a syntax tree.

**Parameters:**

- `source` (byte[]): Source code bytes
- `language` (string): Language name

**Returns:** Tree - Parsed syntax tree

**Throws:**

- `LanguageNotFoundException`: If language not found
- `ParseException`: If parsing fails

**Example:**

```csharp
try
{
    var tree = TsPackClient.Parse("def foo(): pass"u8, "python");
    Console.WriteLine(tree.RootNode.Sexp);
}
catch (Exception ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}
```text

## Code Intelligence

### `TsPackClient.Process(string source, ProcessConfig config): ProcessResult`

Extract code intelligence from source code.

**Parameters:**

- `source` (string): Source code
- `config` (ProcessConfig): Configuration

**Returns:** ProcessResult - Analysis result

**Throws:**

- `LanguageNotFoundException`: If language not found
- `ParseException`: If parsing fails
- `ProcessException`: If analysis fails

**Example:**

```csharp
try
{
    var config = new ProcessConfig("python").All();
    var result = TsPackClient.Process("def hello(): pass", config);

    Console.WriteLine($"Functions: {result.Structure.Count}");
    Console.WriteLine($"Lines: {result.Metrics.TotalLines}");
}
catch (Exception ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}
```text

## Types

### `ProcessConfig`

Configuration for code intelligence analysis.

**Constructor:**

```csharp
var config = new ProcessConfig("python");
```text

**Methods:**

#### `Structure(): ProcessConfig`

Enable structure extraction.

#### `ImportExports(): ProcessConfig`

Enable imports/exports extraction.

#### `Comments(): ProcessConfig`

Enable comment extraction.

#### `Docstrings(): ProcessConfig`

Enable docstring extraction.

#### `Symbols(): ProcessConfig`

Enable symbol extraction.

#### `Metrics(): ProcessConfig`

Enable metric extraction.

#### `Diagnostics(): ProcessConfig`

Enable diagnostic extraction.

#### `WithChunks(int maxSize, int overlap): ProcessConfig`

Configure code chunking.

#### `All(): ProcessConfig`

Enable all features.

**Example:**

```csharp
var config = new ProcessConfig("python")
    .Structure()
    .ImportExports()
    .Comments()
    .WithChunks(2000, 400);
```text

### `ProcessResult`

Result from code intelligence analysis.

**Properties:**

```csharp
public class ProcessResult
{
    public string Language { get; }
    public FileMetrics Metrics { get; }
    public List<StructureItem> Structure { get; }
    public List<ImportInfo> Imports { get; }
    public List<ExportInfo> Exports { get; }
    public List<CommentInfo> Comments { get; }
    public List<DocstringInfo> Docstrings { get; }
    public List<SymbolInfo> Symbols { get; }
    public List<Diagnostic> Diagnostics { get; }
    public List<CodeChunk> Chunks { get; }
    public int ParseErrors { get; }
}
```text

**Example:**

```csharp
var result = TsPackClient.Process(source, config);

Console.WriteLine($"Language: {result.Language}");
foreach (var item in result.Structure)
{
    Console.WriteLine($"  {item.Kind}: {item.Name}");
}
```text

### `Language`

tree-sitter Language object.

**Properties:**

- `string Name { get; }` - Get language name

**Methods:**

- `Parser CreateParser()` - Create a new parser

### `Parser`

tree-sitter Parser object.

**Methods:**

- `Tree Parse(byte[] source)` - Parse source code
- `Tree Parse(byte[] source, Tree? oldTree)` - Parse with incremental update
- `void SetTimeoutMicros(ulong micros)` - Set parse timeout

### `Tree`

Parsed syntax tree.

**Properties:**

- `Node RootNode { get; }` - Get root node

**Methods:**

- `Tree Copy()` - Copy tree

### `Node`

Syntax tree node.

**Properties:**

- `string Type { get; }` - Get node type
- `string Kind { get; }` - Get node kind
- `Point StartPoint { get; }` - Get start position
- `Point EndPoint { get; }` - Get end position
- `int ChildCount { get; }` - Get number of children
- `string Sexp { get; }` - Get S-expression

**Methods:**

- `string GetText(byte[] source)` - Get node text
- `Node? GetChild(int index)` - Get child node

## Exception Handling

```csharp
using TreeSitterLanguagePack;

try
{
    var language = await TsPackClient.GetLanguage("python");
    var parser = new Parser();
    parser.SetLanguage(language);
    var tree = parser.Parse("x = 1");
}
catch (LanguageNotFoundException ex)
{
    Console.WriteLine("Language not available");
}
catch (DownloadException ex)
{
    Console.WriteLine("Download failed");
}
catch (ParseException ex)
{
    Console.WriteLine("Parse error");
}
catch (Exception ex)
{
    Console.WriteLine("Unexpected error");
}
```text

## Usage Patterns

### Pre-download Languages

```csharp
public class AppConfiguration
{
    public static async Task Initialize()
    {
        try
        {
            await TsPackClient.Download(new[]
            {
                "python", "rust", "typescript"
            });
        }
        catch (DownloadException ex)
        {
            throw new InvalidOperationException(
                "Failed to download languages", ex);
        }
    }
}

class Program
{
    static async Task Main()
    {
        await AppConfiguration.Initialize();

        // Fast, no network required
        var parser = await TsPackClient.GetParser("python");
        // ...
    }
}
```text

### Custom Cache Directory

```csharp
try
{
    await TsPackClient.Configure(cacheDir: "/opt/ts-pack-cache");
}
catch (DownloadException ex)
{
    throw new InvalidOperationException("Configuration failed", ex);
}
```text

### Batch Processing

```csharp
public class CodeAnalyzer
{
    private readonly Parser _parser;

    public CodeAnalyzer(string language)
    {
        _parser = TsPackClient.GetParser(language).Result;
    }

    public void AnalyzeFiles(List<string> files)
    {
        foreach (var file in files)
        {
            try
            {
                var source = File.ReadAllBytes(file);
                var tree = _parser.Parse(source);
                Console.WriteLine($"{file}: {tree.RootNode.ChildCount} nodes");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Error: {ex.Message}");
            }
        }
    }
}
```text

### Async Processing

```csharp
public class AsyncAnalyzer
{
    public static async Task AnalyzeFilesAsync(
        List<string> files,
        string language)
    {
        var parser = await TsPackClient.GetParser(language);

        var tasks = files.Select(async file =>
        {
            var source = await File.ReadAllBytesAsync(file);
            var tree = parser.Parse(source);
            Console.WriteLine($"{file}: analyzed");
        });

        await Task.WhenAll(tasks);
    }
}
```text

### Type-Safe Code Intelligence

```csharp
public static class ProcessingExtensions
{
    public static ProcessResult AnalyzeCode(
        this string source,
        string language)
    {
        var config = new ProcessConfig(language).All();
        return TsPackClient.Process(source, config);
    }
}

// Usage
var result = "def foo(): pass".AnalyzeCode("python");
Console.WriteLine($"Functions: {result.Structure.Count}");
```
