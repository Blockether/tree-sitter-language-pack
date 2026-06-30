// `extract_intelligence` is a convenience wrapper over the individual extraction
// functions. `intel::process` calls those functions directly for fine-grained
// control, but `extract_intelligence` is kept as a public API for consumers.
#![allow(dead_code)]

use super::types::*;

/// Extract all intelligence from a parsed source file.
pub fn extract_intelligence(source: &str, language: &str, tree: &tree_sitter::Tree) -> ProcessResult {
    let root = tree.root_node();
    ProcessResult {
        language: language.to_string(),
        metrics: compute_metrics(source, &root),
        structure: extract_structure(&root, source, language),
        imports: extract_imports(&root, source, language),
        exports: extract_exports(&root, source, language),
        comments: extract_comments(&root, source, language),
        docstrings: extract_docstrings(&root, source, language),
        symbols: extract_symbols(&root, source, language),
        diagnostics: extract_diagnostics(&root, source),
        chunks: Vec::new(),
        data: None,
    }
}

pub(crate) fn span_from_node(node: &tree_sitter::Node) -> Span {
    let start = node.start_position();
    let end = node.end_position();
    Span {
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
        start_line: start.row,
        start_column: start.column,
        end_line: end.row,
        end_column: end.column,
    }
}

pub(crate) fn node_text<'a>(node: &tree_sitter::Node, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

fn go_type_spec_symbol_kind(node: &tree_sitter::Node) -> SymbolKind {
    let ty_kind = node
        .child_by_field_name("type")
        .map(|n| n.kind().to_string())
        .unwrap_or_default();
    match ty_kind.as_str() {
        "struct_type" => SymbolKind::Type,
        "interface_type" => SymbolKind::Interface,
        _ => SymbolKind::Type,
    }
}

pub(crate) fn compute_metrics(source: &str, root: &tree_sitter::Node) -> FileMetrics {
    let mut total_lines = 0usize;
    let mut blank_lines = 0;
    let mut comment_lines = 0;
    for line in source.lines() {
        total_lines += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_lines += 1;
        } else if trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            comment_lines += 1;
        }
    }
    let code_lines = total_lines.saturating_sub(blank_lines + comment_lines);
    let mut node_count = 0;
    let mut error_count = 0;
    let mut max_depth = 0;
    count_nodes(root, 0, &mut node_count, &mut error_count, &mut max_depth);

    FileMetrics {
        total_lines,
        code_lines,
        comment_lines,
        blank_lines,
        total_bytes: source.len(),
        node_count,
        error_count,
        max_depth,
    }
}

fn count_nodes(node: &tree_sitter::Node, depth: usize, count: &mut usize, errors: &mut usize, max_depth: &mut usize) {
    *count += 1;
    if depth > *max_depth {
        *max_depth = depth;
    }
    if node.is_error() || node.is_missing() {
        *errors += 1;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        count_nodes(&child, depth + 1, count, errors, max_depth);
    }
}

pub(crate) fn extract_comments(root: &tree_sitter::Node, source: &str, _language: &str) -> Vec<CommentInfo> {
    let mut comments = Vec::with_capacity(16);
    collect_comments(root, source, &mut comments);
    comments
}

fn collect_comments(node: &tree_sitter::Node, source: &str, comments: &mut Vec<CommentInfo>) {
    let kind = node.kind();
    if kind == "comment"
        || kind == "line_comment"
        || kind == "block_comment"
        || kind == "doc_comment"
        || kind == "documentation_comment"
    {
        let text = node_text(node, source).to_string();
        let comment_kind = if kind == "doc_comment" || kind == "documentation_comment" {
            CommentKind::Doc
        } else if kind == "block_comment" {
            CommentKind::Block
        } else if text.starts_with("///")
            || text.starts_with("//!")
            || text.starts_with("/**")
            || text.starts_with("/*!")
            || text.starts_with("##")
        {
            CommentKind::Doc
        } else {
            CommentKind::Line
        };
        comments.push(CommentInfo {
            text,
            kind: comment_kind,
            span: span_from_node(node),
            associated_node: node.next_named_sibling().map(|n| n.kind().to_string()),
        });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_comments(&child, source, comments);
    }
}

pub(crate) fn extract_docstrings(root: &tree_sitter::Node, source: &str, language: &str) -> Vec<DocstringInfo> {
    super::lang::for_language(language).docstrings(root, source)
}

pub(crate) fn extract_imports(root: &tree_sitter::Node, source: &str, language: &str) -> Vec<ImportInfo> {
    super::lang::for_language(language).imports(root, source)
}

pub(crate) fn extract_exports(root: &tree_sitter::Node, source: &str, language: &str) -> Vec<ExportInfo> {
    super::lang::for_language(language).exports(root, source)
}

pub(crate) fn extract_structure(root: &tree_sitter::Node, source: &str, language: &str) -> Vec<StructureItem> {
    super::lang::for_language(language).structure(root, source)
}

pub(crate) fn extract_symbols(root: &tree_sitter::Node, source: &str, _language: &str) -> Vec<SymbolInfo> {
    let mut symbols = Vec::with_capacity(32);
    collect_symbols(root, source, &mut symbols);
    symbols
}

fn collect_symbols(node: &tree_sitter::Node, source: &str, symbols: &mut Vec<SymbolInfo>) {
    let kind = node.kind();
    let symbol_kind = match kind {
        "function_definition" | "function_declaration" | "function_item" => Some(SymbolKind::Function),
        "class_definition" | "class_declaration" => Some(SymbolKind::Class),
        "type_alias_declaration" | "type_item" => Some(SymbolKind::Type),
        "type_spec" => Some(go_type_spec_symbol_kind(node)),
        "interface_declaration" => Some(SymbolKind::Interface),
        "enum_item" | "enum_declaration" => Some(SymbolKind::Enum),
        "const_item" | "const_declaration" => Some(SymbolKind::Constant),
        "let_declaration" | "variable_declaration" | "lexical_declaration" => Some(SymbolKind::Variable),
        _ => None,
    };
    if let Some(sk) = symbol_kind
        && let Some(name_node) = node.child_by_field_name("name")
    {
        symbols.push(SymbolInfo {
            name: node_text(&name_node, source).to_string(),
            kind: sk,
            span: span_from_node(node),
            type_annotation: node
                .child_by_field_name("type")
                .map(|n| node_text(&n, source).to_string()),
            doc: None,
        });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_symbols(&child, source, symbols);
    }
}

pub(crate) fn extract_diagnostics(root: &tree_sitter::Node, source: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::with_capacity(16);
    collect_diagnostics(root, source, &mut diags);
    diags
}

fn collect_diagnostics(node: &tree_sitter::Node, source: &str, diags: &mut Vec<Diagnostic>) {
    if node.is_error() {
        diags.push(Diagnostic {
            message: format!("Syntax error: unexpected '{}'", node_text(node, source)),
            severity: DiagnosticSeverity::Error,
            span: span_from_node(node),
        });
    } else if node.is_missing() {
        diags.push(Diagnostic {
            message: format!("Missing expected node: {}", node.kind()),
            severity: DiagnosticSeverity::Error,
            span: span_from_node(node),
        });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_diagnostics(&child, source, diags);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: parse source using the global registry (avoids Language lifetime issues).
    fn parse_with_language(source: &str, lang_name: &str) -> Option<(tree_sitter::Language, tree_sitter::Tree)> {
        let registry = crate::LanguageRegistry::new();
        let lang = registry.get_language(lang_name).ok()?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang).ok()?;
        let tree = parser.parse(source, None)?;
        Some((lang, tree))
    }

    fn parse_or_skip(source: &str, lang_name: &str) -> Option<tree_sitter::Tree> {
        parse_with_language(source, lang_name).map(|(_, tree)| tree)
    }

    /// Extract Clojure structure via the cached native grammar, skipping the
    /// test (returning `None`) when the grammar lib is not present in the cache.
    fn clojure_structure_or_skip(source: &str) -> Option<Vec<StructureItem>> {
        let cache = dirs::cache_dir()?.join("tree-sitter-language-pack/v1.10.3/libs");
        let registry = crate::LanguageRegistry::with_libs_dir(cache);
        let lang = registry.get_language("clojure").ok()?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang).ok()?;
        let tree = parser.parse(source, None)?;
        Some(extract_structure(&tree.root_node(), source, "clojure"))
    }

    #[test]
    fn clojure_clean_names_and_visibility() {
        let src = "(def ^:private a 1)\n(def ^{:const true} b 2)\n(def ^{:private true} c 3)\n(defn- helper [x] x)\n(defn pub [y] y)\n(def plain 4)\n";
        let Some(items) = clojure_structure_or_skip(src) else {
            return;
        };
        let names: Vec<_> = items.iter().filter_map(|i| i.name.clone()).collect();
        let by = |n: &str| {
            items
                .iter()
                .find(|i| i.name.as_deref() == Some(n))
                .unwrap_or_else(|| panic!("missing {n}; names were {names:?}"))
        };
        // Names are clean — the `^:private` / `^{...}` metadata is stripped.
        assert_eq!(by("a").kind, StructureKind::Constant);
        assert_eq!(by("a").visibility.as_deref(), Some("private")); // ^:private
        assert_eq!(by("b").visibility.as_deref(), Some("public")); // ^{:const true} is not private
        assert_eq!(by("c").visibility.as_deref(), Some("private")); // ^{:private true}
        assert_eq!(by("helper").kind, StructureKind::Function);
        assert_eq!(by("helper").visibility.as_deref(), Some("private")); // defn-
        assert_eq!(by("pub").visibility.as_deref(), Some("public"));
        assert_eq!(by("plain").visibility.as_deref(), Some("public"));
    }

    #[test]
    fn clojure_docstring_and_signature() {
        let src = "(defn greet \"Say hi to n.\" [n] (str \"hi \" n))\n(def ^:private k \"the limit\" 10)\n(defn multi ([x] x) ([x y] (+ x y)))\n(def noval 1)\n(def justval \"only a value\")\n";
        let Some(items) = clojure_structure_or_skip(src) else {
            return;
        };
        let by = |n: &str| items.iter().find(|i| i.name.as_deref() == Some(n)).unwrap();
        assert_eq!(by("greet").doc_comment.as_deref(), Some("Say hi to n."));
        assert_eq!(by("greet").signature.as_deref(), Some("[n]"));
        // `def` with both a docstring AND a value → the string is the docstring.
        assert_eq!(by("k").doc_comment.as_deref(), Some("the limit"));
        // Multi-arity arglists are joined.
        assert_eq!(by("multi").signature.as_deref(), Some("[x] [x y]"));
        // A bare value (no trailing form) is NOT a docstring.
        assert_eq!(by("noval").doc_comment, None);
        assert_eq!(by("noval").signature, None);
        assert_eq!(by("justval").doc_comment, None);
    }

    // -- Structure extraction tests --

    #[test]
    fn test_extract_python_function() {
        let source = "def foo():\n    pass\n";
        let Some(tree) = parse_or_skip(source, "python") else {
            return;
        };
        let intel = extract_intelligence(source, "python", &tree);

        assert_eq!(intel.language, "python");
        assert!(!intel.structure.is_empty(), "should find at least one structure item");
        let func = &intel.structure[0];
        assert_eq!(func.kind, StructureKind::Function);
        assert_eq!(func.name.as_deref(), Some("foo"));
    }

    #[test]
    fn test_extract_python_class() {
        let source = "class MyClass:\n    def method(self):\n        pass\n";
        let Some(tree) = parse_or_skip(source, "python") else {
            return;
        };
        let intel = extract_intelligence(source, "python", &tree);

        let class = intel.structure.iter().find(|s| s.kind == StructureKind::Class);
        assert!(class.is_some(), "should find a class");
        let class = class.unwrap();
        assert_eq!(class.name.as_deref(), Some("MyClass"));
        assert!(!class.children.is_empty(), "class should have child methods");
        assert_eq!(class.children[0].kind, StructureKind::Function);
        assert_eq!(class.children[0].name.as_deref(), Some("method"));
    }

    #[test]
    fn test_extract_ruby_module_class_and_methods() {
        let source = "module Outer\n  class Widget\n    def call\n      true\n    end\n\n    def self.build\n      new\n    end\n  end\nend\n";
        let Some(tree) = parse_or_skip(source, "ruby") else {
            return;
        };
        let intel = extract_intelligence(source, "ruby", &tree);

        let module = intel.structure.iter().find(|s| s.kind == StructureKind::Module);
        assert!(module.is_some(), "should find a Ruby module entry");
        let module = module.unwrap();
        assert_eq!(module.name.as_deref(), Some("Outer"));

        let class = module.children.iter().find(|s| s.kind == StructureKind::Class);
        assert!(class.is_some(), "should find a Ruby class inside the module");
        let class = class.unwrap();
        assert_eq!(class.name.as_deref(), Some("Widget"));

        let method_names = class
            .children
            .iter()
            .filter(|s| s.kind == StructureKind::Method)
            .filter_map(|s| s.name.as_deref())
            .collect::<Vec<_>>();
        assert!(method_names.contains(&"call"), "should find an instance method");
        assert!(method_names.contains(&"build"), "should find a singleton method");
    }

    #[test]
    fn test_extract_rust_function() {
        let source = "fn main() {\n    let x = 5;\n}\n";
        let Some(tree) = parse_or_skip(source, "rust") else {
            return;
        };
        let intel = extract_intelligence(source, "rust", &tree);

        assert!(!intel.structure.is_empty(), "should find at least one structure item");
        let func = &intel.structure[0];
        assert_eq!(func.kind, StructureKind::Function);
        assert_eq!(func.name.as_deref(), Some("main"));
    }

    // -- Import extraction tests --

    #[test]
    fn test_extract_python_imports() {
        let source = "import os\nfrom sys import path\n";
        let Some(tree) = parse_or_skip(source, "python") else {
            return;
        };
        let intel = extract_intelligence(source, "python", &tree);

        assert_eq!(intel.imports.len(), 2, "should find 2 imports");
        assert!(intel.imports[0].source.contains("import os"));
        assert!(intel.imports[1].source.contains("from sys import path"));
    }

    #[test]
    fn test_extract_rust_imports() {
        let source = "use std::collections::HashMap;\nuse std::io;\n";
        let Some(tree) = parse_or_skip(source, "rust") else {
            return;
        };
        let intel = extract_intelligence(source, "rust", &tree);

        assert_eq!(intel.imports.len(), 2, "should find 2 use declarations");
    }

    // -- Comment extraction tests --

    #[test]
    fn test_extract_comments() {
        let source = "// This is a comment\nfn main() {}\n// Another comment\n";
        let Some(tree) = parse_or_skip(source, "rust") else {
            return;
        };
        let intel = extract_intelligence(source, "rust", &tree);

        assert!(intel.comments.len() >= 2, "should find at least 2 comments");
        assert!(intel.comments[0].text.contains("This is a comment"));
    }

    #[test]
    fn test_extract_doc_comments() {
        let source = "/// Documentation comment\nfn documented() {}\n";
        let Some(tree) = parse_or_skip(source, "rust") else {
            return;
        };
        let intel = extract_intelligence(source, "rust", &tree);

        let doc_comments: Vec<_> = intel.comments.iter().filter(|c| c.kind == CommentKind::Doc).collect();
        assert!(!doc_comments.is_empty(), "should find doc comments");
    }

    // -- Metrics tests --

    #[test]
    fn test_metrics_counts() {
        let source = "fn foo() {}\n\n// comment\nfn bar() {}\n";
        let Some(tree) = parse_or_skip(source, "rust") else {
            return;
        };
        let intel = extract_intelligence(source, "rust", &tree);

        assert!(intel.metrics.total_lines >= 4, "should have at least 4 lines");
        assert!(intel.metrics.blank_lines >= 1, "should have at least 1 blank line");
        assert!(intel.metrics.comment_lines >= 1, "should have at least 1 comment line");
        assert!(intel.metrics.code_lines >= 2, "should have at least 2 code lines");
        assert!(intel.metrics.node_count > 0, "should have nodes");
        assert_eq!(intel.metrics.error_count, 0, "valid code should have 0 errors");
        assert!(intel.metrics.max_depth > 0, "tree should have depth > 0");
        assert_eq!(intel.metrics.total_bytes, source.len());
    }

    // -- Symbol extraction tests --

    #[test]
    fn test_extract_symbols() {
        let source = "fn alpha() {}\nfn beta() {}\n";
        let Some(tree) = parse_or_skip(source, "rust") else {
            return;
        };
        let intel = extract_intelligence(source, "rust", &tree);

        let func_symbols: Vec<_> = intel
            .symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .collect();
        assert!(func_symbols.len() >= 2, "should find at least 2 function symbols");
        let names: Vec<_> = func_symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
    }

    #[test]
    fn test_extract_go_type_declarations_as_symbols() {
        let source = "type User struct{}\ntype Service interface{}\ntype ID string\n";
        let Some(tree) = parse_or_skip(source, "go") else {
            return;
        };
        let intel = extract_intelligence(source, "go", &tree);

        assert!(
            intel
                .symbols
                .iter()
                .any(|s| { s.kind == SymbolKind::Type && s.name == "User" })
        );
        assert!(
            intel
                .symbols
                .iter()
                .any(|s| { s.kind == SymbolKind::Interface && s.name == "Service" })
        );
        assert!(
            intel
                .symbols
                .iter()
                .any(|s| { s.kind == SymbolKind::Type && s.name == "ID" })
        );
    }

    // -- Diagnostics tests --

    #[test]
    fn test_error_nodes_detected() {
        // Use Python with clearly invalid syntax to avoid segfault in some grammars
        let source = "def :\n    pass\n";
        let Some(tree) = parse_or_skip(source, "python") else {
            return;
        };
        let intel = extract_intelligence(source, "python", &tree);

        assert!(
            intel.metrics.error_count > 0,
            "invalid syntax should produce error nodes"
        );
        assert!(!intel.diagnostics.is_empty(), "should have diagnostics for errors");
        assert!(
            intel
                .diagnostics
                .iter()
                .any(|d| d.severity == DiagnosticSeverity::Error)
        );
    }

    #[test]
    fn test_valid_code_no_diagnostics() {
        let source = "def foo():\n    pass\n";
        let Some(tree) = parse_or_skip(source, "python") else {
            return;
        };
        let intel = extract_intelligence(source, "python", &tree);

        assert_eq!(intel.metrics.error_count, 0);
        assert!(intel.diagnostics.is_empty(), "valid code should have no diagnostics");
    }

    // -- Docstring tests --

    #[test]
    #[ignore = "Python grammar node types vary across versions; needs grammar-aware matching"]
    fn test_extract_python_docstrings() {
        let source = "def greet():\n    \"\"\"Say hello.\"\"\"\n    pass\n";
        let Some(tree) = parse_or_skip(source, "python") else {
            return;
        };
        let intel = extract_intelligence(source, "python", &tree);

        assert!(!intel.docstrings.is_empty(), "should find python docstring");
        assert_eq!(intel.docstrings[0].format, DocstringFormat::PythonTripleQuote);
    }

    // -- Language field test --

    #[test]
    fn test_intelligence_language_field() {
        let source = "x = 1";
        let Some(tree) = parse_or_skip(source, "python") else {
            return;
        };
        let intel = extract_intelligence(source, "python", &tree);
        assert_eq!(intel.language, "python");
    }

    // -- Kotlin / Java package and class structure tests (issues #111 and #112) --

    #[test]
    fn collect_structure_kotlin_package_and_class() {
        let source = "package foo.bar\n\nclass Widget {}\n";
        let Some(tree) = parse_or_skip(source, "kotlin") else {
            return;
        };
        let intel = extract_intelligence(source, "kotlin", &tree);

        let module = intel.structure.iter().find(|s| s.kind == StructureKind::Module);
        assert!(module.is_some(), "should find a Module entry for the package header");
        assert_eq!(module.unwrap().name.as_deref(), Some("foo.bar"));

        let class = intel.structure.iter().find(|s| s.kind == StructureKind::Class);
        assert!(class.is_some(), "should find a Class entry");
        assert_eq!(class.unwrap().name.as_deref(), Some("Widget"));
    }

    #[test]
    fn collect_structure_java_package_and_class() {
        let source = "package com.example;\n\npublic class Widget {}\n";
        let Some(tree) = parse_or_skip(source, "java") else {
            return;
        };
        let intel = extract_intelligence(source, "java", &tree);

        let module = intel.structure.iter().find(|s| s.kind == StructureKind::Module);
        assert!(
            module.is_some(),
            "should find a Module entry for the package declaration"
        );
        assert_eq!(module.unwrap().name.as_deref(), Some("com.example"));

        let class = intel.structure.iter().find(|s| s.kind == StructureKind::Class);
        assert!(class.is_some(), "should find a Class entry");
        assert_eq!(class.unwrap().name.as_deref(), Some("Widget"));
    }
}
