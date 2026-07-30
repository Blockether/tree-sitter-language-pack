//! Groovy / Gradle. The upstream grammar (`Decodetalkers/tree-sitter-groovy`) is
//! deliberately loose: it has no `class_declaration` or `method_declaration` —
//! everything is `command` / `unit` / `block` / `func`. Structure is therefore
//! recovered from the SHAPES that grammar produces:
//!
//! * `class Widget … { … }` → a `command` whose first `unit` is the keyword
//!   `class`/`interface`/`trait`/`enum`. The type NAME is either the following
//!   `unit` (class) or, when the grammar folds it in, the first `unit` inside
//!   the `block` (interface/trait/enum).
//! * `String render() { … }`, `Widget(String n) { … }`, `def helper(x) { … }` →
//!   a `block` whose FIRST child is a `unit` wrapping a `func`. That shape is
//!   what separates a declaration from a plain call such as `mavenCentral()`,
//!   which has no `block` of its own.
//! * inside a type body only: `String render()` (abstract method — a trailing
//!   `func` with no block) and `int size = 1` (field).
//!
//! Being shape-driven this is a heuristic, so it is kept deliberately narrow:
//! member rules fire only inside a type body, which keeps Gradle DSL blocks
//! (`plugins { … }`, `repositories { mavenCentral() }`) out of the results.

use super::LanguageIntel;
use crate::intel::intelligence::{node_text, span_from_node};
use crate::intel::types::{StructureItem, StructureKind};
use tree_sitter::Node;

pub(crate) struct Groovy;

fn type_keyword(text: &str) -> Option<StructureKind> {
    Some(match text {
        "class" => StructureKind::Class,
        "interface" | "@interface" => StructureKind::Interface,
        "trait" => StructureKind::Trait,
        "enum" => StructureKind::Enum,
        _ => return None,
    })
}

/// `helper(String s)` → `helper`; a bare identifier is returned unchanged.
fn callable_name(text: &str) -> String {
    text.split('(').next().unwrap_or(text).trim().to_string()
}

fn is_plain_identifier(node: &Node) -> bool {
    node.kind() == "unit"
        && node.named_child_count() == 1
        && node.named_child(0).is_some_and(|c| c.kind() == "identifier")
}

/// The `func` wrapped in the first `unit` of `block`, i.e. the declaration head.
fn block_head_func<'t>(block: &Node<'t>) -> Option<Node<'t>> {
    let first = block.named_child(0)?;
    if first.kind() != "unit" {
        return None;
    }
    let mut cursor = first.walk();
    first.named_children(&mut cursor).find(|c| c.kind() == "func")
}

fn func_name(func: &Node, source: &str) -> Option<String> {
    func.named_child(0)
        .filter(|c| c.kind() == "identifier")
        .map(|c| node_text(&c, source).to_string())
}

fn item(
    kind: StructureKind,
    name: Option<String>,
    node: &Node,
    body: Option<&Node>,
    children: Vec<StructureItem>,
) -> StructureItem {
    StructureItem {
        kind,
        name,
        visibility: None,
        span: span_from_node(node),
        children,
        decorators: Vec::new(),
        doc_comment: None,
        signature: None,
        body_span: body.map(span_from_node),
    }
}

impl LanguageIntel for Groovy {
    fn structure(&self, root: &Node, source: &str) -> Vec<StructureItem> {
        let mut items = Vec::with_capacity(16);
        walk(root, source, None, true, &mut items);
        items
    }

    /// `import groovy.transform.CompileStatic` is a `command` whose first unit
    /// is the `import` keyword — indistinguishable by node kind alone.
    fn is_import_node(&self, node: &Node, source: &str) -> bool {
        node.kind() == "command"
            && node
                .named_child(0)
                .is_some_and(|u| u.kind() == "unit" && node_text(&u, source) == "import")
    }
}

/// `enclosing` is `Some(type name)` while walking a class/interface/trait/enum
/// body — the member rules fire only there. `top` marks the file's own top level,
/// where the Gradle-DSL rules (configuration blocks, `version = '…'`) apply; they
/// are deliberately confined there so ordinary Groovy statement bodies stay out.
fn walk(node: &Node, source: &str, enclosing: Option<&str>, top: bool, items: &mut Vec<StructureItem>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "command" || child.kind() == "end_command" {
            if let Some(item) = type_declaration(&child, source) {
                items.push(item);
                continue;
            }
            if let Some(item) = callable_declaration(&child, source, enclosing) {
                items.push(item);
                continue;
            }
            if let Some(name) = enclosing
                && let Some(item) = member(&child, source, name)
            {
                items.push(item);
                continue;
            }
            if top
                && enclosing.is_none()
                && let Some(item) = gradle_entry(&child, source)
            {
                items.push(item);
                continue;
            }
        }
        walk(&child, source, enclosing, false, items);
    }
}

/// Top-level Gradle DSL: a configuration block (`plugins { … }`, `dependencies { … }`,
/// `task hello { … }`) or a project property assignment (`version = '1.0.0'`).
fn gradle_entry(command: &Node, source: &str) -> Option<StructureItem> {
    let mut cursor = command.walk();
    let kids: Vec<Node> = command.named_children(&mut cursor).collect();
    if let Some(block) = kids.iter().find(|n| n.kind() == "block") {
        // `plugins {` folds the name into the block; `task hello {` keeps the
        // leading `task` keyword outside it, and both parts name the entry.
        let mut parts: Vec<String> = kids
            .iter()
            .take_while(|n| n.kind() != "block")
            .filter(|n| is_plain_identifier(n))
            .map(|n| node_text(n, source).to_string())
            .collect();
        if let Some(head) = block.named_child(0).filter(is_plain_identifier) {
            parts.push(callable_name(node_text(&head, source)));
        }
        if parts.is_empty() {
            return None;
        }
        return Some(item(
            StructureKind::Other("block".to_string()),
            Some(parts.join(" ")),
            command,
            Some(block),
            Vec::new(),
        ));
    }
    // `group = 'com.example'`
    let assigns = kids
        .iter()
        .any(|n| n.kind() == "operators" && node_text(n, source) == "=");
    let name = kids.first().filter(|n| is_plain_identifier(n))?;
    assigns.then(|| {
        item(
            StructureKind::Constant,
            Some(node_text(name, source).to_string()),
            command,
            None,
            Vec::new(),
        )
    })
}

/// `class Widget implements Serializable { … }` and friends.
fn type_declaration(command: &Node, source: &str) -> Option<StructureItem> {
    let first = command.named_child(0)?;
    if !is_plain_identifier(&first) {
        return None;
    }
    let kind = type_keyword(node_text(&first, source))?;
    let mut cursor = command.walk();
    let rest: Vec<Node> = command.named_children(&mut cursor).skip(1).collect();
    let block = rest.iter().find(|n| n.kind() == "block").copied();
    // The name is the next `unit` (class) or the block's first `unit` (the
    // grammar folds `interface Renderable {` so the name lands inside the block).
    let name = rest
        .iter()
        .find(|n| is_plain_identifier(n))
        .map(|n| node_text(n, source).to_string())
        .or_else(|| {
            block
                .and_then(|b| b.named_child(0))
                .filter(|u| u.kind() == "unit")
                .map(|u| callable_name(node_text(&u, source)))
        })?;
    let mut children = Vec::new();
    if let Some(block) = &block {
        walk(block, source, Some(&name), false, &mut children);
    }
    Some(item(kind, Some(name), command, block.as_ref(), children))
}

/// A method / constructor / `def` function: a `block` whose head is a `func`.
fn callable_declaration(command: &Node, source: &str, enclosing: Option<&str>) -> Option<StructureItem> {
    let mut cursor = command.walk();
    let block = command.named_children(&mut cursor).find(|c| c.kind() == "block")?;
    let func = block_head_func(&block)?;
    let name = func_name(&func, source)?;
    let kind = match enclosing {
        Some(type_name) if type_name == name => StructureKind::Constructor,
        Some(_) => StructureKind::Method,
        None => StructureKind::Function,
    };
    Some(item(kind, Some(name), command, Some(&block), Vec::new()))
}

/// Inside a type body only: an abstract method (`String render()`) or a field
/// (`int size = 1`, `String name`).
fn member(command: &Node, source: &str, _enclosing: &str) -> Option<StructureItem> {
    let mut cursor = command.walk();
    let kids: Vec<Node> = command.named_children(&mut cursor).collect();
    if kids.len() < 2 {
        return None;
    }
    // Abstract method: a trailing `unit > func`, preceded by the return type.
    let last = kids.last()?;
    if last.kind() == "unit"
        && let Some(func) = {
            let mut c = last.walk();
            last.named_children(&mut c).find(|n| n.kind() == "func")
        }
        && kids[..kids.len() - 1].iter().all(is_plain_identifier)
        && let Some(name) = func_name(&func, source)
    {
        return Some(item(StructureKind::Method, Some(name), command, None, Vec::new()));
    }
    // Field: `Type name` or `Type name = value` — plain identifier units only.
    let idents: Vec<&Node> = kids.iter().take_while(|n| is_plain_identifier(n)).collect();
    if idents.len() >= 2 {
        let name = node_text(idents[idents.len() - 1], source).to_string();
        return Some(item(StructureKind::Field, Some(name), command, None, Vec::new()));
    }
    None
}
