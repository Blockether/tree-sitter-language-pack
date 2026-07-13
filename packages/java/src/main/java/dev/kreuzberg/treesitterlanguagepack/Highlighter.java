package dev.kreuzberg.treesitterlanguagepack;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.regex.Pattern;
import org.jspecify.annotations.Nullable;

/**
 * Tree-sitter syntax highlighting, for every language the pack understands.
 *
 * <p>
 * Source is parsed with the bundled grammar and each byte is labeled by the grammar's OWN {@code highlights.scm} capture scheme, then
 * rendered as ANSI SGR runs. Classification is grammar-accurate (a {@code ;} inside a string is not a comment, a number inside a symbol is
 * not a number) and general across languages — the same parse tree the structural editors ({@link StructuralApi}) walk drives coloring too.
 *
 * <p>
 * Because the pack ships the {@code highlights.scm} TEXT ({@link TreeSitterLanguagePack#getHighlightsQuery(String)}) but no query
 * <em>executor</em>, this class contains a small, self-contained tree-sitter query interpreter: it compiles each language's query into
 * pattern matchers (named nodes, anonymous tokens, alternations {@code [ … ]}, field constraints {@code field:}, nested patterns, and
 * {@code #match?/#eq?/#any-of?} predicates) and runs them over the parse tree. This resolves the field-scoped captures a naive kind→capture
 * map misses — function names, calls, methods, types, properties — so coloring is rich, not just literals and keywords. Patterns are
 * applied in query order (later, more specific patterns override earlier broad ones, matching tree-sitter highlight semantics). Predicates
 * that need a locals/scope engine ({@code #is? #is-not?}) are treated as satisfied.
 *
 * <p>
 * Coloring is driven by a caller-supplied {@code capture → SGR} theme so the exact palette stays a rendering decision (a sensible
 * {@link #DEFAULT_THEME} is provided). A capture like {@code a.b.c} falls back to its {@code a} category. Emitted escapes are zero-width
 * and never cross a newline, so column-aligned / line-oriented painters are unaffected.
 */
public final class Highlighter {

    private Highlighter() {
    }

    // ---------------------------------------------------------------------------
    // Public API
    // ---------------------------------------------------------------------------

    /**
     * A reasonable default {@code capture → ANSI SGR foreground} palette (standard 8/16-color codes). Callers that theme their own terminal
     * should pass a map to {@link #highlightAnsi(String, String, Map)} instead.
     */
    public static final Map<String, String> DEFAULT_THEME = Map.ofEntries(Map.entry("keyword", "35"), Map.entry("string", "32"),
            Map.entry("escape", "35"), Map.entry("number", "33"), Map.entry("constant", "36"), Map.entry("constant.builtin", "36"),
            Map.entry("comment", "90"), Map.entry("function", "34"), Map.entry("type", "36"), Map.entry("constructor", "36"),
            Map.entry("property", "36"), Map.entry("operator", "37"));

    /**
     * ANSI-colorize {@code source} as tree-sitter {@code language} with the {@link #DEFAULT_THEME} palette.
     *
     * @see #highlightAnsi(String, String, Map)
     */
    public static @Nullable String highlightAnsi(final String source, final String language) throws TreeSitterLanguagePackRsException {
        return highlightAnsi(source, language, DEFAULT_THEME);
    }

    /**
     * ANSI-colorize {@code source} as tree-sitter {@code language}, mapping each grammar capture to a foreground SGR code via
     * {@code captureToSgr}. The result has the same newline structure as the input (ready to split into lines); escapes are zero-width and
     * never cross a line break.
     *
     * @param source
     *            code to highlight
     * @param language
     *            tree-sitter language name (e.g. {@code "clojure"})
     * @param captureToSgr
     *            {@code capture → SGR-foreground-code} theme (capture names WITHOUT a leading {@code @}; a {@code "a.b"} capture falls back
     *            to its {@code "a"} category)
     * @return the colored string, or {@code null} when the language ships no highlights query / the source is empty / parsing fails
     * @throws TreeSitterLanguagePackRsException
     *             on a native processing error
     */
    public static @Nullable String highlightAnsi(final String source, final String language, final Map<String, String> captureToSgr)
            throws TreeSitterLanguagePackRsException {
        if (source == null || source.isEmpty() || captureToSgr == null) {
            return null;
        }
        final boolean clj = "clojure".equals(language);
        final List<Pat> patterns = compile(language);
        if (patterns.isEmpty() && !clj) {
            return null;
        }
        final byte[] bs = source.getBytes(StandardCharsets.UTF_8);
        final int n = bs.length;
        final String[] caps = new String[n];
        try (Parser parser = TreeSitterLanguagePack.getParser(language)) {
            final Optional<Tree> parsed = parser.parse(source);
            if (parsed.isEmpty()) {
                return null;
            }
            try (Tree tree = parsed.get(); Node root = tree.rootNode()) {
                final TNode tRoot = materialize(root);
                final List<TNode> all = new ArrayList<>();
                flatten(tRoot, all);
                // Query fills, applied in query order so later (more specific) patterns
                // override earlier broad ones.
                for (final Pat pat : patterns) {
                    for (final TNode node : all) {
                        final List<Cap> matched = matchNode(pat.root, node);
                        if (matched == null || matched.isEmpty()) {
                            continue;
                        }
                        if (!predsPass(pat, matched, bs)) {
                            continue;
                        }
                        for (final Cap c : matched) {
                            fill(caps, c.node.start, c.node.end, c.cap);
                        }
                    }
                }
                // Clojure: the grammar's highlights.scm classifies literals but NOT the
                // head symbol of a list — light known special forms in the keyword slot.
                if (clj) {
                    for (final TNode node : all) {
                        if (!"list_lit".equals(node.kind)) {
                            continue;
                        }
                        for (final TNode ch : node.children) {
                            if (ch.named) {
                                if ("sym_lit".equals(ch.kind) && CLOJURE_SPECIAL.contains(text(ch, bs))) {
                                    fill(caps, ch.start, ch.end, "keyword");
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
        return renderAnsi(bs, caps, n, captureToSgr);
    }

    // ---------------------------------------------------------------------------
    // Clojure special forms (head symbols are unclassified by the grammar)
    // ---------------------------------------------------------------------------

    private static final Set<String> CLOJURE_SPECIAL = Set.of("def", "defn", "defn-", "defmacro", "definline", "defmulti", "defmethod",
            "defprotocol", "defrecord", "deftype", "definterface", "let", "let*", "letfn", "fn", "fn*", "if", "if-not", "if-let", "if-some",
            "when", "when-not", "when-let", "when-some", "when-first", "do", "cond", "condp", "case", "for", "doseq", "dotimes", "while",
            "loop", "recur", "ns", "require", "import", "use", "refer", "try", "catch", "finally", "throw", "quote", "var", "new", "set!",
            "locking", "and", "or", "->", "->>", "as->", "some->", "some->>", "cond->", "cond->>", "binding", "with-open",
            "with-local-vars", "with-redefs", "declare", "reify", "proxy", "extend-type", "extend-protocol", "future", "delay", "lazy-seq",
            "comment", "assert", "doto");

    // ---------------------------------------------------------------------------
    // Compiled query cache
    // ---------------------------------------------------------------------------

    private static final Map<String, List<Pat>> QUERY_CACHE = new ConcurrentHashMap<>();

    /** Compile a language's {@code highlights.scm} into ordered pattern matchers (cached). */
    private static List<Pat> compile(final String language) throws TreeSitterLanguagePackRsException {
        final List<Pat> cached = QUERY_CACHE.get(language);
        if (cached != null) {
            return cached;
        }
        final String scm = TreeSitterLanguagePack.getHighlightsQuery(language);
        final List<Pat> pats = scm == null ? List.of() : new QParser(tokenize(scm)).parseAll();
        QUERY_CACHE.put(language, pats);
        return pats;
    }

    // ---------------------------------------------------------------------------
    // Lightweight materialized parse tree (one FFI pass, then pure-JVM matching)
    // ---------------------------------------------------------------------------

    private static final class TNode {
        final String kind;
        final boolean named;
        final int start;
        final int end;
        final List<TNode> children;
        @Nullable
        String field;

        TNode(final String kind, final boolean named, final int start, final int end, final List<TNode> children) {
            this.kind = kind;
            this.named = named;
            this.start = start;
            this.end = end;
            this.children = children;
        }
    }

    private static TNode materialize(final Node nd) throws TreeSitterLanguagePackRsException {
        final String kind = nd.kind();
        final boolean named = nd.isNamed();
        final int sb = (int) nd.startByte();
        final int eb = (int) nd.endByte();
        final List<TNode> kids = new ArrayList<>();
        try (TreeCursor cur = nd.walk()) {
            if (cur.gotoFirstChild()) {
                do {
                    final String field = cur.fieldName().orElse(null);
                    try (Node child = cur.node()) {
                        final TNode t = materialize(child);
                        t.field = field;
                        kids.add(t);
                    }
                } while (cur.gotoNextSibling());
            }
        }
        return new TNode(kind, named, sb, eb, kids);
    }

    private static void flatten(final TNode n, final List<TNode> out) {
        out.add(n);
        for (final TNode c : n.children) {
            flatten(c, out);
        }
    }

    private static String text(final TNode t, final byte[] bs) {
        final int s = Math.max(0, t.start);
        final int e = Math.min(t.end, bs.length);
        if (e <= s) {
            return "";
        }
        return new String(bs, s, e - s, StandardCharsets.UTF_8);
    }

    // ---------------------------------------------------------------------------
    // Query pattern model
    // ---------------------------------------------------------------------------

    private static final int NAMED = 0;
    private static final int ANON = 1;
    private static final int ALT = 2;

    private static final class PatNode {
        int type;
        @Nullable
        String kind; // NAMED: node kind, null => `_` wildcard
        @Nullable
        String literal; // ANON: token text
        final List<PatNode> alts = new ArrayList<>(); // ALT
        final List<String> caps = new ArrayList<>(); // captures on this node
        final List<Child> children = new ArrayList<>(); // NAMED child constraints
        final List<Pred> preds = new ArrayList<>(); // predicates in-lined here
    }

    private static final class Child {
        @Nullable
        String field;
        boolean neg;
        @Nullable
        PatNode node;
    }

    private static final class Pred {
        @Nullable
        String op;
        @Nullable
        String cap;
        final List<String> args = new ArrayList<>();
        @Nullable
        Pattern regex;
    }

    private static final class Pat {
        final PatNode root;
        final List<Pred> preds;

        Pat(final PatNode root, final List<Pred> preds) {
            this.root = root;
            this.preds = preds;
        }
    }

    private static final class Cap {
        final String cap;
        final TNode node;

        Cap(final String cap, final TNode node) {
            this.cap = cap;
            this.node = node;
        }
    }

    // ---------------------------------------------------------------------------
    // Query tokenizer
    // ---------------------------------------------------------------------------

    private static final int T_LP = 0;
    private static final int T_RP = 1;
    private static final int T_LB = 2;
    private static final int T_RB = 3;
    private static final int T_CAP = 4;
    private static final int T_PRED = 5;
    private static final int T_STR = 6;
    private static final int T_IDENT = 7;
    private static final int T_FIELD = 8;
    private static final int T_NEG = 9;

    private static final class Tok {
        final int t;
        final String s;

        Tok(final int t, final String s) {
            this.t = t;
            this.s = s;
        }
    }

    private static boolean identChar(final char c) {
        return Character.isLetterOrDigit(c) || c == '_';
    }

    private static List<Tok> tokenize(final String q) {
        final List<Tok> out = new ArrayList<>();
        final int n = q.length();
        int i = 0;
        while (i < n) {
            final char ch = q.charAt(i);
            if (Character.isWhitespace(ch)) {
                i++;
                continue;
            }
            if (ch == ';') {
                while (i < n && q.charAt(i) != '\n') {
                    i++;
                }
                continue;
            }
            switch (ch) {
                case '(' :
                    out.add(new Tok(T_LP, "("));
                    i++;
                    continue;
                case ')' :
                    out.add(new Tok(T_RP, ")"));
                    i++;
                    continue;
                case '[' :
                    out.add(new Tok(T_LB, "["));
                    i++;
                    continue;
                case ']' :
                    out.add(new Tok(T_RB, "]"));
                    i++;
                    continue;
                case '.' :
                case '*' :
                case '+' :
                case '?' :
                    i++; // anchors / quantifiers — ignored
                    continue;
                default :
                    break;
            }
            if (ch == '@') {
                int j = i + 1;
                while (j < n && (identChar(q.charAt(j)) || q.charAt(j) == '.' || q.charAt(j) == '-')) {
                    j++;
                }
                out.add(new Tok(T_CAP, q.substring(i + 1, j)));
                i = j;
                continue;
            }
            if (ch == '#') {
                int j = i + 1;
                while (j < n) {
                    final char c = q.charAt(j);
                    if (identChar(c) || c == '?' || c == '!' || c == '.' || c == '-') {
                        j++;
                    } else {
                        break;
                    }
                }
                out.add(new Tok(T_PRED, q.substring(i + 1, j)));
                i = j;
                continue;
            }
            if (ch == '"') {
                final StringBuilder sb = new StringBuilder();
                int j = i + 1;
                while (j < n) {
                    final char c = q.charAt(j);
                    if (c == '\\' && j + 1 < n) {
                        final char e = q.charAt(j + 1);
                        switch (e) {
                            case 'n' :
                                sb.append('\n');
                                break;
                            case 'r' :
                                sb.append('\r');
                                break;
                            case 't' :
                                sb.append('\t');
                                break;
                            case '0' :
                                sb.append('\0');
                                break;
                            case '\\' :
                                sb.append('\\');
                                break;
                            case '"' :
                                sb.append('"');
                                break;
                            default :
                                sb.append('\\').append(e);
                                break;
                        }
                        j += 2;
                        continue;
                    }
                    if (c == '"') {
                        j++;
                        break;
                    }
                    sb.append(c);
                    j++;
                }
                out.add(new Tok(T_STR, sb.toString()));
                i = j;
                continue;
            }
            if (ch == '!') {
                int j = i + 1;
                while (j < n && (identChar(q.charAt(j)) || q.charAt(j) == '-' || q.charAt(j) == '.')) {
                    j++;
                }
                out.add(new Tok(T_NEG, q.substring(i + 1, j)));
                i = j;
                continue;
            }
            if (identChar(ch)) {
                int j = i;
                while (j < n && identChar(q.charAt(j))) {
                    j++;
                }
                final String id = q.substring(i, j);
                i = j;
                if (i < n && q.charAt(i) == ':') {
                    i++;
                    out.add(new Tok(T_FIELD, id));
                } else {
                    out.add(new Tok(T_IDENT, id));
                }
                continue;
            }
            i++; // unknown character — skip
        }
        return out;
    }

    // ---------------------------------------------------------------------------
    // Query parser
    // ---------------------------------------------------------------------------

    private static final class QParser {
        private final List<Tok> ts;
        private int i;

        QParser(final List<Tok> ts) {
            this.ts = ts;
        }

        private @Nullable Tok peek() {
            return i < ts.size() ? ts.get(i) : null;
        }

        private @Nullable Tok peek2() {
            return i + 1 < ts.size() ? ts.get(i + 1) : null;
        }

        private Tok next() {
            return ts.get(i++);
        }

        private boolean is(final int type) {
            final Tok t = peek();
            return t != null && t.t == type;
        }

        List<Pat> parseAll() {
            final List<Pat> out = new ArrayList<>();
            while (peek() != null) {
                final int before = i;
                final Pat p = parsePattern();
                if (p != null) {
                    out.add(p);
                }
                if (i == before) {
                    i++; // guard against no progress
                }
            }
            return out;
        }

        private @Nullable Pat parsePattern() {
            final Tok t = peek();
            if (t == null) {
                return null;
            }
            if (t.t == T_LP) {
                final Tok p2 = peek2();
                if (p2 != null && p2.t == T_PRED) {
                    skipGroup();
                    return null;
                }
                if (p2 != null && (p2.t == T_LP || p2.t == T_LB)) {
                    // wrapping group: ( <node> <pred>* )
                    next(); // (
                    final PatNode node = parseNode();
                    final List<Pred> preds = new ArrayList<>();
                    while (peek() != null && peek().t != T_RP) {
                        if (is(T_LP) && peek2() != null && peek2().t == T_PRED) {
                            parsePred(preds);
                        } else {
                            parseNode();
                        }
                    }
                    if (is(T_RP)) {
                        next();
                    }
                    if (node == null) {
                        return null;
                    }
                    preds.addAll(node.preds);
                    return new Pat(node, preds);
                }
            }
            final PatNode node = parseNode();
            if (node == null) {
                return null;
            }
            return new Pat(node, node.preds);
        }

        private @Nullable PatNode parseNode() {
            final Tok t = peek();
            if (t == null) {
                return null;
            }
            if (t.t == T_LP) {
                next(); // (
                final PatNode node = new PatNode();
                node.type = NAMED;
                final Tok h = peek();
                if (h != null && h.t == T_IDENT) {
                    node.kind = "_".equals(h.s) ? null : h.s;
                    next();
                } else {
                    node.kind = null; // wildcard / container
                }
                while (peek() != null && peek().t != T_RP) {
                    parseInner(node);
                }
                if (is(T_RP)) {
                    next();
                }
                while (is(T_CAP)) {
                    node.caps.add(next().s);
                }
                return node;
            }
            if (t.t == T_LB) {
                next(); // [
                final PatNode alt = new PatNode();
                alt.type = ALT;
                while (peek() != null && peek().t != T_RB) {
                    final PatNode m = parseNode();
                    if (m == null) {
                        break;
                    }
                    alt.alts.add(m);
                }
                if (is(T_RB)) {
                    next();
                }
                while (is(T_CAP)) {
                    alt.caps.add(next().s);
                }
                return alt;
            }
            if (t.t == T_STR) {
                next();
                final PatNode a = new PatNode();
                a.type = ANON;
                a.literal = t.s;
                while (is(T_CAP)) {
                    a.caps.add(next().s);
                }
                return a;
            }
            if (t.t == T_IDENT) {
                next();
                final PatNode node = new PatNode();
                node.type = NAMED;
                node.kind = "_".equals(t.s) ? null : t.s;
                while (is(T_CAP)) {
                    node.caps.add(next().s);
                }
                return node;
            }
            next(); // stray CAP / PRED / bracket
            return null;
        }

        private void parseInner(final PatNode node) {
            final Tok t = peek();
            if (t == null) {
                return;
            }
            if (t.t == T_CAP) {
                node.caps.add(next().s);
                return;
            }
            if (t.t == T_LP && peek2() != null && peek2().t == T_PRED) {
                parsePred(node.preds);
                return;
            }
            if (t.t == T_FIELD) {
                final String f = next().s;
                final PatNode child = parseNode();
                if (child != null) {
                    final Child c = new Child();
                    c.field = f;
                    c.node = child;
                    node.children.add(c);
                }
                return;
            }
            if (t.t == T_NEG) {
                final String f = next().s;
                final Child c = new Child();
                c.field = f;
                c.neg = true;
                node.children.add(c);
                return;
            }
            if (t.t == T_LP || t.t == T_LB || t.t == T_STR || t.t == T_IDENT) {
                final PatNode child = parseNode();
                if (child != null) {
                    final Child c = new Child();
                    c.node = child;
                    node.children.add(c);
                }
                return;
            }
            next(); // RB / PRED / stray
        }

        private void parsePred(final List<Pred> out) {
            next(); // (
            final Pred pr = new Pred();
            final Tok p = next(); // PRED
            pr.op = p.s;
            while (peek() != null && peek().t != T_RP) {
                final Tok a = peek();
                if (a.t == T_CAP) {
                    final String c = next().s;
                    if (pr.cap == null) {
                        pr.cap = c;
                    } else {
                        pr.args.add("@" + c);
                    }
                } else if (a.t == T_STR || a.t == T_IDENT) {
                    pr.args.add(next().s);
                } else if (a.t == T_LP) {
                    skipGroup();
                } else {
                    next();
                }
            }
            if (is(T_RP)) {
                next();
            }
            if (("match?".equals(pr.op) || "not-match?".equals(pr.op)) && !pr.args.isEmpty()) {
                try {
                    pr.regex = Pattern.compile(pr.args.get(0));
                } catch (final RuntimeException e) {
                    pr.regex = null;
                }
            }
            out.add(pr);
        }

        private void skipGroup() {
            int depth = 0;
            while (peek() != null) {
                final Tok t = next();
                if (t.t == T_LP) {
                    depth++;
                } else if (t.t == T_RP) {
                    depth--;
                    if (depth == 0) {
                        break;
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------------------
    // Matching
    // ---------------------------------------------------------------------------

    private static @Nullable List<Cap> matchNode(final PatNode p, final TNode n) {
        switch (p.type) {
            case NAMED : {
                if (p.kind != null && !p.kind.equals(n.kind)) {
                    return null;
                }
                final List<Cap> acc = new ArrayList<>();
                for (final Child ch : p.children) {
                    if (ch.neg) {
                        if (hasField(n, ch.field)) {
                            return null;
                        }
                        if (ch.node == null) {
                            continue;
                        }
                    }
                    final List<Cap> sub = matchChild(ch, n);
                    if (sub == null) {
                        return null;
                    }
                    acc.addAll(sub);
                }
                for (final String cap : p.caps) {
                    acc.add(new Cap(cap, n));
                }
                return acc;
            }
            case ANON : {
                if (n.named || p.literal == null || !p.literal.equals(n.kind)) {
                    return null;
                }
                final List<Cap> acc = new ArrayList<>();
                for (final String cap : p.caps) {
                    acc.add(new Cap(cap, n));
                }
                return acc;
            }
            case ALT : {
                for (final PatNode alt : p.alts) {
                    final List<Cap> sub = matchNode(alt, n);
                    if (sub != null) {
                        final List<Cap> acc = new ArrayList<>(sub);
                        for (final String cap : p.caps) {
                            acc.add(new Cap(cap, n));
                        }
                        return acc;
                    }
                }
                return null;
            }
            default :
                return null;
        }
    }

    private static @Nullable List<Cap> matchChild(final Child ch, final TNode parent) {
        if (ch.node == null) {
            return null;
        }
        for (final TNode c : parent.children) {
            if (ch.field != null && !ch.field.equals(c.field)) {
                continue;
            }
            final List<Cap> sub = matchNode(ch.node, c);
            if (sub != null) {
                return sub;
            }
        }
        return null;
    }

    private static boolean hasField(final TNode n, final @Nullable String field) {
        if (field == null) {
            return false;
        }
        for (final TNode c : n.children) {
            if (field.equals(c.field)) {
                return true;
            }
        }
        return false;
    }

    private static boolean predsPass(final Pat pat, final List<Cap> caps, final byte[] bs) {
        for (final Pred pr : pat.preds) {
            if (pr.op == null) {
                continue;
            }
            final List<TNode> targets = new ArrayList<>();
            for (final Cap c : caps) {
                if (c.cap.equals(pr.cap)) {
                    targets.add(c.node);
                }
            }
            if (targets.isEmpty()) {
                continue;
            }
            switch (pr.op) {
                case "match?" :
                    if (pr.regex == null) {
                        break;
                    }
                    for (final TNode t : targets) {
                        if (!pr.regex.matcher(text(t, bs)).find()) {
                            return false;
                        }
                    }
                    break;
                case "not-match?" :
                    if (pr.regex == null) {
                        break;
                    }
                    for (final TNode t : targets) {
                        if (pr.regex.matcher(text(t, bs)).find()) {
                            return false;
                        }
                    }
                    break;
                case "eq?" :
                    if (pr.args.isEmpty() || pr.args.get(0).startsWith("@")) {
                        break;
                    }
                    for (final TNode t : targets) {
                        if (!text(t, bs).equals(pr.args.get(0))) {
                            return false;
                        }
                    }
                    break;
                case "not-eq?" :
                    if (pr.args.isEmpty() || pr.args.get(0).startsWith("@")) {
                        break;
                    }
                    for (final TNode t : targets) {
                        if (text(t, bs).equals(pr.args.get(0))) {
                            return false;
                        }
                    }
                    break;
                case "any-of?" :
                    for (final TNode t : targets) {
                        if (!pr.args.contains(text(t, bs))) {
                            return false;
                        }
                    }
                    break;
                case "not-any-of?" :
                    for (final TNode t : targets) {
                        if (pr.args.contains(text(t, bs))) {
                            return false;
                        }
                    }
                    break;
                default :
                    break; // #is? #is-not? #set! … need a scope engine — treat as satisfied
            }
        }
        return true;
    }

    private static void fill(final String[] caps, final int start, final int end, final String cap) {
        final int e = Math.min(end, caps.length);
        for (int i = Math.max(0, start); i < e; i++) {
            caps[i] = cap;
        }
    }

    // ---------------------------------------------------------------------------
    // ANSI rendering
    // ---------------------------------------------------------------------------

    /** Resolve a capture to an SGR code: exact name first, then its top-level category. */
    private static @Nullable String sgrFor(final @Nullable String cap, final Map<String, String> theme) {
        if (cap == null) {
            return null;
        }
        final String direct = theme.get(cap);
        if (direct != null) {
            return direct;
        }
        final int dot = cap.indexOf('.');
        return dot > 0 ? theme.get(cap.substring(0, dot)) : null;
    }

    /**
     * Emit consecutive bytes of the same SGR as one {@code \u001b[<code>m…\u001b[0m} run; runs never cross a newline (each line stays
     * self-contained for a line-oriented painter), and byte slices decode as UTF-8 so multibyte glyphs survive.
     */
    private static String renderAnsi(final byte[] bs, final String[] caps, final int n, final Map<String, String> theme) {
        final StringBuilder sb = new StringBuilder(n + 16);
        int runStart = 0;
        String runSgr = n > 0 ? sgrFor(caps[0], theme) : null;
        int i = 0;
        while (i < n) {
            if (bs[i] == (byte) '\n') {
                flush(sb, bs, runStart, i, runSgr);
                sb.append('\n');
                i++;
                runStart = i;
                runSgr = i < n ? sgrFor(caps[i], theme) : null;
            } else {
                final String sgr = sgrFor(caps[i], theme);
                if (!Objects.equals(sgr, runSgr)) {
                    flush(sb, bs, runStart, i, runSgr);
                    runStart = i;
                    runSgr = sgr;
                }
                i++;
            }
        }
        flush(sb, bs, runStart, n, runSgr);
        return sb.toString();
    }

    private static void flush(final StringBuilder sb, final byte[] bs, final int start, final int end, final @Nullable String sgr) {
        if (end <= start) {
            return;
        }
        if (sgr != null) {
            sb.append("\u001b[").append(sgr).append('m');
        }
        sb.append(new String(bs, start, end - start, StandardCharsets.UTF_8));
        if (sgr != null) {
            sb.append("\u001b[0m");
        }
    }
}
