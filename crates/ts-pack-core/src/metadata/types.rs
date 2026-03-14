//! Types for file metadata extracted from source code via tree-sitter.
//!
//! These types represent the structured analysis output from [`crate::analyze`],
//! [`crate::chunk`], and [`crate::process`]. They are serializable via serde
//! when the `serde` feature is enabled.

/// Byte and line/column range in source code.
///
/// Represents both byte offsets and line/column positions for a region of source.
/// Lines and columns are zero-indexed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span {
    /// Start byte offset (inclusive).
    pub start_byte: usize,
    /// End byte offset (exclusive).
    pub end_byte: usize,
    /// Start line number (zero-indexed).
    pub start_line: usize,
    /// Start column number (zero-indexed).
    pub start_column: usize,
    /// End line number (zero-indexed).
    pub end_line: usize,
    /// End column number (zero-indexed).
    pub end_column: usize,
}

/// Complete metadata extracted from a source file.
///
/// Contains structural analysis, imports/exports, metrics, comments, symbols,
/// and diagnostics for a single source file.
///
/// # Example
///
/// ```no_run
/// let metadata = ts_pack_core::analyze("def hello(): pass", "python").unwrap();
/// println!("Language: {}", metadata.language);
/// println!("Total lines: {}", metadata.metrics.total_lines);
/// println!("Structures found: {}", metadata.structure.len());
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileMetadata {
    /// The language used for parsing.
    pub language: String,
    /// Aggregate metrics (line counts, byte size, error count, etc.).
    pub metrics: FileMetrics,
    /// Top-level structural items (functions, classes, structs, etc.).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty", default))]
    pub structure: Vec<StructureItem>,
    /// Import statements found in the source.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty", default))]
    pub imports: Vec<ImportInfo>,
    /// Export statements found in the source.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty", default))]
    pub exports: Vec<ExportInfo>,
    /// Comments extracted from the source.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty", default))]
    pub comments: Vec<CommentInfo>,
    /// Docstrings extracted from the source.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty", default))]
    pub docstrings: Vec<DocstringInfo>,
    /// Symbols (variables, constants, types) extracted from the source.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty", default))]
    pub symbols: Vec<SymbolInfo>,
    /// Diagnostics (syntax errors, missing nodes) from parsing.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty", default))]
    pub diagnostics: Vec<Diagnostic>,
}

/// Aggregate metrics for a source file.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileMetrics {
    /// Total number of lines in the file.
    pub total_lines: usize,
    /// Number of lines containing code (non-blank, non-comment).
    pub code_lines: usize,
    /// Number of lines containing comments.
    pub comment_lines: usize,
    /// Number of blank lines.
    pub blank_lines: usize,
    /// Total byte size of the source.
    pub total_bytes: usize,
    /// Total number of syntax tree nodes.
    pub node_count: usize,
    /// Number of ERROR and MISSING nodes in the syntax tree.
    pub error_count: usize,
    /// Maximum nesting depth of the syntax tree.
    pub max_depth: usize,
}

/// The kind of a structural item in source code.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StructureKind {
    /// A standalone function.
    Function,
    /// A method on a class/struct/impl.
    Method,
    /// A class definition.
    Class,
    /// A struct definition.
    Struct,
    /// An interface or protocol definition.
    Interface,
    /// An enum definition.
    Enum,
    /// A module or namespace declaration.
    Module,
    /// A trait definition (Rust-specific).
    Trait,
    /// An impl block (Rust-specific).
    Impl,
    /// A namespace declaration.
    Namespace,
    /// Any other structural kind not covered above.
    Other(String),
}

/// A structural item (function, class, struct, etc.) in source code.
///
/// Represents a top-level or nested code structure with optional metadata
/// like visibility, decorators, and documentation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructureItem {
    /// The kind of structure (function, class, struct, etc.).
    pub kind: StructureKind,
    /// The name of the item, if it has one.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub name: Option<String>,
    /// Visibility modifier (e.g., "pub", "private", "protected").
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none", default))]
    pub visibility: Option<String>,
    /// Source location of the entire item.
    pub span: Span,
    /// Nested structural items (e.g., methods inside a class).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty", default))]
    pub children: Vec<StructureItem>,
    /// Decorators or attributes applied to this item.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty", default))]
    pub decorators: Vec<String>,
    /// Associated documentation comment text.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none", default))]
    pub doc_comment: Option<String>,
    /// Function/method signature string.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none", default))]
    pub signature: Option<String>,
    /// Source location of the body (excluding signature).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub body_span: Option<Span>,
}

/// The kind of a comment.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommentKind {
    /// A single-line comment (e.g., `//` or `#`).
    Line,
    /// A multi-line block comment (e.g., `/* ... */`).
    Block,
    /// A documentation comment (e.g., `///` or `/** ... */`).
    Doc,
}

/// A comment extracted from source code.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommentInfo {
    /// The text content of the comment.
    pub text: String,
    /// Whether this is a line, block, or doc comment.
    pub kind: CommentKind,
    /// Source location of the comment.
    pub span: Span,
    /// The type of the syntax node this comment is associated with, if any.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub associated_node: Option<String>,
}

/// The format of a docstring.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DocstringFormat {
    /// Python triple-quoted string (`"""..."""`).
    PythonTripleQuote,
    /// JavaScript JSDoc (`/** ... */`).
    JSDoc,
    /// Rust doc comment (`///` or `//!`).
    Rustdoc,
    /// Go documentation comment.
    GoDoc,
    /// Java documentation comment (`/** ... */`).
    JavaDoc,
    /// Any other docstring format.
    Other(String),
}

/// A docstring extracted from source code.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocstringInfo {
    /// The raw text content of the docstring.
    pub text: String,
    /// The documentation format (Python, JSDoc, Rustdoc, etc.).
    pub format: DocstringFormat,
    /// Source location of the docstring.
    pub span: Span,
    /// Name of the item this docstring documents, if determinable.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub associated_item: Option<String>,
    /// Parsed sections (Args, Returns, Raises, etc.).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty", default))]
    pub parsed_sections: Vec<DocSection>,
}

/// A section within a docstring (e.g., Args, Returns, Raises).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocSection {
    /// The section type (e.g., "Args", "Returns", "Raises", "Example").
    pub kind: String,
    /// The parameter or item name, if applicable.
    pub name: Option<String>,
    /// The section description text.
    pub description: String,
}

/// An import statement extracted from source code.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportInfo {
    /// The module or package being imported (e.g., "os", "react", "std::io").
    pub source: String,
    /// Specific items imported (e.g., `from os import path, getcwd`).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty", default))]
    pub items: Vec<String>,
    /// Alias for the import (e.g., `import numpy as np` → alias = "np").
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none", default))]
    pub alias: Option<String>,
    /// Whether this is a wildcard import (e.g., `from os import *`).
    pub is_wildcard: bool,
    /// Source location of the import statement.
    pub span: Span,
}

/// The kind of an export.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExportKind {
    /// A named export (e.g., `export function foo`).
    Named,
    /// A default export (e.g., `export default class`).
    Default,
    /// A re-export from another module (e.g., `export { foo } from './bar'`).
    ReExport,
}

/// An export statement extracted from source code.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExportInfo {
    /// The exported identifier name.
    pub name: String,
    /// Whether this is a named, default, or re-export.
    pub kind: ExportKind,
    /// Source location of the export statement.
    pub span: Span,
}

/// The kind of a symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SymbolKind {
    Variable,
    Constant,
    Function,
    Class,
    Type,
    Interface,
    Enum,
    Module,
    Other(String),
}

/// A symbol (variable, function, type, etc.) extracted from source code.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub type_annotation: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none", default))]
    pub doc: Option<String>,
}

/// Severity of a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

/// A diagnostic (syntax error, missing node, etc.) from parsing.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub span: Span,
}

/// A chunk of source code with rich metadata.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CodeChunk {
    pub content: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub metadata: ChunkInfo,
}

/// Metadata for a single chunk of source code.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChunkInfo {
    pub language: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub node_types: Vec<String>,
    pub context_path: Vec<String>,
    pub symbols_defined: Vec<String>,
    pub comments: Vec<CommentInfo>,
    pub docstrings: Vec<DocstringInfo>,
    pub has_error_nodes: bool,
}

/// Combined result of metadata extraction and chunking.
///
/// Returned by [`crate::process`], which performs both operations in a single pass.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessResult {
    /// File-level metadata (structure, imports, metrics, etc.).
    pub metadata: FileMetadata,
    /// Source code chunks with per-chunk metadata.
    pub chunks: Vec<CodeChunk>,
}
