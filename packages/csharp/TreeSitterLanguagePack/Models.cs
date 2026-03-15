using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace TreeSitterLanguagePack;

/// <summary>
/// Configuration for the <see cref="TsPackClient.Process"/> method.
/// Serialized to JSON before passing to the FFI layer.
/// </summary>
public sealed class ProcessConfig
{
    /// <summary>Language name (required).</summary>
    [JsonPropertyName("language")]
    public required string Language { get; set; }

    /// <summary>Extract structural items (functions, classes, etc.). Default: true.</summary>
    [JsonPropertyName("structure")]
    public bool Structure { get; set; } = true;

    /// <summary>Extract import statements. Default: true.</summary>
    [JsonPropertyName("imports")]
    public bool Imports { get; set; } = true;

    /// <summary>Extract export statements. Default: true.</summary>
    [JsonPropertyName("exports")]
    public bool Exports { get; set; } = true;

    /// <summary>Extract comments. Default: false.</summary>
    [JsonPropertyName("comments")]
    public bool Comments { get; set; }

    /// <summary>Extract docstrings. Default: false.</summary>
    [JsonPropertyName("docstrings")]
    public bool Docstrings { get; set; }

    /// <summary>Extract symbol definitions. Default: false.</summary>
    [JsonPropertyName("symbols")]
    public bool Symbols { get; set; }

    /// <summary>Include parse diagnostics. Default: false.</summary>
    [JsonPropertyName("diagnostics")]
    public bool Diagnostics { get; set; }

    /// <summary>Maximum chunk size in bytes. Null disables chunking.</summary>
    [JsonPropertyName("chunk_max_size")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public int? ChunkMaxSize { get; set; }
}

/// <summary>
/// Complete analysis result from processing a source file.
/// </summary>
public sealed class ProcessResult
{
    /// <summary>The language used for parsing.</summary>
    [JsonPropertyName("language")]
    public string Language { get; set; } = string.Empty;

    /// <summary>Aggregate metrics for the source file.</summary>
    [JsonPropertyName("metrics")]
    public FileMetrics Metrics { get; set; } = new();

    /// <summary>Functions, classes, structs, etc.</summary>
    [JsonPropertyName("structure")]
    public List<StructureItem> Structure { get; set; } = [];

    /// <summary>Import statements.</summary>
    [JsonPropertyName("imports")]
    public List<ImportInfo> Imports { get; set; } = [];

    /// <summary>Export statements.</summary>
    [JsonPropertyName("exports")]
    public List<ExportInfo> Exports { get; set; } = [];

    /// <summary>Comments extracted from source.</summary>
    [JsonPropertyName("comments")]
    public List<CommentInfo> Comments { get; set; } = [];

    /// <summary>Docstrings extracted from source.</summary>
    [JsonPropertyName("docstrings")]
    public List<DocstringInfo> Docstrings { get; set; } = [];

    /// <summary>Symbol definitions.</summary>
    [JsonPropertyName("symbols")]
    public List<SymbolInfo> Symbols { get; set; } = [];

    /// <summary>Parse diagnostics (errors, warnings).</summary>
    [JsonPropertyName("diagnostics")]
    public List<Diagnostic> Diagnostics { get; set; } = [];

    /// <summary>Chunked code segments.</summary>
    [JsonPropertyName("chunks")]
    public List<CodeChunk> Chunks { get; set; } = [];
}

/// <summary>
/// Aggregate metrics for a source file.
/// </summary>
public sealed class FileMetrics
{
    [JsonPropertyName("total_lines")]
    public int TotalLines { get; set; }

    [JsonPropertyName("code_lines")]
    public int CodeLines { get; set; }

    [JsonPropertyName("comment_lines")]
    public int CommentLines { get; set; }

    [JsonPropertyName("blank_lines")]
    public int BlankLines { get; set; }

    [JsonPropertyName("total_bytes")]
    public int TotalBytes { get; set; }

    [JsonPropertyName("node_count")]
    public int NodeCount { get; set; }

    [JsonPropertyName("error_count")]
    public int ErrorCount { get; set; }

    [JsonPropertyName("max_depth")]
    public int MaxDepth { get; set; }
}

/// <summary>
/// Byte and line/column range in source code.
/// </summary>
public sealed class Span
{
    [JsonPropertyName("start_byte")]
    public int StartByte { get; set; }

    [JsonPropertyName("end_byte")]
    public int EndByte { get; set; }

    [JsonPropertyName("start_line")]
    public int StartLine { get; set; }

    [JsonPropertyName("start_column")]
    public int StartColumn { get; set; }

    [JsonPropertyName("end_line")]
    public int EndLine { get; set; }

    [JsonPropertyName("end_column")]
    public int EndColumn { get; set; }
}

/// <summary>
/// A structural item (function, class, struct, etc.) in source code.
/// </summary>
public sealed class StructureItem
{
    [JsonPropertyName("kind")]
    public object Kind { get; set; } = string.Empty;

    [JsonPropertyName("name")]
    public string? Name { get; set; }

    [JsonPropertyName("visibility")]
    public string? Visibility { get; set; }

    [JsonPropertyName("span")]
    public Span Span { get; set; } = new();

    [JsonPropertyName("children")]
    public List<StructureItem> Children { get; set; } = [];

    [JsonPropertyName("decorators")]
    public List<string> Decorators { get; set; } = [];

    [JsonPropertyName("doc_comment")]
    public string? DocComment { get; set; }

    [JsonPropertyName("signature")]
    public string? Signature { get; set; }

    [JsonPropertyName("body_span")]
    public Span? BodySpan { get; set; }
}

/// <summary>
/// An import statement extracted from source code.
/// </summary>
public sealed class ImportInfo
{
    [JsonPropertyName("source")]
    public string Source { get; set; } = string.Empty;

    [JsonPropertyName("items")]
    public List<string> Items { get; set; } = [];

    [JsonPropertyName("alias")]
    public string? Alias { get; set; }

    [JsonPropertyName("is_wildcard")]
    public bool IsWildcard { get; set; }

    [JsonPropertyName("span")]
    public Span Span { get; set; } = new();
}

/// <summary>
/// An export statement extracted from source code.
/// </summary>
public sealed class ExportInfo
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("kind")]
    public object Kind { get; set; } = string.Empty;

    [JsonPropertyName("span")]
    public Span Span { get; set; } = new();
}

/// <summary>
/// A comment extracted from source code.
/// </summary>
public sealed class CommentInfo
{
    [JsonPropertyName("text")]
    public string Text { get; set; } = string.Empty;

    [JsonPropertyName("kind")]
    public object Kind { get; set; } = string.Empty;

    [JsonPropertyName("span")]
    public Span Span { get; set; } = new();

    [JsonPropertyName("associated_node")]
    public string? AssociatedNode { get; set; }
}

/// <summary>
/// A docstring extracted from source code.
/// </summary>
public sealed class DocstringInfo
{
    [JsonPropertyName("text")]
    public string Text { get; set; } = string.Empty;

    [JsonPropertyName("format")]
    public object Format { get; set; } = string.Empty;

    [JsonPropertyName("span")]
    public Span Span { get; set; } = new();

    [JsonPropertyName("associated_item")]
    public string? AssociatedItem { get; set; }

    [JsonPropertyName("parsed_sections")]
    public List<DocSection> ParsedSections { get; set; } = [];
}

/// <summary>
/// A section within a docstring (e.g., Args, Returns, Raises).
/// </summary>
public sealed class DocSection
{
    [JsonPropertyName("kind")]
    public string Kind { get; set; } = string.Empty;

    [JsonPropertyName("name")]
    public string? Name { get; set; }

    [JsonPropertyName("description")]
    public string Description { get; set; } = string.Empty;
}

/// <summary>
/// A symbol (variable, function, type, etc.) extracted from source code.
/// </summary>
public sealed class SymbolInfo
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("kind")]
    public object Kind { get; set; } = string.Empty;

    [JsonPropertyName("span")]
    public Span Span { get; set; } = new();

    [JsonPropertyName("type_annotation")]
    public string? TypeAnnotation { get; set; }

    [JsonPropertyName("doc")]
    public string? Doc { get; set; }
}

/// <summary>
/// A diagnostic (syntax error, missing node, etc.) from parsing.
/// </summary>
public sealed class Diagnostic
{
    [JsonPropertyName("message")]
    public string Message { get; set; } = string.Empty;

    [JsonPropertyName("severity")]
    public object Severity { get; set; } = string.Empty;

    [JsonPropertyName("span")]
    public Span Span { get; set; } = new();
}

/// <summary>
/// A chunk of source code with rich metadata.
/// </summary>
public sealed class CodeChunk
{
    [JsonPropertyName("content")]
    public string Content { get; set; } = string.Empty;

    [JsonPropertyName("start_byte")]
    public int StartByte { get; set; }

    [JsonPropertyName("end_byte")]
    public int EndByte { get; set; }

    [JsonPropertyName("start_line")]
    public int StartLine { get; set; }

    [JsonPropertyName("end_line")]
    public int EndLine { get; set; }

    [JsonPropertyName("metadata")]
    public ChunkContext Metadata { get; set; } = new();
}

/// <summary>
/// Metadata for a single chunk of source code.
/// </summary>
public sealed class ChunkContext
{
    [JsonPropertyName("language")]
    public string Language { get; set; } = string.Empty;

    [JsonPropertyName("chunk_index")]
    public int ChunkIndex { get; set; }

    [JsonPropertyName("total_chunks")]
    public int TotalChunks { get; set; }

    [JsonPropertyName("node_types")]
    public List<string> NodeTypes { get; set; } = [];

    [JsonPropertyName("context_path")]
    public List<string> ContextPath { get; set; } = [];

    [JsonPropertyName("symbols_defined")]
    public List<string> SymbolsDefined { get; set; } = [];

    [JsonPropertyName("comments")]
    public List<CommentInfo> Comments { get; set; } = [];

    [JsonPropertyName("docstrings")]
    public List<DocstringInfo> Docstrings { get; set; } = [];

    [JsonPropertyName("has_error_nodes")]
    public bool HasErrorNodes { get; set; }
}
