package dev.kreuzberg.treesitterlanguagepack;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;
import org.jspecify.annotations.Nullable;

/**
 * Unified structural editing over tree-sitter, for every language the pack understands (Clojure included). Target a definition by NAME and
 * replace or insert around it; the engine locates the node from the structural outline, splices its line span, re-parses the result, and
 * REFUSES the edit if it introduces a syntax error — so a write never corrupts a file.
 *
 * <p>
 * This is the language-neutral, JVM-native replacement for per-language structural editors (e.g. a Clojure-only rewrite-clj path): the same
 * locate-by-name editing for any supported language, with parse validation.
 */
public final class StructuralApi {

    private StructuralApi() {
    }

    /** What to do at the located definition. */
    public enum Op {
        /** Replace the whole definition. */
        REPLACE,
        /** Insert {@code code} immediately before the definition. */
        INSERT_BEFORE,
        /** Insert {@code code} immediately after the definition. */
        INSERT_AFTER,
        /** Append {@code code} at end of file (ignores the target). */
        APPEND,
        /**
         * Replace the existing doc string of the target definition. {@code code} is the full replacement doc literal (e.g.
         * {@code "\"New doc.\""}).
         */
        REPLACE_DOC,
        /**
         * Add a doc string to a definition that has none. {@code code} is the doc literal; it is placed at the language's idiomatic spot
         * (e.g. after a Clojure {@code defn} name). Refuses if a doc already exists (use {@link #REPLACE_DOC}) or for languages without a
         * wired placement.
         */
        ADD_DOC
    }

    /** A located definition: name, kind, and 1-based inclusive line span. */
    public record Target(@Nullable String name, String kind, int startLine, int endLine) {
    }

    /** Raised when an edit cannot be applied (missing/ambiguous target, bad result). */
    public static final class EditException extends RuntimeException {
        private static final long serialVersionUID = 1L;

        /**
         * @param message
         *            human-readable, actionable reason.
         */
        public EditException(final String message) {
            super(message);
        }
    }

    /**
     * Depth-first structural outline: every definition as a {@link Target} with 1-based inclusive line numbers.
     *
     * @param source
     *            file contents
     * @param language
     *            tree-sitter language name (e.g. {@code "clojure"})
     * @return the outline (empty when the language has no structure extraction)
     * @throws TreeSitterLanguagePackRsException
     *             on a native processing error
     */
    public static List<Target> outline(final String source, final String language) throws TreeSitterLanguagePackRsException {
        final ProcessConfig cfg = ProcessConfig.builder().withLanguage(language).withStructure(true).build();
        final ProcessResult res = TreeSitterLanguagePack.process(source, cfg);
        final List<Target> out = new ArrayList<>();
        flatten(res.structure(), out, source.split("\n", -1));
        return out;
    }

    private static void flatten(final @Nullable List<StructureItem> items, final List<Target> out, final String[] lines) {
        if (items == null) {
            return;
        }
        for (final StructureItem it : items) {
            final Span s = it.span();
            out.add(new Target(it.name(), kindOf(it), (int) s.startLine() + 1, endLineOf(s, lines)));
            flatten(it.children(), out, lines);
        }
    }

    /**
     * The 1-based LAST CONTENT line of a span. tree-sitter end positions are exclusive, and several grammars let a definition node swallow
     * its terminating newline and even the blank lines up to the next sibling (Groovy's {@code command}). Line-based edits must not inherit
     * those extra lines: splicing them away deletes whatever definition starts there.
     *
     * @param s
     *            the span to convert
     * @param lines
     *            the source split on {@code \n}
     * @return 1-based inclusive last line the span actually has content on
     */
    private static int endLineOf(final Span s, final String[] lines) {
        final int start = (int) s.startLine() + 1;
        int end = (int) s.endLine() + 1;
        if (s.endColumn() == 0 && end > start) {
            end--;
        }
        while (end > start && (end > lines.length || lines[end - 1].isBlank())) {
            end--;
        }
        return end;
    }

    /**
     * The item's kind as a user-facing word: the {@code StructureKind.Other} label when the item carries one (a GraphQL {@code type}, a
     * Terraform {@code resource}, an Elixir {@code Macro}), else the enum's own value. Without the label every language-specific construct
     * flattens into one indistinguishable {@code "other"}.
     */
    private static String kindOf(final StructureItem it) {
        final String label = it.kindLabel();
        if (label != null && !label.isBlank()) {
            return label;
        }
        return it.kind() == null ? "" : it.kind().getValue();
    }

    /**
     * Kind filter: matches the enum value OR the {@code Other} label, so both {@code "other"} and {@code "resource"} select a Terraform
     * resource.
     */
    private static boolean kindMatches(final StructureItem it, final @Nullable String kind) {
        if (kind == null) {
            return true;
        }
        final String enumValue = it.kind() == null ? "" : it.kind().getValue();
        final String label = it.kindLabel();
        return kind.equalsIgnoreCase(enumValue) || (label != null && kind.equalsIgnoreCase(label));
    }

    /**
     * Structural edit by definition name. {@link Op#APPEND} ignores {@code target}; the others locate the definition by {@code target} (and
     * optional {@code kind} to disambiguate same-named definitions). The returned content is guaranteed to parse.
     *
     * @param source
     *            current file contents
     * @param language
     *            tree-sitter language name
     * @param op
     *            what to do at the located node
     * @param target
     *            definition name (ignored for {@link Op#APPEND})
     * @param kind
     *            optional kind filter (e.g. {@code "function"}, {@code "method"})
     * @param code
     *            replacement / inserted source
     * @return the new file contents
     * @throws EditException
     *             target missing/ambiguous, or result has a syntax error
     * @throws TreeSitterLanguagePackRsException
     *             on a native processing error
     */
    public static String edit(final String source, final String language, final Op op, final @Nullable String target,
            final @Nullable String kind, final String code) throws TreeSitterLanguagePackRsException {
        if (code == null) {
            throw new EditException("structural edit requires non-null code");
        }
        // ADD_DOC allows a null target: it inserts a module-level docstring (see addDoc's
        // DOC_IN_BODY branch). Every other locating op still needs a target name.
        if (op != Op.APPEND && op != Op.ADD_DOC && (target == null || target.isBlank())) {
            throw new EditException("The " + op + " structural edit needs a `target` definition name"
                    + " (or a path/anchor locator); none was supplied. List the file's definitions to" + " see valid names.");
        }
        if (op == Op.REPLACE_DOC) {
            return replaceDoc(source, language, target, code);
        }
        if (op == Op.ADD_DOC) {
            return addDoc(source, language, target, code);
        }
        final List<String> lines = new ArrayList<>(Arrays.asList(source.split("\n", -1)));
        final int start;
        final int end;
        if (op == Op.APPEND) {
            start = 0;
            end = 0;
        } else {
            final Target t = locate(outline(source, language), target, kind);
            start = t.startLine();
            end = t.endLine();
        }
        final String result = splice(lines, op, start, end, code);
        final List<Diagnostic> errors = errorDiagnostics(result, language);
        if (!errors.isEmpty()) {
            throw new EditException("Edit rejected: it introduces " + errors.size() + " syntax error(s); the file was not changed.");
        }
        return result;
    }

    private static Target locate(final List<Target> items, final @Nullable String target, final @Nullable String kind) {
        final List<Target> matches = new ArrayList<>();
        for (final Target t : items) {
            if (Objects.equals(t.name(), target) && (kind == null || kind.equalsIgnoreCase(t.kind()))) {
                matches.add(t);
            }
        }
        if (matches.isEmpty()) {
            throw new EditException("No definition named '" + target + "'" + (kind == null ? "" : " of kind " + kind)
                    + " found. List the file's definitions to see valid names.");
        }
        if (matches.size() > 1) {
            final StringBuilder kinds = new StringBuilder();
            for (final Target t : matches) {
                if (kinds.length() > 0) {
                    kinds.append(", ");
                }
                kinds.append(t.kind());
            }
            throw new EditException(matches.size() + " definitions named '" + target + "' — pass kind to disambiguate (" + kinds + ").");
        }
        return matches.get(0);
    }

    private static String splice(final List<String> lines, final Op op, final int start, final int end, final String code) {
        // Splitting on "\n" leaves each CRLF line ending as a trailing "\r" on the line.
        // Inserted code and blank separators come in LF-only, so a CRLF file would end up
        // with mixed endings — stamp them to match the file instead.
        final boolean crlf = lines.size() > 1 && lines.get(0).endsWith("\r");
        final String blank = crlf ? "\r" : "";
        List<String> body = stripBlankLines(code);
        if (crlf) {
            final List<String> stamped = new ArrayList<>(body.size());
            for (final String l : body) {
                stamped.add(l.endsWith("\r") ? l : l + "\r");
            }
            body = stamped;
        }
        // `source.split("\n", -1)` leaves a trailing "" element for a file that ends in a
        // newline. Seam normalisation (trimTrailingBlanks / trimLeadingBlanks) can eat that
        // element — INSERT_AFTER on the LAST definition trims the whole tail away — which
        // would silently strip the file's final newline (and, in Groovy, whose grammar
        // requires a terminating newline, make the edit fail the syntax gate outright).
        final boolean hadFinalNewline = !lines.isEmpty() && lines.get(lines.size() - 1).isEmpty();
        final List<String> out = new ArrayList<>(lines.size() + body.size() + 2);
        switch (op) {
            case APPEND -> {
                out.addAll(lines);
                // A source ending in "\n" leaves a trailing "" element. Normalise trailing
                // blanks away, then append exactly one blank-line separator before the new
                // node and keep a final newline — so an appended defn is never glued onto
                // the previous form.
                trimTrailingBlanks(out);
                if (!out.isEmpty()) {
                    out.add(blank);
                }
                out.addAll(body);
                out.add(blank);
            }
            case REPLACE -> {
                out.addAll(lines.subList(0, start - 1));
                out.addAll(body);
                out.addAll(lines.subList(end, lines.size()));
            }
            case INSERT_BEFORE -> {
                final int before = start - 1; // 0-based index of the target's first line
                final List<String> head = new ArrayList<>(lines.subList(0, before));
                final List<String> tail = new ArrayList<>(lines.subList(before, lines.size()));
                // Normalise any run of blank lines at the seam down to exactly one — never
                // zero (glued) and never two-or-more (double blank), whether the extra
                // blanks came from us or were already sitting in the source.
                trimTrailingBlanks(head);
                if (!head.isEmpty()) {
                    out.addAll(head);
                    out.add(blank); // exactly one blank above the inserted node
                }
                out.addAll(body);
                out.add(blank); // exactly one blank between the inserted node and the target
                out.addAll(tail);
            }
            case INSERT_AFTER -> {
                final List<String> head = new ArrayList<>(lines.subList(0, end));
                final List<String> tail = new ArrayList<>(lines.subList(end, lines.size()));
                trimTrailingBlanks(head);
                trimLeadingBlanks(tail);
                out.addAll(head);
                out.add(blank); // exactly one blank between the target and the inserted node
                out.addAll(body);
                if (!tail.isEmpty()) {
                    out.add(blank); // exactly one blank below the inserted node
                }
                out.addAll(tail);
            }
            default -> throw new EditException("Unknown op: " + op);
        }
        // Restore a final newline the source had (never remove one it lacked).
        if (hadFinalNewline && (out.isEmpty() || !out.get(out.size() - 1).isEmpty())) {
            out.add(""); // the terminator itself is just the empty trailing element
        }
        return String.join("\n", out);
    }

    /** Drop trailing blank lines from a mutable list, in place. */
    private static void trimTrailingBlanks(final List<String> l) {
        while (!l.isEmpty() && l.get(l.size() - 1).isBlank()) {
            l.remove(l.size() - 1);
        }
    }

    /** Drop leading blank lines from a mutable list, in place. */
    private static void trimLeadingBlanks(final List<String> l) {
        while (!l.isEmpty() && l.get(0).isBlank()) {
            l.remove(0);
        }
    }

    /**
     * Split {@code code} into lines with its leading and trailing blank lines stripped, so an inserted node never carries its own edge
     * blanks into the seam (internal blank lines are preserved).
     */
    private static List<String> stripBlankLines(final String code) {
        final List<String> l = new ArrayList<>(Arrays.asList(code.split("\n", -1)));
        trimLeadingBlanks(l);
        trimTrailingBlanks(l);
        return l;
    }

    /**
     * Byte span [start, end) of the COMMENT BLOCK that documents {@code target} — the run of comment nodes sitting directly above the
     * definition with no blank line in between — or {@code null} when there is none.
     *
     * <p>
     * Most languages carry a definition's doc as such a leading comment, and the native docstring extractor only reports docs that are a
     * real string node INSIDE the definition (Python, Clojure). Without this, {@code add_doc} could never see an existing comment doc (so
     * it stacked a second one) and {@code replace_doc} could never edit one.
     */
    private static int @Nullable [] docCommentSpan(final String source, final String language, final @Nullable String target)
            throws TreeSitterLanguagePackRsException {
        if (target == null || target.isBlank()) {
            return null;
        }
        final int defStart;
        try {
            defStart = locateDef(source, language, target).startByte();
        } catch (final EditException e) {
            return null;
        }
        final byte[] src = source.getBytes(StandardCharsets.UTF_8);
        final List<int[]> comments = new ArrayList<>();
        final int[] rawRegion = new int[]{-1, -1};
        try (Parser parser = TreeSitterLanguagePack.getParser(language);
                Tree tree = parser.parse(source).orElseThrow(() -> new EditException("could not parse " + language + " source"));
                Node root = tree.rootNode()) {
            collectComments(root, defStart, comments);
            findRawTextRegion(root, defStart, rawRegion);
        }
        comments.sort((a, b) -> Integer.compare(a[0], b[0]));
        int start = -1;
        int end = -1;
        for (int i = comments.size() - 1; i >= 0; i--) {
            final int[] c = comments.get(i);
            int gapTo = start < 0 ? defStart : start;
            // A definition's own start can sit BEHIND modifiers on its line (`export function`,
            // `pub fn`, `public static void`), so measure the gap to the start of that LINE.
            final int lineStart = lineStartOf(src, gapTo);
            if (lineStart >= c[1]) {
                gapTo = lineStart;
            }
            if (c[1] > gapTo || !isDocGap(src, c[1], gapTo)) {
                break;
            }
            start = c[0];
            if (end < 0) {
                // Some grammars (Rust doc comments) end a comment node ON the newline that
                // terminates it; splicing that away would glue the doc to the definition.
                int e = c[1];
                while (e > c[0] && isAsciiWhitespace(src[e - 1])) {
                    e--;
                }
                end = e;
            }
        }
        if (start < 0 && rawRegion[0] >= 0) {
            // The definition lives in a region the HOST grammar leaves unparsed (a Svelte/Vue
            // <script> body is one raw_text node), so its doc comment is not a comment node in
            // this tree at all. Read the comment lines textually, inside that region only.
            return rawTextDocComment(src, rawRegion[0], defStart);
        }
        return start < 0 ? null : new int[]{start, end};
    }

    /** Index of the first byte of the line holding {@code pos}. */
    private static int lineStartOf(final byte[] src, final int pos) {
        int i = Math.min(pos, src.length);
        while (i > 0 && src[i - 1] != '\n') {
            i--;
        }
        return i;
    }

    /** Openers a doc comment can start with in the embedded languages we leave unparsed. */
    private static final String[] COMMENT_OPENERS = {"//", "#", "--", ";", "/*", "*", "<!--", "%"};

    /**
     * Byte span of the innermost RAW-TEXT leaf holding {@code pos} — a region the host grammar keeps as one opaque token (Svelte/Vue script
     * bodies, template literals). Written into {@code out} as {start, end}, left untouched when there is none.
     */
    private static void findRawTextRegion(final Node node, final int pos, final int[] out) throws TreeSitterLanguagePackRsException {
        if (node.startByte() > pos || node.endByte() <= pos) {
            return;
        }
        if (node.namedChildCount() == 0 && node.kind().toLowerCase(java.util.Locale.ROOT).contains("raw_text")) {
            out[0] = (int) node.startByte();
            out[1] = (int) node.endByte();
            return;
        }
        final int count = (int) node.childCount();
        for (int i = 0; i < count; i++) {
            final java.util.Optional<Node> child = node.child(i);
            if (child.isPresent()) {
                try (Node c = child.get()) {
                    findRawTextRegion(c, pos, out);
                }
            }
        }
    }

    /**
     * Span of the run of comment-looking LINES directly above {@code defStart}, bounded by {@code regionStart}. Purely textual on purpose:
     * inside a raw-text region there is no tree to ask, and the alternative is stacking a second doc on a documented definition.
     */
    private static int @Nullable [] rawTextDocComment(final byte[] src, final int rawStart, final int defStart) {
        // A raw_text node can begin AFTER the indentation of its first line, so bound the scan
        // by that line's start instead — never earlier, so the host markup is never touched.
        final int regionStart = lineStartOf(src, rawStart);
        int lineStart = lineStartOf(src, defStart);
        int start = -1;
        int end = -1;
        while (lineStart > regionStart) {
            final int prevEnd = lineStart - 1;
            final int prevStart = lineStartOf(src, prevEnd);
            if (prevStart < regionStart) {
                break;
            }
            final String line = new String(src, prevStart, prevEnd - prevStart, StandardCharsets.UTF_8).trim();
            boolean isComment = false;
            for (final String opener : COMMENT_OPENERS) {
                if (!line.startsWith(opener)) {
                    continue;
                }
                // `#`, `--`, `*` and `%` also open real code (a JS private field, a decrement,
                // a multiplication), so those only count as a comment when a space follows.
                final boolean needsSpace = "#".equals(opener) || "--".equals(opener) || "*".equals(opener) || "%".equals(opener);
                if (needsSpace && line.length() > opener.length() && !isAsciiWhitespace((byte) line.charAt(opener.length()))) {
                    continue;
                }
                isComment = true;
                break;
            }
            if (!isComment) {
                break;
            }
            int contentStart = prevStart;
            while (contentStart < prevEnd && isAsciiWhitespace(src[contentStart])) {
                contentStart++;
            }
            int contentEnd = prevEnd;
            while (contentEnd > contentStart && isAsciiWhitespace(src[contentEnd - 1])) {
                contentEnd--;
            }
            start = contentStart;
            if (end < 0) {
                end = contentEnd;
            }
            lineStart = prevStart;
        }
        return start < 0 ? null : new int[]{start, end};
    }

    /** Collect the byte spans of every comment node ending at or before {@code limit}. */
    private static void collectComments(final Node node, final int limit, final List<int[]> out) throws TreeSitterLanguagePackRsException {
        if (node.startByte() > limit) {
            return;
        }
        if (node.kind().toLowerCase(java.util.Locale.ROOT).contains("comment") && node.endByte() <= limit) {
            out.add(new int[]{(int) node.startByte(), (int) node.endByte()});
            return;
        }
        final int count = (int) node.childCount();
        for (int i = 0; i < count; i++) {
            final java.util.Optional<Node> child = node.child(i);
            if (child.isPresent()) {
                try (Node c = child.get()) {
                    collectComments(c, limit, out);
                }
            }
        }
    }

    /**
     * True when everything between {@code from} and {@code to} is whitespace holding at most ONE line break — i.e. the comment hugs what
     * follows it, so it reads as its doc rather than as a detached remark.
     */
    private static boolean isDocGap(final byte[] src, final int from, final int to) {
        int newlines = 0;
        for (int i = from; i < to; i++) {
            if (!isAsciiWhitespace(src[i])) {
                return false;
            }
            if (src[i] == '\n' && ++newlines > 1) {
                return false;
            }
        }
        return true;
    }

    /**
     * Replace the existing doc string of {@code target} (byte-precise, so inline docs are handled), falling back to its leading comment
     * block. Throws if the definition has no doc at all.
     */
    private static String replaceDoc(final String source, final String language, final @Nullable String target, final String code)
            throws TreeSitterLanguagePackRsException {
        final ProcessConfig cfg = ProcessConfig.builder().withLanguage(language).withDocstrings(true).build();
        final ProcessResult res = TreeSitterLanguagePack.process(source, cfg);
        final List<DocstringInfo> docs = res.docstrings();
        DocstringInfo match = null;
        if (docs != null) {
            for (final DocstringInfo d : docs) {
                if (Objects.equals(d.associatedItem(), target)) {
                    match = d;
                    break;
                }
            }
        }
        final int docStart;
        final int docEnd;
        if (match == null) {
            final int[] commentSpan = docCommentSpan(source, language, target);
            if (commentSpan == null) {
                throw new EditException("No existing doc string for '" + target
                        + "'. Add one by replacing the whole definition (struct_edit replace) with code that includes the doc.");
            }
            docStart = commentSpan[0];
            docEnd = commentSpan[1];
        } else {
            final Span s = match.span();
            docStart = (int) s.startByte();
            docEnd = (int) s.endByte();
        }
        final String result = spliceBytes(source, docStart, docEnd, code);
        final List<Diagnostic> errors = errorDiagnostics(result, language);
        if (!errors.isEmpty()) {
            throw new EditException(
                    "Doc replacement rejected: it introduces " + errors.size() + " syntax error(s); the file was not changed.");
        }
        return result;
    }

    /**
     * Structurally replace a sub-expression: find the unique syntax node — or the unique contiguous run of sibling nodes — whose text
     * equals {@code match} (optionally scoped to definition {@code target}), and replace it with {@code code}. Unlike a raw-text patch this
     * matches whole nodes at real syntax boundaries (never inside a string/comment or a partial token), so a snippet spanning several
     * adjacent forms (e.g. some let-binding pairs) works too. It refuses to act if the match is not unique.
     *
     * @param source
     *            current file contents
     * @param language
     *            tree-sitter language name
     * @param match
     *            the snippet identifying the node to replace (end-trimmed)
     * @param code
     *            replacement source
     * @param target
     *            optional definition name to scope the search within
     * @param kind
     *            optional kind filter for {@code target}
     * @return the new file contents
     * @throws EditException
     *             no match, ambiguous match, or syntax-broken result
     * @throws TreeSitterLanguagePackRsException
     *             on a native processing error
     */
    public static String replaceNode(final String source, final String language, final String match, final String code,
            final @Nullable String target, final @Nullable String kind) throws TreeSitterLanguagePackRsException {
        if (match == null || code == null) {
            throw new EditException("replaceNode requires both match and code");
        }
        final String needle = match.strip();
        if (needle.isEmpty()) {
            throw new EditException("replaceNode match must be non-blank");
        }
        final long[] scope = target == null ? new long[]{0L, Long.MAX_VALUE} : targetByteSpan(source, language, target, kind);
        final byte[] srcBytes = source.getBytes(StandardCharsets.UTF_8);
        // Fuzzy match: compare whitespace-normalised text, so the snippet need not
        // reproduce the file's exact indentation / line breaks.
        final String normNeedle = normalizeWs(needle);
        final List<int[]> hits = new ArrayList<>();
        try (Parser parser = TreeSitterLanguagePack.getParser(language);
                Tree tree = parser.parse(source).orElseThrow(() -> new EditException("could not parse " + language + " source"));
                Node root = tree.rootNode()) {
            collectMatches(root, srcBytes, normNeedle, scope[0], scope[1], hits);
        }
        if (hits.isEmpty()) {
            throw new EditException("No node matching the snippet" + (target == null ? "" : " inside '" + target + "'") + " was found.");
        }
        if (hits.size() > 1) {
            throw new EditException(hits.size() + " nodes match the snippet"
                    + (target == null ? " — scope it with target, or" : " inside '" + target + "' —")
                    + " make the snippet more specific. Refusing to guess.");
        }
        final int[] hit = hits.get(0);
        // The match is whitespace-NORMALISED, so the winning node's byte span can reach
        // past the snippet's own text — several grammars (Groovy's `command`, Elixir's
        // top-level call) end a definition node ON the following newline. Splicing that
        // span would swallow the separator: the next definition ends up glued to the
        // replacement, or the file loses its final newline. Shrink the hit to its
        // non-whitespace core; surrounding whitespace is the file's, not the snippet's.
        int hitStart = hit[0];
        int hitEnd = hit[1];
        while (hitStart < hitEnd && isAsciiWhitespace(srcBytes[hitStart])) {
            hitStart++;
        }
        while (hitEnd > hitStart && isAsciiWhitespace(srcBytes[hitEnd - 1])) {
            hitEnd--;
        }
        final String result = spliceBytes(source, hitStart, hitEnd, code);
        final List<Diagnostic> errors = errorDiagnostics(result, language);
        if (!errors.isEmpty()) {
            throw new EditException("Edit rejected: it introduces " + errors.size() + " syntax error(s); the file was not changed.");
        }
        return result;
    }

    /** UTF-8 keeps ASCII whitespace unambiguous, so a byte test is safe here. */
    private static boolean isAsciiWhitespace(final byte b) {
        return b == ' ' || b == '\t' || b == '\n' || b == '\r' || b == '\f' || b == 0x0B;
    }

    private static void collectMatches(final Node node, final byte[] src, final String normNeedle, final long start, final long end,
            final List<int[]> hits) throws TreeSitterLanguagePackRsException {
        final int sb = (int) node.startByte();
        final int eb = (int) node.endByte();
        if (sb >= start && eb <= end && normalizeWs(new String(src, sb, eb - sb, StandardCharsets.UTF_8)).equals(normNeedle)) {
            hits.add(new int[]{sb, eb});
            return; // a matched node's children can't be a distinct match of the same text
        }
        final int count = (int) node.childCount();
        // Record child byte spans while recursing, so we can additionally match a
        // CONTIGUOUS RUN of sibling nodes whose combined text equals the snippet —
        // e.g. several let-binding pairs or statements that are not, on their own, a
        // single syntax node. Single children are already covered by the recursion.
        final int[] cstart = new int[count];
        final int[] cend = new int[count];
        for (int i = 0; i < count; i++) {
            final java.util.Optional<Node> child = node.child(i);
            if (child.isPresent()) {
                try (Node c = child.get()) {
                    cstart[i] = (int) c.startByte();
                    cend[i] = (int) c.endByte();
                    collectMatches(c, src, normNeedle, start, end, hits);
                }
            } else {
                cstart[i] = -1;
                cend[i] = -1;
            }
        }
        // Windows of two or more adjacent siblings (start child i, end child j).
        for (int i = 0; i < count; i++) {
            if (cstart[i] < 0 || cstart[i] < start) {
                continue;
            }
            for (int j = i + 1; j < count; j++) {
                if (cend[j] < 0) {
                    break; // a gap breaks the contiguous run
                }
                if (cend[j] > end) {
                    break; // spans past the scope; wider windows only grow
                }
                final int ws = cstart[i];
                final int we = cend[j];
                if (we - ws < normNeedle.length()) {
                    continue; // too short to equal the snippet even after ws-normalisation
                }
                if (normalizeWs(new String(src, ws, we - ws, StandardCharsets.UTF_8)).equals(normNeedle)) {
                    hits.add(new int[]{ws, we});
                }
            }
        }
    }

    /** Trim and collapse internal whitespace runs to a single space (fuzzy match). */
    private static String normalizeWs(final String s) {
        return s.strip().replaceAll("\\s+", " ");
    }

    private static long[] targetByteSpan(final String source, final String language, final String target, final @Nullable String kind)
            throws TreeSitterLanguagePackRsException {
        final ProcessConfig cfg = ProcessConfig.builder().withLanguage(language).withStructure(true).build();
        final ProcessResult res = TreeSitterLanguagePack.process(source, cfg);
        final List<long[]> spans = new ArrayList<>();
        findSpans(res.structure(), target, kind, spans);
        if (spans.isEmpty()) {
            throw new EditException("No definition named '" + target + "' to scope within.");
        }
        if (spans.size() > 1) {
            throw new EditException(spans.size() + " definitions named '" + target + "' — pass kind to scope the search.");
        }
        return spans.get(0);
    }

    private static void findSpans(final @Nullable List<StructureItem> items, final String target, final @Nullable String kind,
            final List<long[]> out) {
        if (items == null) {
            return;
        }
        for (final StructureItem it : items) {
            if (Objects.equals(it.name(), target) && kindMatches(it, kind)) {
                final Span s = it.span();
                out.add(new long[]{s.startByte(), s.endByte()});
            }
            findSpans(it.children(), target, kind, out);
        }
    }

    /** Replace the UTF-8 byte range [start, end) of {@code source} with {@code code}. */
    private static String spliceBytes(final String source, final int startByte, final int endByte, final String code) {
        final byte[] bytes = source.getBytes(StandardCharsets.UTF_8);
        final String before = new String(bytes, 0, startByte, StandardCharsets.UTF_8);
        final String after = new String(bytes, endByte, bytes.length - endByte, StandardCharsets.UTF_8);
        return before + code + after;
    }

    /** Languages whose doc string sits as the first statement INSIDE the body. */
    private static final java.util.Set<String> DOC_IN_BODY = java.util.Set.of("python");
    /** Lisp-family languages whose doc string sits right AFTER the def name. */
    private static final java.util.Set<String> DOC_AFTER_NAME = java.util.Set.of("clojure");

    /**
     * Add a doc string to {@code target} when it has none, at the language's idiomatic spot:
     * <ul>
     * <li>lisps (Clojure): right after the def-form name;</li>
     * <li>Python: as the first statement inside the body;</li>
     * <li>everything else: as a comment on the line above the definition — {@code code} must be the comment written in that language's own
     * comment syntax (a line comment, block comment, or doc comment).</li>
     * </ul>
     * Refuses when a doc already exists (use {@link Op#REPLACE_DOC}); the result is re-parsed and rejected on any syntax error.
     */
    private static String addDoc(final String source, final String language, final @Nullable String target, final String code)
            throws TreeSitterLanguagePackRsException {
        // Refuse if there is already a doc — adding would duplicate it.
        final ProcessConfig dcfg = ProcessConfig.builder().withLanguage(language).withDocstrings(true).build();
        final ProcessResult dres = TreeSitterLanguagePack.process(source, dcfg);
        if (dres.docstrings() != null) {
            for (final DocstringInfo d : dres.docstrings()) {
                if (Objects.equals(d.associatedItem(), target)) {
                    throw new EditException("'" + target + "' already has a doc string — use replace_doc to change it.");
                }
            }
        }
        if (docCommentSpan(source, language, target) != null) {
            throw new EditException("'" + target + "' already has a doc comment — use replace_doc to change it.");
        }

        final byte[] src = source.getBytes(StandardCharsets.UTF_8);
        final String result;
        if (DOC_AFTER_NAME.contains(language)) {
            final int at = clojureNameEndByte(source, target);
            if (at < 0) {
                throw new EditException("No def-form named '" + target + "' to add a doc string to.");
            }
            result = spliceBytes(source, at, at, " " + code);
        } else if (DOC_IN_BODY.contains(language) && target == null) {
            // Module-level docstring: the first statement of the file. It may follow a
            // shebang and/or comments (those are not statements), so insert after a
            // leading shebang line; otherwise at the very top.
            int ins = 0;
            if (src.length >= 2 && src[0] == '#' && src[1] == '!') {
                while (ins < src.length && src[ins] != '\n') {
                    ins++;
                }
                if (ins < src.length) {
                    ins++; // past the newline
                }
            }
            result = spliceBytes(source, ins, ins, code + "\n\n");
        } else {
            final DefSpans def = locateDef(source, language, target);
            if (DOC_IN_BODY.contains(language)) {
                if (def.bodyStartByte() < 0) {
                    throw new EditException("'" + target + "' has no body to place a doc string in.");
                }
                final int at = def.bodyStartByte();
                // An inline suite (`def f(): return 1`) has its body on the SAME line as
                // the header. Splicing the doc before the body would dedent the body to
                // column 0 — tree-sitter still parses it, but Python rejects it
                // ('return' outside function). Rewrite the suite onto an indented block.
                boolean inline = true;
                for (int i = def.startByte(); i < at; i++) {
                    if (src[i] == '\n') {
                        inline = false;
                        break;
                    }
                }
                if (inline && def.bodyEndByte() > at) {
                    final String baseIndent = indentAt(src, startOfLine(src, def.startByte()));
                    final String bodyIndent = baseIndent + "    ";
                    final String bodyText = new String(src, at, def.bodyEndByte() - at, StandardCharsets.UTF_8);
                    result = spliceBytes(source, at, def.bodyEndByte(), "\n" + bodyIndent + code + "\n" + bodyIndent + bodyText);
                } else {
                    final String indent = indentAt(src, startOfLine(src, at));
                    result = spliceBytes(source, at, at, code + "\n" + indent);
                }
            } else {
                // comment-before: insert the comment line above the definition,
                // matching its indentation.
                final int lineStart = startOfLine(src, def.startByte());
                final String indent = indentAt(src, lineStart);
                result = spliceBytes(source, lineStart, lineStart, indent + code + "\n");
            }
        }

        final List<Diagnostic> errors = errorDiagnostics(result, language);
        if (!errors.isEmpty()) {
            throw new EditException("add_doc rejected: it introduces " + errors.size() + " syntax error(s); the file was not changed.");
        }
        return result;
    }

    /** Start byte of the definition and the [start, end) of its body (-1 if no body). */
    private record DefSpans(int startByte, int bodyStartByte, int bodyEndByte) {
    }

    private static DefSpans locateDef(final String source, final String language, final @Nullable String target)
            throws TreeSitterLanguagePackRsException {
        final ProcessConfig cfg = ProcessConfig.builder().withLanguage(language).withStructure(true).build();
        final ProcessResult res = TreeSitterLanguagePack.process(source, cfg);
        final List<DefSpans> found = new ArrayList<>();
        collectDefSpans(res.structure(), target, found);
        if (found.isEmpty()) {
            throw new EditException("No definition named '" + target + "' to add a doc string to.");
        }
        if (found.size() > 1) {
            throw new EditException(found.size() + " definitions named '" + target + "' — cannot pick one.");
        }
        return found.get(0);
    }

    private static void collectDefSpans(final @Nullable List<StructureItem> items, final @Nullable String target,
            final List<DefSpans> out) {
        if (items == null) {
            return;
        }
        for (final StructureItem it : items) {
            if (Objects.equals(it.name(), target)) {
                final int body = it.bodySpan() == null ? -1 : (int) it.bodySpan().startByte();
                final int bodyEnd = it.bodySpan() == null ? -1 : (int) it.bodySpan().endByte();
                out.add(new DefSpans((int) it.span().startByte(), body, bodyEnd));
            }
            collectDefSpans(it.children(), target, out);
        }
    }

    /** Byte index of the first byte of the line containing {@code pos}. */
    private static int startOfLine(final byte[] src, final int pos) {
        int i = pos;
        while (i > 0 && src[i - 1] != '\n') {
            i--;
        }
        return i;
    }

    /** Leading whitespace (indent) of the line beginning at {@code lineStart}. */
    private static String indentAt(final byte[] src, final int lineStart) {
        int i = lineStart;
        while (i < src.length && (src[i] == ' ' || src[i] == '\t')) {
            i++;
        }
        return new String(src, lineStart, i - lineStart, StandardCharsets.UTF_8);
    }

    /**
     * End byte of the NAME symbol of the Clojure def-form named {@code target} (the insertion point for a doc string), or -1 if not found.
     */
    private static int clojureNameEndByte(final String source, final @Nullable String target) throws TreeSitterLanguagePackRsException {
        final byte[] src = source.getBytes(StandardCharsets.UTF_8);
        try (Parser parser = TreeSitterLanguagePack.getParser("clojure");
                Tree tree = parser.parse(source).orElseThrow(() -> new EditException("could not parse clojure source"));
                Node root = tree.rootNode()) {
            return findClojureNameEnd(root, src, target);
        }
    }

    private static int findClojureNameEnd(final Node node, final byte[] src, final @Nullable String target)
            throws TreeSitterLanguagePackRsException {
        if ("list_lit".equals(node.kind())) {
            final List<Node> syms = new ArrayList<>();
            final long named = node.namedChildCount();
            for (long i = 0; i < named; i++) {
                final java.util.Optional<Node> ch = node.namedChild((int) i);
                if (ch.isPresent()) {
                    final Node c = ch.get();
                    if ("sym_lit".equals(c.kind())) {
                        syms.add(c);
                    } else {
                        c.close();
                    }
                }
            }
            try {
                if (syms.size() >= 2) {
                    final String head = byteText(src, syms.get(0));
                    final String name = byteText(src, syms.get(1));
                    if (DEF_FORMS.contains(head) && name.equals(target)) {
                        return (int) syms.get(1).endByte();
                    }
                }
            } finally {
                for (final Node s : syms) {
                    s.close();
                }
            }
        }
        final long count = node.childCount();
        for (long i = 0; i < count; i++) {
            final java.util.Optional<Node> child = node.child((int) i);
            if (child.isPresent()) {
                try (Node c = child.get()) {
                    final int hit = findClojureNameEnd(c, src, target);
                    if (hit >= 0) {
                        return hit;
                    }
                }
            }
        }
        return -1;
    }

    private static String byteText(final byte[] src, final Node node) throws TreeSitterLanguagePackRsException {
        final int sb = (int) node.startByte();
        final int eb = (int) node.endByte();
        return new String(src, sb, eb - sb, StandardCharsets.UTF_8);
    }

    private static final java.util.Set<String> DEF_FORMS = java.util.Set.of("defn", "defn-", "defmacro", "def", "defonce");

    private static List<Diagnostic> errorDiagnostics(final String source, final String language) throws TreeSitterLanguagePackRsException {
        final ProcessConfig cfg = ProcessConfig.builder().withLanguage(language).withDiagnostics(true).build();
        final ProcessResult res = TreeSitterLanguagePack.process(source, cfg);
        final List<Diagnostic> errors = new ArrayList<>();
        final List<Diagnostic> all = res.diagnostics();
        if (all != null) {
            for (final Diagnostic d : all) {
                if (d.severity() == DiagnosticSeverity.Error) {
                    errors.add(d);
                }
            }
        }
        return errors;
    }

    // ---------------------------------------------------------------------------
    // Symbol-aware operations (reuse the same parse tree — no new parsing).
    // ---------------------------------------------------------------------------

    /**
     * One occurrence of an identifier: byte range plus 1-based line / 0-based column. Both the byte range and the column count UTF-8 BYTES,
     * not characters, so a line with multi-byte text has a column past its character offset.
     */
    public record ReferenceHit(int startByte, int endByte, int line, int column) {
    }

    /**
     * Every occurrence of the identifier {@code name} in {@code source} — leaf tokens whose text equals {@code name}, so matches sit at
     * real identifier boundaries (never inside a larger token, string, or comment). No scope resolution, so shadowed / unrelated same-named
     * identifiers are included too.
     *
     * <p>
     * Tracing MANY identifiers through one file? Call {@link #findReferences(String, String, Collection)} instead: it scans ONCE for the
     * whole batch, where N single-name calls parse the file N times.
     *
     * @param source
     *            file contents
     * @param language
     *            tree-sitter language name
     * @param name
     *            identifier to find
     * @return occurrences in source order
     * @throws TreeSitterLanguagePackRsException
     *             on a native processing error
     */
    public static List<ReferenceHit> findReferences(final String source, final String language, final String name)
            throws TreeSitterLanguagePackRsException {
        if (name == null || name.isBlank()) {
            throw new EditException("findReferences requires a non-blank name");
        }
        final String needle = name.strip();
        return findReferences(source, language, List.of(needle)).get(needle);
    }

    /**
     * Every occurrence of EACH identifier in {@code names} — the batch form of {@link #findReferences(String, String, String)}, and the one
     * to use when a file is traced for many symbols at once (an index, a call graph, a rename preview).
     *
     * <p>
     * The scan itself runs in Rust: ONE parse and ONE tree walk serve the whole batch, matching is a hashed lookup behind a byte-length
     * sieve, and only the hits cross the FFI boundary. Cost is O(nodes) + O(hits) and barely moves with the number of names, where the
     * per-name form parses the file again for every single one.
     *
     * <p>
     * Identical matching rules to the single-name form.
     *
     * @param source
     *            file contents
     * @param language
     *            tree-sitter language name
     * @param names
     *            identifiers to find; blank entries and duplicates are dropped
     * @return name to occurrences in source order: one entry per distinct non-blank name, in input order, with an immutable empty list when
     *         it never occurs
     * @throws EditException
     *             when no non-blank name was given
     * @throws TreeSitterLanguagePackRsException
     *             on a native processing error
     */
    public static Map<String, List<ReferenceHit>> findReferences(final String source, final String language, final Collection<String> names)
            throws TreeSitterLanguagePackRsException {
        final Map<String, List<ReferenceHit>> hits = new LinkedHashMap<>();
        if (names != null) {
            for (final String n : names) {
                if (n != null && !n.isBlank()) {
                    // A shared immutable empty list, not a fresh ArrayList. An index traces every
                    // name it knows through every file, and in a real batch the vast majority of
                    // those name/file pairs have no hit at all — those buckets would be pure
                    // garbage. The mutable list is allocated below, when a hit actually lands.
                    hits.putIfAbsent(n.strip(), List.of());
                }
            }
        }
        if (hits.isEmpty()) {
            throw new EditException("findReferences requires at least one non-blank name");
        }
        final List<dev.kreuzberg.treesitterlanguagepack.ReferenceHit> found = TreeSitterLanguagePackRs.findReferences(source, language,
                List.copyOf(hits.keySet()));
        for (final dev.kreuzberg.treesitterlanguagepack.ReferenceHit hit : found) {
            hits.computeIfPresent(hit.name(), (name, bucket) -> {
                final List<ReferenceHit> target = bucket instanceof ArrayList ? bucket : new ArrayList<>();
                final Span span = hit.span();
                target.add(new ReferenceHit((int) span.startByte(), (int) span.endByte(), (int) span.startLine() + 1,
                        (int) span.startColumn()));
                return target;
            });
        }
        return hits;
    }

    /**
     * Rename every occurrence of the identifier {@code oldName} to {@code newName} (text/identifier based — see {@link #findReferences}).
     * Refuses when there is nothing to rename; the result is re-parsed and rejected on any syntax error. Occurrences inside string literals
     * and comments are NOT renamed — {@link #findReferences} skips them — while identifiers inside string INTERPOLATIONS are real
     * references and are renamed.
     *
     * @param source
     *            file contents
     * @param language
     *            tree-sitter language name
     * @param oldName
     *            identifier to rename
     * @param newName
     *            replacement identifier
     * @return the new file contents
     * @throws EditException
     *             no occurrences, or syntax-broken result
     * @throws TreeSitterLanguagePackRsException
     *             on a native processing error
     */
    public static String rename(final String source, final String language, final String oldName, final String newName)
            throws TreeSitterLanguagePackRsException {
        if (newName == null || newName.isBlank()) {
            throw new EditException("rename requires a non-blank newName");
        }
        final List<ReferenceHit> hits = findReferences(source, language, oldName);
        if (hits.isEmpty()) {
            throw new EditException("No occurrences of '" + oldName + "' to rename.");
        }
        // ONE pass over the UTF-8 bytes. Splicing per occurrence re-encoded AND re-decoded
        // the whole file for every hit, which is quadratic exactly where rename matters most
        // — a short name used hundreds of times in a large file. Copy the gaps between the
        // occurrences instead and write `newName` in each one's place.
        final byte[] src = source.getBytes(StandardCharsets.UTF_8);
        final byte[] replacement = newName.getBytes(StandardCharsets.UTF_8);
        // `findReferences` returns leaf tokens in source order, so occurrences are already
        // ascending and disjoint; skip anything that is not, rather than trusting the input
        // enough to compute a negative gap.
        final List<ReferenceHit> ordered = new ArrayList<>(hits.size());
        int guard = 0;
        for (final ReferenceHit h : hits) {
            if (h.startByte() >= guard && h.endByte() >= h.startByte() && h.endByte() <= src.length) {
                ordered.add(h);
                guard = h.endByte();
            }
        }
        int size = src.length;
        for (final ReferenceHit h : ordered) {
            size += replacement.length - (h.endByte() - h.startByte());
        }
        final byte[] renamed = new byte[size];
        int read = 0;
        int write = 0;
        for (final ReferenceHit h : ordered) {
            final int gap = h.startByte() - read;
            System.arraycopy(src, read, renamed, write, gap);
            write += gap;
            System.arraycopy(replacement, 0, renamed, write, replacement.length);
            write += replacement.length;
            read = h.endByte();
        }
        System.arraycopy(src, read, renamed, write, src.length - read);
        final String result = new String(renamed, StandardCharsets.UTF_8);
        final List<Diagnostic> errors = errorDiagnostics(result, language);
        if (!errors.isEmpty()) {
            throw new EditException("rename rejected: it introduces " + errors.size() + " syntax error(s); the file was not changed.");
        }
        return result;
    }

    // ---------------------------------------------------------------------------
    // Batch scanning over MANY FILES — the fan-out lives next to the parse.
    //
    // A single file's scan is already one parse and one walk (see the batch
    // `findReferences` above). What a repo-wide consumer does on top of that is
    // always the same: hold a worker pool, keep results in request order, keep one
    // bad file from sinking the run, and resolve every language BEFORE the threads
    // start. That policy belongs here, with the parse it is sized for, instead of
    // being reinvented (and mistuned) in every JVM caller.
    // ---------------------------------------------------------------------------

    /**
     * How many files {@link #mapParallel(List, Function)} walks at once: one worker per CPU, floor 2, ceiling 16.
     *
     * <p>
     * The parse itself is serialized process-wide (the tree cache's lock is also the serialization point third-party scanners with mutable
     * static state need), but everything around it is not: the content-addressed tree cache, the structure/reference walks, the FFI
     * boundary and the caller's own per-file work all run concurrently. So a repo-wide scan scales nearly linearly until the parse
     * serialization saturates, and past the core count the curve is flat — there is nothing to buy by oversubscribing.
     */
    private static final int SCAN_PARALLELISM = Math.max(2, Math.min(16, Runtime.getRuntime().availableProcessors()));

    /**
     * One file handed to a batch scan: its identity, the language to parse it as, and its contents.
     *
     * <p>
     * The caller reads the file. This library never touches the filesystem for you, so path confinement, encoding and unsaved-buffer
     * contents stay where they belong — with the host.
     *
     * @param path
     *            how the caller identifies this file; echoed back on the result row, never interpreted
     * @param language
     *            tree-sitter language name (e.g. {@code "clojure"})
     * @param source
     *            file contents
     */
    public record FileSource(String path, String language, String source) {

        /**
         * @throws NullPointerException
         *             when any component is null
         */
        public FileSource {
            Objects.requireNonNull(path, "path");
            Objects.requireNonNull(language, "language");
            Objects.requireNonNull(source, "source");
        }
    }

    /**
     * One file's outcome in {@link #findReferences(List, Collection)} — TOTAL: either references or the message of the failure THIS file
     * alone hit, never a thrown batch.
     *
     * @param path
     *            the {@link FileSource#path()} this row answers, echoed verbatim
     * @param references
     *            name to occurrences, exactly as the single-file batch form returns; empty when this file failed
     * @param error
     *            the failure message for this file, or null when it scanned
     */
    public record FileReferences(String path, Map<String, List<ReferenceHit>> references, @Nullable String error) {

        /** @return true when this file failed and {@link #references()} is therefore empty */
        public boolean isFailed() {
            return error != null;
        }
    }

    /**
     * {@code map} over {@code items} across {@link #SCAN_PARALLELISM} workers, in REQUEST ORDER.
     *
     * <p>
     * Workers pull the next index off a shared cursor, so one huge file cannot strand a worker while the others idle. The first failure
     * observed is rethrown AS THROWN — never wrapped in an {@code ExecutionException} — so a caller's error handling still sees its own
     * exception type; every worker is awaited first, so no task outlives the call.
     *
     * <p>
     * This is the pool {@link #findReferences(List, Collection)} runs on, exposed because a host's per-file work (reading, decoding,
     * building its own rows) wants the same sizing and the same ordering guarantee as the scan it wraps.
     *
     * @param <T>
     *            item type
     * @param <R>
     *            result type
     * @param items
     *            work items; null or empty yields an empty list
     * @param fn
     *            applied to each item, possibly on another thread — it must be safe to call concurrently
     * @return one result per item, in the order the items were given
     */
    public static <T, R> List<R> mapParallel(final @Nullable List<T> items, final Function<? super T, ? extends R> fn) {
        Objects.requireNonNull(fn, "fn");
        final List<T> work = items == null ? List.of() : new ArrayList<>(items);
        final int n = work.size();
        final List<R> results = new ArrayList<>(n);
        if (n < 2) {
            for (final T item : work) {
                results.add(fn.apply(item));
            }
            return results;
        }
        final Object[] out = new Object[n];
        final AtomicInteger cursor = new AtomicInteger();
        final AtomicReference<Throwable> failure = new AtomicReference<>();
        final int workers = Math.min(SCAN_PARALLELISM, n);
        final List<Thread> threads = new ArrayList<>(workers);
        for (int w = 0; w < workers; w++) {
            final Thread thread = new Thread(() -> {
                for (int i = cursor.getAndIncrement(); i < n; i = cursor.getAndIncrement()) {
                    try {
                        out[i] = fn.apply(work.get(i));
                    } catch (final RuntimeException | Error e) {
                        // First one wins; the remaining workers keep draining so the call
                        // never returns while a thread of ours is still running.
                        failure.compareAndSet(null, e);
                    }
                }
            }, "tslp-scan-" + w);
            thread.start();
            threads.add(thread);
        }
        for (final Thread thread : threads) {
            try {
                thread.join();
            } catch (final InterruptedException e) {
                Thread.currentThread().interrupt();
                throw new EditException("batch scan interrupted");
            }
        }
        final Throwable first = failure.get();
        if (first instanceof RuntimeException runtime) {
            throw runtime;
        }
        if (first instanceof Error error) {
            throw error;
        }
        for (int i = 0; i < n; i++) {
            @SuppressWarnings("unchecked")
            final R value = (R) out[i];
            results.add(value);
        }
        return results;
    }

    /**
     * Every occurrence of EACH identifier in {@code names}, in EVERY file of {@code files} — the batch form over FILES, where
     * {@link #findReferences(String, String, Collection)} is the batch form over names.
     *
     * <p>
     * Files are scanned across {@link #SCAN_PARALLELISM} workers and returned in REQUEST ORDER, one row per input file. Every distinct
     * language is resolved ONCE on the calling thread before the workers start, so a dynamically loaded grammar is fetched and registered
     * exactly once instead of being raced for by every worker that needs it.
     *
     * <p>
     * TOTAL per file: an unparsable file, an unknown language or any native error becomes that row's {@link FileReferences#error()} instead
     * of failing the batch, because one unreadable file must not sink a repo-wide trace.
     *
     * @param files
     *            files to scan; null or empty yields an empty list
     * @param names
     *            identifiers to find; blank entries and duplicates are dropped
     * @return one row per input file, in input order
     * @throws EditException
     *             when no non-blank name was given
     */
    public static List<FileReferences> findReferences(final @Nullable List<FileSource> files, final Collection<String> names) {
        final List<String> wanted = new ArrayList<>(new LinkedHashSet<>(
                names == null ? List.<String>of() : names.stream().filter(n -> n != null && !n.isBlank()).map(String::strip).toList()));
        if (wanted.isEmpty()) {
            throw new EditException("findReferences requires at least one non-blank name");
        }
        if (files == null || files.isEmpty()) {
            return List.of();
        }
        for (final String language : new LinkedHashSet<>(files.stream().map(FileSource::language).toList())) {
            try {
                TreeSitterLanguagePackRs.hasLanguage(language);
            } catch (final TreeSitterLanguagePackRsException | RuntimeException e) {
                // A language that cannot be resolved is reported per FILE below, with the
                // scan error that names it — prewarming is an optimisation, not a gate.
                continue;
            }
        }
        return mapParallel(files, file -> {
            try {
                return new FileReferences(file.path(), findReferences(file.source(), file.language(), wanted), null);
            } catch (final TreeSitterLanguagePackRsException | RuntimeException e) {
                final String message = e.getMessage();
                return new FileReferences(file.path(), Map.of(), message == null || message.isBlank() ? e.getClass().getName() : message);
            }
        });
    }
}
