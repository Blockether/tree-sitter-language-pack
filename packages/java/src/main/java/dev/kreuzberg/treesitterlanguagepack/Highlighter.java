package dev.kreuzberg.treesitterlanguagepack;

import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.jspecify.annotations.Nullable;

/**
 * Tree-sitter syntax highlighting, for every language the pack understands.
 * Parses source with the bundled grammar and labels each byte by the grammar's
 * OWN {@code highlights.scm} capture scheme, then renders ANSI SGR runs — so the
 * classification is accurate (a {@code ;} inside a string is not a comment, a
 * number inside a symbol is not a number) and general across languages.
 *
 * <p>This is the JVM-native replacement for a per-consumer regex lexer: the same
 * parse tree the structural editors ({@link StructuralApi}) walk drives coloring
 * too, so a caller gets grammar-accurate highlighting without shipping a
 * hand-rolled tokenizer.
 *
 * <p>The pack ships the {@code highlights.scm} TEXT ({@link
 * TreeSitterLanguagePack#getHighlightsQuery(String)}) but no query <em>executor</em>,
 * so the simple, kind-based rules are read into a {@code {node-kind → capture}}
 * map and the tree is labeled by node kind during a plain DFS. Field-scoped /
 * predicated patterns (e.g. {@code (call function: (id) @fn)}, {@code #match?})
 * are skipped — they need a real query engine — while literals / keywords /
 * comments / operators still get colored.
 *
 * <p>Coloring is driven by a caller-supplied {@code capture → SGR} theme so the
 * exact palette stays a rendering decision (a sensible {@link #DEFAULT_THEME} is
 * provided). Emitted escapes are zero-width and never cross a newline, so
 * column-aligned / line-oriented painters are unaffected.
 */
public final class Highlighter {

  private Highlighter() {}

  // ---------------------------------------------------------------------------
  // highlights.scm → {node-kind-or-literal-token → capture}
  // ---------------------------------------------------------------------------

  /** A bracketed alternation with a trailing capture: {@code [ … ] @cap}. */
  private static final Pattern GROUP =
      Pattern.compile("(?s)\\[([^\\[\\]]*?)\\]\\s*@([A-Za-z0-9_.]+)");
  /** A named node inside a group body: {@code (kind)}. */
  private static final Pattern GROUP_KIND = Pattern.compile("\\(([a-zA-Z_][a-zA-Z0-9_]*)\\)");
  /** An anonymous literal token inside a group body: {@code "lit"}. */
  private static final Pattern GROUP_LIT = Pattern.compile("\"((?:[^\"\\\\]|\\\\.)*)\"");
  /** A single named node with a capture: {@code (kind) @cap}. */
  private static final Pattern SINGLE_KIND =
      Pattern.compile("\\(([a-zA-Z_][a-zA-Z0-9_]*)\\)\\s*@([A-Za-z0-9_.]+)");
  /** A single anonymous token with a capture: {@code "lit" @cap}. */
  private static final Pattern SINGLE_LIT =
      Pattern.compile("\"((?:[^\"\\\\]|\\\\.)*)\"\\s*@([A-Za-z0-9_.]+)");

  /** Per-language cache of the parsed highlights map — the query text is static. */
  private static final Map<String, Map<String, String>> KIND_CAP_CACHE = new ConcurrentHashMap<>();

  /**
   * Core special forms / macros. The Clojure grammar's {@code highlights.scm}
   * classifies literals but NOT the head symbol of a list, so these are lit up
   * in the {@code keyword} slot — but only for a real {@code sym_lit} at a list
   * head (never a match inside a string / comment).
   */
  private static final Set<String> CLOJURE_SPECIAL = Set.of(
      "def", "defn", "defn-", "defmacro", "definline", "defmulti", "defmethod", "defprotocol",
      "defrecord", "deftype", "definterface", "let", "let*", "letfn", "fn", "fn*", "if", "if-not",
      "if-let", "if-some", "when", "when-not", "when-let", "when-some", "when-first", "do", "cond",
      "condp", "case", "for", "doseq", "dotimes", "while", "loop", "recur", "ns", "require",
      "import", "use", "refer", "try", "catch", "finally", "throw", "quote", "var", "new", "set!",
      "locking", "and", "or", "->", "->>", "as->", "some->", "some->>", "cond->", "cond->>",
      "binding", "with-open", "with-local-vars", "with-redefs", "declare", "reify", "proxy",
      "extend-type", "extend-protocol", "future", "delay", "lazy-seq", "comment", "assert", "doto");

  /**
   * A reasonable default {@code capture → ANSI SGR foreground} palette (standard
   * 8/16-color codes). Callers that theme their own terminal should pass a map to
   * {@link #highlightAnsi(String, String, Map)} instead.
   */
  public static final Map<String, String> DEFAULT_THEME = Map.of(
      "string", "31",
      "number", "34",
      "constant", "36",
      "constant.builtin", "35",
      "keyword", "35",
      "comment", "90",
      "escape", "35",
      "type", "36",
      "constructor", "36");

  /**
   * The parsed {@code {node-kind-or-literal-token → capture}} map for a language
   * (kind-based subset of its {@code highlights.scm}), cached. The capture names
   * carry NO leading {@code @}.
   *
   * @param language tree-sitter language name
   * @return the map (empty when the language ships no highlights query)
   * @throws TreeSitterLanguagePackRsException on a native processing error
   */
  public static Map<String, String> kindToCapture(final String language)
      throws TreeSitterLanguagePackRsException {
    final Map<String, String> cached = KIND_CAP_CACHE.get(language);
    if (cached != null) {
      return cached;
    }
    final Map<String, String> parsed = parseScm(TreeSitterLanguagePack.getHighlightsQuery(language));
    KIND_CAP_CACHE.put(language, parsed);
    return parsed;
  }

  private static Map<String, String> parseScm(final @Nullable String scm) {
    final Map<String, String> m = new HashMap<>();
    if (scm == null) {
      return m;
    }
    // grouped alternations: [ … ] @cap
    final Matcher g = GROUP.matcher(scm);
    while (g.find()) {
      final String body = g.group(1);
      final String cap = g.group(2);
      final Matcher gk = GROUP_KIND.matcher(body);
      while (gk.find()) {
        m.putIfAbsent(gk.group(1), cap);
      }
      final Matcher gl = GROUP_LIT.matcher(body);
      while (gl.find()) {
        m.putIfAbsent(gl.group(1), cap);
      }
    }
    // single named node: (kind) @cap
    final Matcher sk = SINGLE_KIND.matcher(scm);
    while (sk.find()) {
      m.putIfAbsent(sk.group(1), sk.group(2));
    }
    // single anonymous token: "lit" @cap
    final Matcher sl = SINGLE_LIT.matcher(scm);
    while (sl.find()) {
      m.putIfAbsent(sl.group(1), sl.group(2));
    }
    return m;
  }

  // ---------------------------------------------------------------------------
  // parse + walk + render
  // ---------------------------------------------------------------------------

  /**
   * ANSI-colorize {@code source} as tree-sitter {@code language} with the
   * {@link #DEFAULT_THEME} palette.
   *
   * @see #highlightAnsi(String, String, Map)
   */
  public static @Nullable String highlightAnsi(final String source, final String language)
      throws TreeSitterLanguagePackRsException {
    return highlightAnsi(source, language, DEFAULT_THEME);
  }

  /**
   * ANSI-colorize {@code source} as tree-sitter {@code language}, mapping each
   * grammar capture to a foreground SGR code via {@code captureToSgr}. The result
   * has the same newline structure as the input (ready to split into lines);
   * escapes are zero-width and never cross a line break.
   *
   * @param source       code to highlight
   * @param language     tree-sitter language name (e.g. {@code "clojure"})
   * @param captureToSgr {@code capture → SGR-foreground-code} theme (capture names
   *                     WITHOUT a leading {@code @}; a {@code "a.b"} capture falls
   *                     back to its {@code "a"} category)
   * @return the colored string, or {@code null} when the language ships no
   *         highlights query / the source is empty / parsing fails
   * @throws TreeSitterLanguagePackRsException on a native processing error
   */
  public static @Nullable String highlightAnsi(
      final String source, final String language, final Map<String, String> captureToSgr)
      throws TreeSitterLanguagePackRsException {
    if (source == null || source.isEmpty() || captureToSgr == null) {
      return null;
    }
    final Map<String, String> kindCap = kindToCapture(language);
    if (kindCap.isEmpty()) {
      return null;
    }
    final byte[] bs = source.getBytes(StandardCharsets.UTF_8);
    final int n = bs.length;
    final String[] caps = new String[n];
    final boolean clj = "clojure".equals(language);
    try (Parser parser = TreeSitterLanguagePack.getParser(language)) {
      final Optional<Tree> parsed = parser.parse(source);
      if (parsed.isEmpty()) {
        return null;
      }
      try (Tree tree = parsed.get(); Node root = tree.rootNode()) {
        walk(root, bs, kindCap, caps, clj);
      }
    }
    return renderAnsi(bs, caps, n, captureToSgr);
  }

  private static void walk(final Node nd, final byte[] bs, final Map<String, String> kindCap,
      final String[] caps, final boolean clj) throws TreeSitterLanguagePackRsException {
    final String k = nd.kind();
    final String cap = kindCap.get(k);
    if (cap != null) {
      fill(caps, (int) nd.startByte(), (int) nd.endByte(), cap);
    }
    // Clojure: the head symbol of a list is not classified by the grammar —
    // light known special forms in the keyword slot.
    if (clj && "list_lit".equals(k)) {
      final Optional<Node> head = nd.namedChild(0);
      if (head.isPresent()) {
        try (Node c0 = head.get()) {
          if ("sym_lit".equals(c0.kind())) {
            final int sb = (int) c0.startByte();
            final int eb = (int) c0.endByte();
            final String t = new String(bs, sb, eb - sb, StandardCharsets.UTF_8);
            if (CLOJURE_SPECIAL.contains(t)) {
              fill(caps, sb, eb, "keyword");
            }
          }
        }
      }
    }
    final long count = nd.childCount();
    for (long i = 0; i < count; i++) {
      final Optional<Node> child = nd.child((int) i);
      if (child.isPresent()) {
        try (Node c = child.get()) {
          walk(c, bs, kindCap, caps, clj);
        }
      }
    }
  }

  private static void fill(final String[] caps, final int start, final int end, final String cap) {
    final int e = Math.min(end, caps.length);
    for (int i = start; i < e; i++) {
      caps[i] = cap;
    }
  }

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
   * Emit consecutive bytes of the same SGR as one {@code \u001b[<code>m…\u001b[0m}
   * run; runs never cross a newline (each line stays self-contained for a
   * line-oriented painter), and byte slices decode as UTF-8 so multibyte glyphs
   * survive.
   */
  private static String renderAnsi(final byte[] bs, final String[] caps, final int n,
      final Map<String, String> theme) {
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

  private static void flush(final StringBuilder sb, final byte[] bs, final int start, final int end,
      final @Nullable String sgr) {
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
