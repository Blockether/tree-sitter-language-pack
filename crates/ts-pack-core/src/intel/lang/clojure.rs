//! Clojure (incl. cljs/cljc/bb — all reported as `clojure`).
//!
//! Clojure parses as generic s-expressions (`list_lit`/`sym_lit`), so there are
//! no `*_definition` node kinds to match on: structure is SEMANTIC — a top-level
//! list whose head symbol is a def-form (`defn`, `def`, `defrecord`, …) is the
//! definition. This module therefore overrides `structure` wholesale rather than
//! using the generic node-kind walker.

use super::LanguageIntel;
use crate::intel::intelligence::{node_text, span_from_node};
use crate::intel::types::*;
use tree_sitter::Node;

pub(crate) struct Clojure;

impl LanguageIntel for Clojure {
    fn structure(&self, root: &Node, source: &str) -> Vec<StructureItem> {
        let mut items = Vec::with_capacity(32);
        let mut cursor = root.walk();
        for child in root.named_children(&mut cursor) {
            if let Some(item) = clojure_form(&child, source) {
                items.push(item);
            }
        }
        items
    }

    fn docstring_at(&self, node: &Node, source: &str, out: &mut Vec<DocstringInfo>) {
        // A Clojure doc string is the str_lit right after the name in a def-form,
        // e.g. `(defn f "doc" [x] …)`. It is only a docstring when a form follows
        // it (otherwise `(def x "v")` would misread the value).
        if node.kind() != "list_lit" {
            return;
        }
        let mut cursor = node.walk();
        let kids: Vec<Node> = node.named_children(&mut cursor).collect();
        if kids.len() > 3
            && kids[0].kind() == "sym_lit"
            && kids[1].kind() == "sym_lit"
            && kids[2].kind() == "str_lit"
            && matches!(
                node_text(&kids[0], source),
                "defn" | "defn-" | "defmacro" | "def" | "defonce"
            )
        {
            out.push(DocstringInfo {
                text: node_text(&kids[2], source).to_string(),
                format: DocstringFormat::Plain,
                span: span_from_node(&kids[2]),
                // Clean name (drops reader metadata) so doc lookups by the real
                // def name resolve — matches the structure items.
                associated_item: Some(clojure_sym_name(&kids[1], source).to_string()),
                parsed_sections: Vec::new(),
            });
        }
    }
}

/// Trim and collapse internal whitespace runs to a single space.
fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Map a Clojure def-form head symbol to a structure kind. Non-def-form heads
/// return `None` (the list is not a definition).
fn clojure_def_kind(head: &str) -> Option<StructureKind> {
    match head {
        "defn" | "defn-" | "definline" => Some(StructureKind::Function),
        "defmacro" => Some(StructureKind::Macro),
        "def" | "defonce" => Some(StructureKind::Constant),
        "defmulti" | "defmethod" => Some(StructureKind::Method),
        "defprotocol" => Some(StructureKind::Protocol),
        "definterface" => Some(StructureKind::Interface),
        "defrecord" => Some(StructureKind::Struct),
        "deftype" => Some(StructureKind::Type),
        "ns" => Some(StructureKind::Namespace),
        _ => None,
    }
}

/// The clean symbol name of a `sym_lit` node: the text of its `sym_name` child,
/// which skips any leading reader metadata. In this grammar the metadata is
/// nested *inside* the `sym_lit` (e.g. `(sym_lit (meta_lit "^:private")
/// (sym_name "a"))`), so `node_text` of the `sym_lit` itself would wrongly
/// include `^:private`. Falls back to the full text if no `sym_name` child
/// exists (defensive; the grammar always nests one).
pub(crate) fn clojure_sym_name<'a>(sym_lit: &Node, source: &'a str) -> &'a str {
    let mut cursor = sym_lit.walk();
    for child in sym_lit.named_children(&mut cursor) {
        if child.kind() == "sym_name" {
            return node_text(&child, source);
        }
    }
    node_text(sym_lit, source)
}

/// Whether a name `sym_lit`'s reader metadata marks the def private — either the
/// shorthand `^:private` (a bare `:private` keyword in the `meta_lit`) or the
/// map form `^{:private true}` (a `:private`/`true` key-value pair).
fn clojure_meta_private(sym_lit: &Node, source: &str) -> bool {
    let mut cursor = sym_lit.walk();
    for meta in sym_lit.named_children(&mut cursor) {
        if meta.kind() != "meta_lit" {
            continue;
        }
        let mut mcur = meta.walk();
        for m in meta.named_children(&mut mcur) {
            match m.kind() {
                // `^:private`
                "kwd_lit" if node_text(&m, source) == ":private" => return true,
                // `^{:private true}` — find the `:private` key, check its value is `true`.
                "map_lit" => {
                    let mut kcur = m.walk();
                    let kids: Vec<Node> = m.named_children(&mut kcur).collect();
                    for (i, k) in kids.iter().enumerate() {
                        if k.kind() == "kwd_lit"
                            && node_text(k, source) == ":private"
                            && kids.get(i + 1).map(|v| node_text(v, source)) == Some("true")
                        {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    false
}

/// Visibility of a Clojure def: `private` for `defn-` or a `^:private` /
/// `^{:private true}` metadata marker on the name; `public` otherwise.
fn clojure_visibility(head: &str, name_sym_lit: &Node, source: &str) -> String {
    if head == "defn-" || clojure_meta_private(name_sym_lit, source) {
        "private".to_string()
    } else {
        "public".to_string()
    }
}

/// The arglist signature of a Clojure fn-like form. Single-arity: the first
/// `vec_lit` (the `[params]` vector) among the form's direct children.
/// Multi-arity: each arity is a `list_lit` of shape `([params] …)`, so join the
/// `vec_lit` of each. `None` when there is no arglist.
fn clojure_signature(named: &[Node], source: &str) -> Option<String> {
    if let Some(v) = named.iter().find(|n| n.kind() == "vec_lit") {
        return Some(collapse_ws(node_text(v, source)));
    }
    let arities: Vec<String> = named
        .iter()
        .filter(|n| n.kind() == "list_lit")
        .filter_map(|lst| {
            let mut c = lst.walk();
            lst.named_children(&mut c)
                .find(|n| n.kind() == "vec_lit")
                .map(|v| collapse_ws(node_text(&v, source)))
        })
        .collect();
    (!arities.is_empty()).then(|| arities.join(" "))
}

/// The docstring of a def-form: the `str_lit` right after the name, but only
/// when another form follows it — so `(def x "value")` (no trailing form) is
/// read as a value, while `(defn f "doc" [x] …)` / `(def x "doc" 1)` are docs.
/// `named` is the form's named children: `[head, name, doc?, …]`. Returns the
/// inner text with surrounding quotes stripped.
fn clojure_doc_comment(named: &[Node], source: &str) -> Option<String> {
    if named.len() > 3 && named[2].kind() == "str_lit" {
        let raw = node_text(&named[2], source);
        let inner = raw.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(raw);
        return Some(inner.to_string());
    }
    None
}

/// One Clojure structure item from a `list_lit`, or `None` when the list is not
/// a recognised def-form. Head symbol → kind; the next symbol → name (cleaned of
/// reader metadata). Metadata + a `defn-` head set `visibility`; a trailing
/// docstring sets `doc_comment`; a fn/macro arglist sets `signature`. For
/// `defmethod` the dispatch value is appended to the name (e.g. `area :circle`)
/// so callers can target one method of a multimethod unambiguously.
fn clojure_form(node: &Node, source: &str) -> Option<StructureItem> {
    if node.kind() != "list_lit" {
        return None;
    }
    let mut cursor = node.walk();
    let named: Vec<Node> = node.named_children(&mut cursor).collect();
    // Head symbol identifies the def-form.
    let head_node = named.first()?;
    if head_node.kind() != "sym_lit" {
        return None;
    }
    let head = node_text(head_node, source);
    let kind = clojure_def_kind(head)?;
    // The defined name is the next symbol; its clean text is the `sym_name`
    // child (the grammar nests reader metadata like `^:private` inside the
    // `sym_lit`). The same metadata, plus a `defn-` head, gives visibility.
    let name_node = named.iter().skip(1).find(|n| n.kind() == "sym_lit");
    let visibility = name_node.map(|n| clojure_visibility(head, n, source));
    let name = name_node.map(|n| {
        let base = clojure_sym_name(n, source).to_string();
        if head == "defmethod" {
            // Append the dispatch value (the form right after the name) so two
            // (defmethod area :circle …) / (defmethod area :rect …) are distinct.
            let dispatch = named
                .iter()
                .skip_while(|x| x.id() != n.id())
                .nth(1)
                .map(|d| collapse_ws(node_text(d, source)));
            match dispatch {
                Some(d) if !d.is_empty() => format!("{base} {d}"),
                _ => base,
            }
        } else {
            base
        }
    });
    let doc_comment = clojure_doc_comment(&named, source);
    let signature = match kind {
        StructureKind::Function | StructureKind::Macro => clojure_signature(&named, source),
        _ => None,
    };
    Some(StructureItem {
        kind,
        name,
        visibility,
        span: span_from_node(node),
        children: Vec::new(),
        decorators: Vec::new(),
        doc_comment,
        signature,
        body_span: None,
    })
}
