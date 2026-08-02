package dev.kreuzberg.treesitterlanguagepack;

/**
 * One occurrence of a traced identifier: the name that matched plus its source span.
 *
 * <p>
 * Produced by the Rust batch scan behind {@link StructuralApi#findReferences(String, String, java.util.Collection)}: a leaf token whose
 * text equals the requested identifier, so matches always sit on real token boundaries and never inside a larger word, string, or comment.
 *
 * @param name
 *            the identifier that matched
 * @param span
 *            byte range and 0-based line / column of the occurrence
 */
public record ReferenceHit(String name, Span span) {
}
