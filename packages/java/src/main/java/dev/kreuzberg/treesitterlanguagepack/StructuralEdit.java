package dev.kreuzberg.treesitterlanguagepack;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import org.jspecify.annotations.Nullable;

/**
 * Unified structural editing over tree-sitter, for every language the pack
 * understands (Clojure included). Target a definition by NAME and replace or
 * insert around it; the engine locates the node from the structural outline,
 * splices its line span, re-parses the result, and REFUSES the edit if it
 * introduces a syntax error — so a write never corrupts a file.
 *
 * <p>This is the language-neutral, JVM-native replacement for per-language
 * structural editors (e.g. a Clojure-only rewrite-clj path): the same
 * locate-by-name editing for any supported language, with parse validation.
 */
public final class StructuralEdit {

  private StructuralEdit() {}

  /** What to do at the located definition. */
  public enum Op {
    /** Replace the whole definition. */
    REPLACE,
    /** Insert {@code code} immediately before the definition. */
    INSERT_BEFORE,
    /** Insert {@code code} immediately after the definition. */
    INSERT_AFTER,
    /** Append {@code code} at end of file (ignores the target). */
    APPEND
  }

  /** A located definition: name, kind, and 1-based inclusive line span. */
  public record Target(@Nullable String name, String kind, int startLine, int endLine) {}

  /** Raised when an edit cannot be applied (missing/ambiguous target, bad result). */
  public static final class EditException extends RuntimeException {
    private static final long serialVersionUID = 1L;

    /** @param message human-readable, actionable reason. */
    public EditException(final String message) {
      super(message);
    }
  }

  /**
   * Depth-first structural outline: every definition as a {@link Target} with
   * 1-based inclusive line numbers.
   *
   * @param source   file contents
   * @param language tree-sitter language name (e.g. {@code "clojure"})
   * @return the outline (empty when the language has no structure extraction)
   * @throws TreeSitterLanguagePackRsException on a native processing error
   */
  public static List<Target> outline(final String source, final String language)
      throws TreeSitterLanguagePackRsException {
    final ProcessConfig cfg = ProcessConfig.builder().withLanguage(language).withStructure(true).build();
    final ProcessResult res = TreeSitterLanguagePack.process(source, cfg);
    final List<Target> out = new ArrayList<>();
    flatten(res.structure(), out);
    return out;
  }

  private static void flatten(final @Nullable List<StructureItem> items, final List<Target> out) {
    if (items == null) {
      return;
    }
    for (final StructureItem it : items) {
      final Span s = it.span();
      final String kind = it.kind() == null ? "" : it.kind().getValue();
      out.add(new Target(it.name(), kind, (int) s.startLine() + 1, (int) s.endLine() + 1));
      flatten(it.children(), out);
    }
  }

  /**
   * Structural edit by definition name. {@link Op#APPEND} ignores {@code target};
   * the others locate the definition by {@code target} (and optional {@code kind}
   * to disambiguate same-named definitions). The returned content is guaranteed
   * to parse.
   *
   * @param source   current file contents
   * @param language tree-sitter language name
   * @param op       what to do at the located node
   * @param target   definition name (ignored for {@link Op#APPEND})
   * @param kind     optional kind filter (e.g. {@code "function"}, {@code "method"})
   * @param code     replacement / inserted source
   * @return the new file contents
   * @throws EditException                     target missing/ambiguous, or result has a syntax error
   * @throws TreeSitterLanguagePackRsException  on a native processing error
   */
  public static String edit(final String source, final String language, final Op op,
      final @Nullable String target, final @Nullable String kind, final String code)
      throws TreeSitterLanguagePackRsException {
    if (code == null) {
      throw new EditException("structural edit requires non-null code");
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
      throw new EditException("Edit rejected: it introduces " + errors.size()
          + " syntax error(s); the file was not changed.");
    }
    return result;
  }

  private static Target locate(final List<Target> items, final @Nullable String target,
      final @Nullable String kind) {
    final List<Target> matches = new ArrayList<>();
    for (final Target t : items) {
      if (Objects.equals(t.name(), target) && (kind == null || kind.equalsIgnoreCase(t.kind()))) {
        matches.add(t);
      }
    }
    if (matches.isEmpty()) {
      throw new EditException("No definition named '" + target + "'"
          + (kind == null ? "" : " of kind " + kind) + " found. Use index(path) to list definitions.");
    }
    if (matches.size() > 1) {
      final StringBuilder kinds = new StringBuilder();
      for (final Target t : matches) {
        if (kinds.length() > 0) {
          kinds.append(", ");
        }
        kinds.append(t.kind());
      }
      throw new EditException(matches.size() + " definitions named '" + target
          + "' — pass kind to disambiguate (" + kinds + ").");
    }
    return matches.get(0);
  }

  private static String splice(final List<String> lines, final Op op, final int start,
      final int end, final String code) {
    final List<String> out = new ArrayList<>(lines.size() + 1);
    switch (op) {
      case APPEND -> {
        out.addAll(lines);
        out.add(code);
      }
      case REPLACE -> {
        out.addAll(lines.subList(0, start - 1));
        out.add(code);
        out.addAll(lines.subList(end, lines.size()));
      }
      case INSERT_BEFORE -> {
        out.addAll(lines.subList(0, start - 1));
        out.add(code);
        out.addAll(lines.subList(start - 1, lines.size()));
      }
      case INSERT_AFTER -> {
        out.addAll(lines.subList(0, end));
        out.add(code);
        out.addAll(lines.subList(end, lines.size()));
      }
      default -> throw new EditException("Unknown op: " + op);
    }
    return String.join("\n", out);
  }

  private static List<Diagnostic> errorDiagnostics(final String source, final String language)
      throws TreeSitterLanguagePackRsException {
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
}
