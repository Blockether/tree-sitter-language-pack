//! Structure extraction for source EMBEDDED in another file: the JavaScript /
//! TypeScript inside a Svelte or Vue `<script>` block. The embedded text is
//! parsed with its own grammar and the resulting spans are shifted back into the
//! host file's coordinates, so every span/anchor still addresses the real file.

use crate::intel::types::{ImportInfo, Span, StructureItem};

/// Parse `text` with `language`'s grammar, or `None` when the grammar is
/// unavailable (a build with a reduced language set) or the text does not parse.
fn parse(text: &str, language: &str) -> Option<tree_sitter::Tree> {
    let lang = crate::get_language(language).ok()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang).ok()?;
    parser.parse(text, None)
}

/// Parse `text` as `language` and return its structure translated into host-file
/// coordinates. Returns an empty vec when the grammar is unavailable (a build
/// with a reduced language set) or the text does not parse — never panics.
pub(super) fn embedded_structure(text: &str, language: &str, offset: (usize, usize, usize)) -> Vec<StructureItem> {
    let Some(tree) = parse(text, language) else {
        return Vec::new();
    };
    let mut items = super::for_language(language).structure(&tree.root_node(), text);
    shift_items(&mut items, offset);
    items
}

/// The embedded text's imports, in host-file coordinates — so a `<script>`'s
/// `import … from "…"` is reported as an import OF THE COMPONENT FILE.
pub(super) fn embedded_imports(text: &str, language: &str, offset: (usize, usize, usize)) -> Vec<ImportInfo> {
    let Some(tree) = parse(text, language) else {
        return Vec::new();
    };
    let mut imports = super::for_language(language).imports(&tree.root_node(), text);
    for import in &mut imports {
        shift_span(&mut import.span, offset);
    }
    imports
}

fn shift_items(items: &mut [StructureItem], offset: (usize, usize, usize)) {
    for item in items {
        shift_span(&mut item.span, offset);
        if let Some(body) = item.body_span.as_mut() {
            shift_span(body, offset);
        }
        shift_items(&mut item.children, offset);
    }
}

/// `offset` is `(byte, line, column)` of the embedded text's start. Only spans
/// on the embedded text's FIRST line need the column shift.
fn shift_span(span: &mut Span, offset: (usize, usize, usize)) {
    let (byte, line, column) = offset;
    if span.start_line == 0 {
        span.start_column += column;
    }
    if span.end_line == 0 {
        span.end_column += column;
    }
    span.start_byte += byte;
    span.end_byte += byte;
    span.start_line += line;
    span.end_line += line;
}
