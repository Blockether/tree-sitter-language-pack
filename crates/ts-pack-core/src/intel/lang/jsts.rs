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
                source,
                out,
            );
        }
        // Class-field arrow methods (`handleClick = () => { … }`). A plain data
        // field is not structural — descend past it without recording.
        "public_field_definition" | "field_definition" => {
            match node
                .child_by_field_name("value")
                .and_then(|v| callable_value(&v))
            {
                Some((_, body)) => push(node, StructureKind::Method, name_field(node, source), body, source, out),
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
                source,
                out,
            );
        }
        "type_alias_declaration" => {
            push(node, StructureKind::Type, name_field(node, source), None, source, out);
        }
        "enum_declaration" => {
            push(node, StructureKind::Enum, name_field(node, source), None, source, out);
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
                source,
                out,
            );
        }
        // Abstract method signatures inside an abstract class body (`abstract f(): T;`).
        "abstract_method_signature" => {
            push(node, StructureKind::Method, name_field(node, source), None, source, out);
        }
        "lexical_declaration" | "variable_declaration" => walk_declaration(node, source, out),
        _ => descend(node, source, out),
    }
}

/// `const`/`let`/`var` declarations. A declarator whose value is a function or
/// class expression (directly, or wrapped in a known HOC) is a named definition;
/// the identifier being bound supplies the name. When the whole statement binds
/// exactly one such value, record the whole statement (so the span covers the
/// `const … = …;`); with several declarators, record each callable one on its
/// own declarator span. Declarations that bind no function value are not
/// structural, but we still descend in case a value literal nests a named def.
fn walk_declaration(node: &Node, source: &str, out: &mut Vec<StructureItem>) {
    let mut cursor = node.walk();
    let declarators: Vec<Node> = node
        .named_children(&mut cursor)
        .filter(|n| n.kind() == "variable_declarator")
        .collect();

    let callable: Vec<&Node> = declarators
        .iter()
        .filter(|d| declarator_value(d).and_then(|v| callable_value(&v)).is_some())
        .collect();

    if callable.is_empty() {
        descend(node, source, out);
        return;
    }

    if declarators.len() == 1 {
        // Span the whole `const NAME = … ;` statement.
        let d = &declarators[0];
        if let Some((kind, body)) = declarator_value(d).and_then(|v| callable_value(&v)) {
            push(node, kind, name_field(d, source), body, source, out);
        }
    } else {
        for d in callable {
            if let Some((kind, body)) = declarator_value(d).and_then(|v| callable_value(&v)) {
                push(d, kind, name_field(d, source), body, source, out);
            }
        }
    }
}

/// Classify a declarator/field *value*: a function/class expression, or a call
/// carrying a callback argument (a hook/HOC like `useCallback`/`memo`). Returns
/// the [`StructureKind`] plus the function/class body to descend for nested defs.
fn callable_value<'t>(value: &Node<'t>) -> Option<(StructureKind, Option<Node<'t>>)> {
    let kind = value.kind();
    if is_function_expr(kind) {
        return Some((StructureKind::Function, value.child_by_field_name("body")));
    }
    if kind == "class" {
        return Some((StructureKind::Class, value.child_by_field_name("body")));
    }
    // A call whose argument list carries a function expression — the callback in
    // a hook or HOC (`useCallback(() => …)`, `useMemo(() => …)`, `memo(…)`,
    // `forwardRef((p, r) => …)`, a custom `useThing(() => …)`). The binding name
    // is the real definition; descend the callback body for any nested defs.
    if kind == "call_expression"
        && let Some(cb) = call_fn_arg(value)
    {
        return Some((StructureKind::Function, cb.child_by_field_name("body")));
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

/// Record one definition, descending into `body` for its nested definitions.
fn push(
    span_node: &Node,
    kind: StructureKind,
    name: Option<String>,
    body: Option<Node>,
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
        body_span: body.map(|b| span_from_node(&b)),
        ..Default::default()
    });
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
        assert!(!kids.contains(&"styles"), "StyleSheet.create must not be a def: {kids:?}");
        assert!(!kids.contains(&"count"), "useState must not be a def: {kids:?}");
    }
}
