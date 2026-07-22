//! JavaScript / TypeScript / TSX intelligence.
//!
//! Imports are `import_statement`s and exports `export_statement`s. Structure,
//! though, needs a JS/TS-aware walk rather than the generic node-kind one.
//!
//! In every other curly-brace language a definition is a node that carries a
//! `name` field, so the generic walker just reads it. Modern JS/TS breaks that
//! assumption: the dominant pattern binds an **anonymous** function or class
//! *expression* to a name through a declaration —
//!
//! ```ignore
//! const Button = (props) => { … };          // React component
//! export const useThing = () => { … };       // hook
//! const Wrapped = memo(() => { … });          // HOC-wrapped component
//! class Panel { handleClick = () => { … }; }  // class-field arrow
//! ```
//!
//! Feeding those through the generic walker records the bare `arrow_function` /
//! `function_expression` nodes: every one is *nameless* (the name lives on the
//! enclosing `variable_declarator` / field, not the function), and it fires on
//! **every** inline callback too — `arr.map(x => …)`, `onPress={() => …}`,
//! `useEffect(() => …)` — burying the real components under a pile of anonymous
//! rows. This walker instead names a function/class after the identifier it is
//! bound to and skips anonymous inline functions entirely. It also records the
//! type-level definitions the generic walker would miss for TS/TSX — `interface`,
//! `type` alias, `enum`, `namespace`/ambient `module`, and abstract methods.

use super::LanguageIntel;
use crate::intel::intelligence::{node_text, span_from_node};
use crate::intel::types::{StructureItem, StructureKind};
use tree_sitter::Node;

pub(crate) struct JsTs;

impl LanguageIntel for JsTs {
    fn is_import(&self, node_kind: &str) -> bool {
        node_kind == "import_statement"
    }

    fn is_export(&self, node_kind: &str) -> bool {
        node_kind == "export_statement"
    }

    fn structure(&self, root: &Node, source: &str) -> Vec<StructureItem> {
        let mut items = Vec::with_capacity(32);
        walk(root, source, &mut items);
        items
    }
}

/// Recursive structure walk. At a *definition* node record it (naming it and,
/// where the language hides the name on a binding, recovering it) and descend
/// into its body for nested defs; otherwise recurse into all children so
/// definitions nested inside containers (blocks, exports, expressions) surface.
fn walk(node: &Node, source: &str, out: &mut Vec<StructureItem>) {
    match node.kind() {
        "function_declaration" | "generator_function_declaration" | "function_signature" => {
            push(
                node,
                StructureKind::Function,
                name_field(node, source),
                body_of(node),
                signature_of(node, source),
                source,
                out,
            );
        }
        "class_declaration" | "abstract_class_declaration" => {
            push(
                node,
                StructureKind::Class,
                name_field(node, source),
                body_of(node),
                None,
                source,
                out,
            );
        }
        "method_definition" => {
            push(
                node,
                StructureKind::Method,
                name_field(node, source),
                body_of(node),
                signature_of(node, source),
                source,
                out,
            );
        }
        // Class-field arrow methods (`handleClick = () => { … }`). A plain data
        // field is not structural — descend past it without recording.
        "public_field_definition" | "field_definition" => {
            match node.child_by_field_name("value").and_then(|v| callable_value(&v)) {
                Some((_, body, fn_node)) => push(
                    node,
                    StructureKind::Method,
                    name_field(node, source),
                    body,
                    signature_of(&fn_node, source),
                    source,
                    out,
                ),
                None => descend(node, source, out),
            }
        }
        // Named type-level definitions (`interface Props { … }`, `type T = …`,
        // `enum E { … }`). Recorded as leaves: their members — property/method
        // signatures, enum variants — are type structure, not nested defs.
        "interface_declaration" => {
            push(
                node,
                StructureKind::Interface,
                name_field(node, source),
                None,
                None,
                source,
                out,
            );
        }
        "type_alias_declaration" => {
            push(
                node,
                StructureKind::Type,
                name_field(node, source),
                None,
                None,
                source,
                out,
            );
        }
        "enum_declaration" => {
            push(
                node,
                StructureKind::Enum,
                name_field(node, source),
                None,
                None,
                source,
                out,
            );
        }
        // `namespace X { … }` / `module X.Y { … }` (both parse as internal_module).
        // Descend into the body so nested functions/classes/interfaces surface as
        // children rather than leaking to the top level.
        "internal_module" => {
            push(
                node,
                StructureKind::Namespace,
                name_field(node, source),
                body_of(node),
                None,
                source,
                out,
            );
        }
        // Ambient `declare module "pkg" { … }` — the name is a string specifier.
        "module" => {
            push(
                node,
                StructureKind::Module,
                module_name(node, source),
                body_of(node),
                None,
                source,
                out,
            );
        }
        // Abstract method signatures inside an abstract class body (`abstract f(): T;`).
        "abstract_method_signature" => {
            push(
                node,
                StructureKind::Method,
                name_field(node, source),
                None,
                signature_of(node, source),
                source,
                out,
            );
        }
        "lexical_declaration" | "variable_declaration" => walk_declaration(node, source, out),
        _ => descend(node, source, out),
    }
}

/// `const`/`let`/`var` declarations. A declarator whose value is a function or
/// class expression (directly, or wrapped in a known HOC) is a named definition;
/// the identifier being bound supplies the name and the callback body its arity.
/// When the whole statement binds exactly one such value, record the whole
/// statement (so the span covers `const … = …;`); with several declarators,
/// record each on its own declarator span.
///
/// A declarator whose value is NOT callable is a plain value binding. At module
/// scope (`const TERMINAL_EVENTS = new Set([…])`, a top-level or namespace-level
/// `const`/`let`/`var`) it is a targetable constant/variable definition and is
/// recorded. Inside a function body the same shape is value noise (`useRef(null)`,
/// `useState(0)`, `StyleSheet.create(…)`) and is intentionally skipped. Only plain
/// identifier bindings are recorded — destructuring patterns are not.
fn walk_declaration(node: &Node, source: &str, out: &mut Vec<StructureItem>) {
    let mut cursor = node.walk();
    let declarators: Vec<Node> = node
        .named_children(&mut cursor)
        .filter(|n| n.kind() == "variable_declarator")
        .collect();

    let single = declarators.len() == 1;
    let module_scope = is_module_scope(node);
    let value_kind = declaration_kind(node, source);
    let mut recorded = false;

    for d in &declarators {
        // Span the whole `const NAME = … ;` statement when it is the only binding;
        // otherwise each declarator carries its own span.
        let span_node = if single { node } else { d };
        match declarator_value(d).and_then(|v| callable_value(&v)) {
            Some((kind, body, fn_node)) => {
                push(
                    span_node,
                    kind,
                    name_field(d, source),
                    body,
                    signature_of(&fn_node, source),
                    source,
                    out,
                );
                recorded = true;
            }
            None if module_scope && declarator_name_is_identifier(d) => {
                push(
                    span_node,
                    value_kind.clone(),
                    name_field(d, source),
                    None,
                    None,
                    source,
                    out,
                );
                recorded = true;
            }
            None => {}
        }
    }

    // Nothing recorded (no callable value, and not a module-scope binding): descend
    // in case a value literal nests a named def.
    if !recorded {
        descend(node, source, out);
    }
}

/// Classify a declarator/field *value*: a function/class expression, or a call
/// carrying a callback argument (a hook/HOC like `useCallback`/`memo`). Returns
/// the [`StructureKind`], the function/class body to descend for nested defs, and
/// the function-ish node whose parameter list supplies the signature/arity.
fn callable_value<'t>(value: &Node<'t>) -> Option<(StructureKind, Option<Node<'t>>, Node<'t>)> {
    let kind = value.kind();
    if is_function_expr(kind) {
        return Some((StructureKind::Function, value.child_by_field_name("body"), *value));
    }
    if kind == "class" {
        return Some((StructureKind::Class, value.child_by_field_name("body"), *value));
    }
    // A call whose argument list carries a function expression — the callback in
    // a hook or HOC (`useCallback(() => …)`, `useMemo(() => …)`, `memo(…)`,
    // `forwardRef((p, r) => …)`, a custom `useThing(() => …)`). The binding name
    // is the real definition; the callback supplies the body and the signature.
    if kind == "call_expression"
        && let Some(cb) = call_fn_arg(value)
    {
        return Some((StructureKind::Function, cb.child_by_field_name("body"), cb));
    }
    None
}

/// A node kind that is an anonymous function *expression* — the right-hand side
/// of a binding (never a `*_declaration`, which the walker handles by name).
fn is_function_expr(kind: &str) -> bool {
    matches!(
        kind,
        "arrow_function" | "function_expression" | "function" | "generator_function"
    )
}

/// The first argument of `call` that is a function expression — the callback in
/// a hook or HOC call (`useCallback(() => …)`, `memo(() => …)`). Its presence is
/// what marks the enclosing `const NAME = call(…)` binding as a named definition;
/// the callback body carries any nested defs. A call with no function argument
/// (`useRef(null)`, `StyleSheet.create({…})`) binds a value, not a def, and is
/// intentionally not matched.
fn call_fn_arg<'t>(call: &Node<'t>) -> Option<Node<'t>> {
    let args = call.child_by_field_name("arguments")?;
    let mut cursor = args.walk();
    args.named_children(&mut cursor).find(|a| is_function_expr(a.kind()))
}

fn declarator_value<'t>(declarator: &Node<'t>) -> Option<Node<'t>> {
    declarator.child_by_field_name("value")
}

fn body_of<'t>(node: &Node<'t>) -> Option<Node<'t>> {
    node.child_by_field_name("body")
}

fn name_field(node: &Node, source: &str) -> Option<String> {
    node.child_by_field_name("name")
        .map(|n| node_text(&n, source).to_string())
        .filter(|s| !s.is_empty())
}

/// Name for an ambient `module "pkg" { … }` — the specifier is a string node, so
/// return its inner text without the surrounding quotes. Falls back to the raw
/// name text for any non-string form.
fn module_name(node: &Node, source: &str) -> Option<String> {
    let name = node.child_by_field_name("name")?;
    if name.kind() == "string" {
        let mut cursor = name.walk();
        if let Some(frag) = name.named_children(&mut cursor).find(|c| c.kind() == "string_fragment") {
            return Some(node_text(&frag, source).to_string());
        }
    }
    let t = node_text(&name, source);
    (!t.is_empty()).then(|| t.to_string())
}

/// The `//` / `/* … */` / JSDoc comment block written directly above a
/// definition — its intent gist for the outline. An `export`-wrapped def keeps
/// its comment above the `export` keyword, so climb through export wrappers
/// first, then gather the run of `comment` siblings immediately preceding the
/// def (a blank line between the comment and the def detaches it). The comment
/// delimiters are stripped so the first line reads as prose. `None` when there
/// is no attached comment.
fn leading_doc_comment(node: &Node, source: &str) -> Option<String> {
    let mut anchor = *node;
    while let Some(parent) = anchor.parent() {
        if parent.kind() == "export_statement" {
            anchor = parent;
        } else {
            break;
        }
    }

    let mut comments: Vec<Node> = Vec::new();
    let mut below = anchor.start_position().row;
    let mut sib = anchor.prev_sibling();
    while let Some(comment) = sib {
        if comment.kind() != "comment" {
            break;
        }
        // A blank line between the comment and what follows detaches it.
        if comment.end_position().row + 1 < below {
            break;
        }
        below = comment.start_position().row;
        comments.push(comment);
        sib = comment.prev_sibling();
    }
    if comments.is_empty() {
        return None;
    }
    comments.reverse();
    let text = comments
        .iter()
        .map(|c| clean_comment(node_text(c, source)))
        .collect::<Vec<_>>()
        .join("\n");
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// Strip comment delimiters (`//`, `///`, `/* … */`, per-line `*`) from a raw
/// comment node, leaving one cleaned line of human text per source line.
fn clean_comment(raw: &str) -> String {
    let raw = raw.trim();
    let block = raw
        .strip_prefix("/**")
        .or_else(|| raw.strip_prefix("/*"))
        .map(|s| s.strip_suffix("*/").unwrap_or(s));
    if let Some(block) = block {
        block
            .lines()
            .map(|l| l.trim().trim_start_matches('*').trim())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        raw.trim_start_matches('/').trim().to_string()
    }
}

/// Record one definition, descending into `body` for its nested definitions.
fn push(
    span_node: &Node,
    kind: StructureKind,
    name: Option<String>,
    body: Option<Node>,
    signature: Option<String>,
    source: &str,
    out: &mut Vec<StructureItem>,
) {
    let mut children = Vec::new();
    if let Some(b) = &body {
        descend(b, source, &mut children);
    }
    out.push(StructureItem {
        kind,
        name,
        span: span_from_node(span_node),
        children,
        signature,
        body_span: body.map(|b| span_from_node(&b)),
        doc_comment: leading_doc_comment(span_node, source),
        ..Default::default()
    });
}

/// Trim and collapse internal whitespace runs to a single space.
fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The parameter-list signature (arity) of a function/method/arrow node: its
/// `formal_parameters` text, or a bare single arrow parameter (`x => …`) wrapped
/// in parens, with any TypeScript return-type annotation appended
/// (`(a: number, b: number): number`). `None` when the node carries no parameter
/// list (e.g. a class expression).
fn signature_of(node: &Node, source: &str) -> Option<String> {
    let params = node
        .child_by_field_name("parameters")
        .or_else(|| node.child_by_field_name("parameter"))?;
    let mut sig = collapse_ws(node_text(&params, source));
    if !sig.starts_with('(') {
        sig = format!("({sig})");
    }
    if let Some(rt) = node.child_by_field_name("return_type") {
        let rt = collapse_ws(node_text(&rt, source));
        if !rt.is_empty() {
            sig.push_str(&rt);
        }
    }
    Some(sig)
}

/// Whether `node` sits at module scope — no function/method/arrow body encloses
/// it. Distinguishes a top-level (or namespace-level) `const NAME = value`
/// definition from a value binding inside a function body.
fn is_module_scope(node: &Node) -> bool {
    let mut cur = node.parent();
    while let Some(p) = cur {
        if matches!(
            p.kind(),
            "function_declaration"
                | "generator_function_declaration"
                | "function_expression"
                | "function"
                | "generator_function"
                | "arrow_function"
                | "method_definition"
        ) {
            return false;
        }
        cur = p.parent();
    }
    true
}

/// Classify a declaration statement by its keyword: `const` → [`Constant`], any
/// other (`let`/`var`) → [`Variable`].
fn declaration_kind(node: &Node, source: &str) -> StructureKind {
    let mut cursor = node.walk();
    let is_const = node.children(&mut cursor).any(|c| node_text(&c, source) == "const");
    if is_const {
        StructureKind::Constant
    } else {
        StructureKind::Variable
    }
}

/// Whether the declarator binds a plain identifier (not a destructuring
/// array/object pattern) — only those yield a clean, targetable name.
fn declarator_name_is_identifier(declarator: &Node) -> bool {
    declarator
        .child_by_field_name("name")
        .is_some_and(|n| n.kind() == "identifier")
}

fn descend(node: &Node, source: &str, out: &mut Vec<StructureItem>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(&child, source, out);
    }
}

#[cfg(test)]
mod tests {
    use crate::intel::intelligence::extract_intelligence;
    use crate::intel::types::*;

    fn parse_or_skip(source: &str, lang_name: &str) -> Option<tree_sitter::Tree> {
        let registry = crate::LanguageRegistry::new();
        let lang = registry.get_language(lang_name).ok()?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang).ok()?;
        parser.parse(source, None)
    }

    fn collect_names<'a>(items: &'a [StructureItem], out: &mut Vec<&'a str>) {
        for s in items {
            if let Some(n) = s.name.as_deref() {
                out.push(n);
            }
            collect_names(&s.children, out);
        }
    }

    fn count_anonymous(items: &[StructureItem]) -> usize {
        items
            .iter()
            .map(|s| {
                let here =
                    usize::from(s.name.is_none() && matches!(s.kind, StructureKind::Function | StructureKind::Method));
                here + count_anonymous(&s.children)
            })
            .sum()
    }

    const SAMPLE: &str = r#"
import React from "react";

export const Button = ({ label }: { label: string }) => {
  return <button>{label}</button>;
};

function App() {
  const items = [1, 2, 3];
  return <div>{items.map((n) => <span key={n}>{n}</span>)}</div>;
}

const useCounter = () => {
  const [n, setN] = React.useState(0);
  return n;
};

const Wrapped = memo(() => {
  return null;
});

class Panel extends React.Component {
  handleClick = () => {
    this.forceUpdate();
  };

  render() {
    return null;
  }
}
"#;

    #[test]
    fn tsx_names_function_and_class_bindings_and_skips_inline_callbacks() {
        let Some(tree) = parse_or_skip(SAMPLE, "tsx") else {
            return; // grammar not available in this build — CI covers it.
        };
        let intel = extract_intelligence(SAMPLE, "tsx", &tree);

        let mut names = Vec::new();
        collect_names(&intel.structure, &mut names);

        for expected in [
            "Button",      // arrow-function component (const binding)
            "App",         // function declaration
            "useCounter",  // arrow-function hook
            "Wrapped",     // memo()-wrapped component
            "Panel",       // class
            "handleClick", // class-field arrow method
            "render",      // class method
        ] {
            assert!(names.contains(&expected), "missing {expected}; got {names:?}");
        }

        // The inline `items.map((n) => …)` callback must NOT appear as a
        // nameless structure row — that anonymous noise is exactly the bug.
        assert_eq!(
            count_anonymous(&intel.structure),
            0,
            "no anonymous function/method rows expected; got {names:?}"
        );

        let button = intel
            .structure
            .iter()
            .find(|s| s.name.as_deref() == Some("Button"))
            .unwrap();
        assert_eq!(button.kind, StructureKind::Function);
        // Span covers the whole `const Button = … ;` statement, not just the arrow.
        assert_eq!(button.span.start_line, 3);

        let panel = intel
            .structure
            .iter()
            .find(|s| s.name.as_deref() == Some("Panel"))
            .unwrap();
        assert_eq!(panel.kind, StructureKind::Class);
        let method_names: Vec<&str> = panel.children.iter().filter_map(|s| s.name.as_deref()).collect();
        assert!(method_names.contains(&"handleClick"), "class methods: {method_names:?}");
        assert!(method_names.contains(&"render"), "class methods: {method_names:?}");
    }

    const TYPE_SAMPLE: &str = r#"
export interface ButtonProps {
  label: string;
  onClick(): void;
}

type Variant = "primary" | "secondary";
export type Handler = (e: Event) => void;

export enum Status {
  Active,
  Inactive,
}

export namespace Geometry {
  export function area(r: number): number {
    return Math.PI * r * r;
  }
  export interface Point {
    x: number;
    y: number;
  }
}

declare module "virtual:foo" {
  export const version: string;
}

abstract class Base {
  abstract render(): void;
  concrete(): void {}
}
"#;

    /// Find the first structure item with `name` anywhere in the tree.
    fn find_named<'a>(items: &'a [StructureItem], name: &str) -> Option<&'a StructureItem> {
        for s in items {
            if s.name.as_deref() == Some(name) {
                return Some(s);
            }
            if let Some(found) = find_named(&s.children, name) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn tsx_records_type_level_and_namespaced_definitions() {
        let Some(tree) = parse_or_skip(TYPE_SAMPLE, "tsx") else {
            return; // grammar not available in this build — CI covers it.
        };
        let intel = extract_intelligence(TYPE_SAMPLE, "tsx", &tree);
        let top = &intel.structure;

        // interface / type alias / enum — previously dropped entirely.
        assert_eq!(
            find_named(top, "ButtonProps").map(|s| &s.kind),
            Some(&StructureKind::Interface)
        );
        assert_eq!(find_named(top, "Variant").map(|s| &s.kind), Some(&StructureKind::Type));
        assert_eq!(find_named(top, "Handler").map(|s| &s.kind), Some(&StructureKind::Type));
        assert_eq!(find_named(top, "Status").map(|s| &s.kind), Some(&StructureKind::Enum));

        // `namespace Geometry { … }` is recorded AND its members nest as children
        // rather than leaking to the top level.
        let geometry = top
            .iter()
            .find(|s| s.name.as_deref() == Some("Geometry"))
            .expect("namespace Geometry recorded");
        assert_eq!(geometry.kind, StructureKind::Namespace);
        let nested: Vec<&str> = geometry.children.iter().filter_map(|s| s.name.as_deref()).collect();
        assert!(nested.contains(&"area"), "namespace members: {nested:?}");
        assert!(nested.contains(&"Point"), "namespace members: {nested:?}");
        // `area` must NOT also appear at the top level.
        assert!(
            !top.iter().any(|s| s.name.as_deref() == Some("area")),
            "namespace member leaked to top level"
        );

        // Ambient `declare module "virtual:foo"` — recorded with the quotes stripped.
        let module = find_named(top, "virtual:foo").expect("ambient module recorded");
        assert_eq!(module.kind, StructureKind::Module);

        // Abstract method signature inside an abstract class body.
        let base = top
            .iter()
            .find(|s| s.name.as_deref() == Some("Base"))
            .expect("abstract class Base recorded");
        let methods: Vec<&str> = base.children.iter().filter_map(|s| s.name.as_deref()).collect();
        assert!(methods.contains(&"render"), "abstract methods: {methods:?}");
        assert!(methods.contains(&"concrete"), "class methods: {methods:?}");
    }

    const HOOK_SAMPLE: &str = r#"
export function Screen() {
  const value = useMemo(() => compute(), [dep]);
  const onPress = useCallback(() => { doThing(); }, []);
  const listRef = useRef(null);
  const [count, setCount] = useState(0);
  const styles = StyleSheet.create({ a: {} });
  return null;
}
"#;

    #[test]
    fn tsx_names_hook_bound_callbacks_and_skips_value_hooks() {
        let Some(tree) = parse_or_skip(HOOK_SAMPLE, "tsx") else {
            return; // grammar not available in this build — CI covers it.
        };
        let intel = extract_intelligence(HOOK_SAMPLE, "tsx", &tree);
        let screen = intel
            .structure
            .iter()
            .find(|s| s.name.as_deref() == Some("Screen"))
            .expect("Screen component recorded");
        let kids: Vec<&str> = screen.children.iter().filter_map(|s| s.name.as_deref()).collect();

        // `useMemo` / `useCallback` bindings ARE named definitions (a call with a
        // function argument) — the component's handlers, previously dropped.
        assert!(kids.contains(&"value"), "useMemo binding missing: {kids:?}");
        assert!(kids.contains(&"onPress"), "useCallback binding missing: {kids:?}");

        // `useRef` / `useState` / `StyleSheet.create` carry NO function argument —
        // they bind values, not definitions, and must stay out of the outline.
        assert!(!kids.contains(&"listRef"), "useRef must not be a def: {kids:?}");
        assert!(
            !kids.contains(&"styles"),
            "StyleSheet.create must not be a def: {kids:?}"
        );
        assert!(!kids.contains(&"count"), "useState must not be a def: {kids:?}");
    }

    const COMMENT_SAMPLE: &str = r#"
// A greeting component.
export function Hello() {
  return null;
}

/**
 * Adds two numbers.
 * @param a first
 */
function add(a: number, b: number) {
  return a + b;
}

// A detached note.

function undocumented() {}

// Bound handler intent.
const handler = () => {};
"#;

    #[test]
    fn tsx_attaches_leading_comments_as_doc_gists() {
        let Some(tree) = parse_or_skip(COMMENT_SAMPLE, "tsx") else {
            return; // grammar not available in this build — CI covers it.
        };
        let intel = extract_intelligence(COMMENT_SAMPLE, "tsx", &tree);
        let doc = |name: &str| {
            intel
                .structure
                .iter()
                .find(|s| s.name.as_deref() == Some(name))
                .unwrap_or_else(|| panic!("{name} recorded"))
                .doc_comment
                .clone()
        };

        // A line comment above the def rides in as its gist — the exported
        // function keeps the comment written above `export`.
        assert_eq!(doc("Hello").as_deref(), Some("A greeting component."));

        // A JSDoc block is de-delimited; the first line is the gist.
        let add = doc("add").expect("add has a doc comment");
        assert!(add.starts_with("Adds two numbers."), "add doc: {add:?}");

        // A blank line between the comment and the def detaches it.
        assert_eq!(doc("undocumented"), None);

        // Comments attach to name-bound arrow definitions too.
        assert_eq!(doc("handler").as_deref(), Some("Bound handler intent."));
    }

    const CONST_SAMPLE: &str = r#"
const TERMINAL_EVENTS = new Set(["turn.completed", "turn.failed"]);
export const MAX = 42;
let mutableCount = 0;

export function reducer(state: State, event: Event): State {
  const local = new Map();
  const [a, b] = pair;
  return state;
}
"#;

    #[test]
    fn tsx_records_module_scope_value_bindings_and_function_signatures() {
        let Some(tree) = parse_or_skip(CONST_SAMPLE, "tsx") else {
            return; // grammar not available in this build — CI covers it.
        };
        let intel = extract_intelligence(CONST_SAMPLE, "tsx", &tree);
        let top = &intel.structure;

        // A top-level `const NAME = <non-callable>` is now a targetable constant
        // (previously dropped, so `struct_patch` could not find it).
        let ev = find_named(top, "TERMINAL_EVENTS").expect("TERMINAL_EVENTS recorded");
        assert_eq!(ev.kind, StructureKind::Constant);
        // Span covers the whole `const … = …;` statement.
        assert_eq!(ev.span.start_line, 1);
        // `export const` is a constant too.
        assert_eq!(find_named(top, "MAX").map(|s| &s.kind), Some(&StructureKind::Constant));
        // `let` is a mutable variable, not a constant.
        assert_eq!(
            find_named(top, "mutableCount").map(|s| &s.kind),
            Some(&StructureKind::Variable)
        );

        // Value bindings INSIDE a function body stay out of the outline…
        assert!(find_named(top, "local").is_none(), "in-body const must not be a def");
        // …and destructuring patterns are never recorded (no clean identifier name).
        assert!(find_named(top, "a").is_none(), "destructuring must not be a def");

        // The function reports its parameter-list signature (arity + types).
        let reducer = find_named(top, "reducer").expect("reducer recorded");
        assert_eq!(reducer.kind, StructureKind::Function);
        assert_eq!(
            reducer.signature.as_deref(),
            Some("(state: State, event: Event): State")
        );
    }

    const SIG_SAMPLE: &str = r#"
export const Button = (label: string, onClick: () => void) => {
  return null;
};

const single = x => x + 1;

function plain(a, b, c) {
  return a;
}

class C {
  method(p: number): void {}
}
"#;

    #[test]
    fn tsx_reports_signatures_for_arrow_function_and_method_arities() {
        let Some(tree) = parse_or_skip(SIG_SAMPLE, "tsx") else {
            return; // grammar not available in this build — CI covers it.
        };
        let intel = extract_intelligence(SIG_SAMPLE, "tsx", &tree);
        let top = &intel.structure;

        // Arrow-bound component: full parenthesised parameter list.
        assert_eq!(
            find_named(top, "Button").and_then(|s| s.signature.as_deref()),
            Some("(label: string, onClick: () => void)")
        );
        // A single unparenthesised arrow param is wrapped in parens.
        assert_eq!(
            find_named(top, "single").and_then(|s| s.signature.as_deref()),
            Some("(x)")
        );
        // A plain function declaration reports its arity.
        assert_eq!(
            find_named(top, "plain").and_then(|s| s.signature.as_deref()),
            Some("(a, b, c)")
        );
        // A class method reports params plus its return type.
        assert_eq!(
            find_named(top, "method").and_then(|s| s.signature.as_deref()),
            Some("(p: number): void")
        );
    }
}
