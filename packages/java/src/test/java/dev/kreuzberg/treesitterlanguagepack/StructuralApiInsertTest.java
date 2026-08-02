package dev.kreuzberg.treesitterlanguagepack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import dev.kreuzberg.treesitterlanguagepack.StructuralApi.Op;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Blank-line separation for {@link StructuralApi.Op#INSERT_BEFORE}, {@link StructuralApi.Op#INSERT_AFTER} and
 * {@link StructuralApi.Op#APPEND}: an inserted top-level definition must never be glued onto its neighbour — there has to be exactly one
 * blank line between sibling forms (idempotent when a blank line already sits on that side).
 */
final class StructuralApiInsertTest {

    private static final String CLJ = "(ns foo.bar)\n\n(defn alpha\n  [x]\n  (inc x))\n\n(defn beta\n  [y]\n  (dec y))\n";

    @Test
    @DisplayName("insert_after leaves a blank line between the target and the new node")
    void insertAfterSeparatesWithBlankLine() throws Exception {
        final String out = StructuralApi.edit(CLJ, "clojure", Op.INSERT_AFTER, "alpha", null, "(defn gamma\n  [z]\n  (* z z))");
        // one blank line above the inserted node, one below (the pre-existing one).
        assertTrue(out.contains("(inc x))\n\n(defn gamma"), "blank line above gamma:\n" + out);
        assertTrue(out.contains("(* z z))\n\n(defn beta"), "blank line below gamma:\n" + out);
        assertFalse(out.contains("(inc x))\n(defn gamma"), "glued to alpha:\n" + out);
    }

    @Test
    @DisplayName("insert_before leaves a blank line between the new node and the target")
    void insertBeforeSeparatesWithBlankLine() throws Exception {
        final String out = StructuralApi.edit(CLJ, "clojure", Op.INSERT_BEFORE, "beta", null, "(defn gamma\n  [z]\n  (* z z))");
        assertTrue(out.contains("(inc x))\n\n(defn gamma"), "blank line above gamma:\n" + out);
        assertTrue(out.contains("(* z z))\n\n(defn beta"), "blank line below gamma:\n" + out);
        assertFalse(out.contains("(* z z))\n(defn beta"), "glued to beta:\n" + out);
    }

    @Test
    @DisplayName("insert between adjacent forms adds exactly one separator on each side")
    void insertBetweenAdjacentForms() throws Exception {
        final String adjacent = "(def a 1)\n(def b 2)\n";
        final String out = StructuralApi.edit(adjacent, "clojure", Op.INSERT_AFTER, "a", null, "(def c 3)");
        assertEquals("(def a 1)\n\n(def c 3)\n\n(def b 2)\n", out);
    }

    @Test
    @DisplayName("append separates the new node with a blank line and keeps a trailing newline")
    void appendSeparatesWithBlankLine() throws Exception {
        final String out = StructuralApi.edit(CLJ, "clojure", Op.APPEND, null, null, "(def end 3)");
        assertTrue(out.contains("(dec y))\n\n(def end 3)"), "blank line before appended node:\n" + out);
        assertTrue(out.endsWith("(def end 3)\n"), "trailing newline kept:\n" + out);
        assertFalse(out.contains("(dec y))\n(def end 3)"), "glued to beta:\n" + out);
    }

    @Test
    @DisplayName("insert_after collapses a pre-existing double blank below to a single one")
    void insertAfterNormalisesDoubleBlankBelow() throws Exception {
        // alpha is already followed by TWO blank lines before beta.
        final String doubled = "(defn alpha\n  [x]\n  (inc x))\n\n\n(defn beta\n  [y]\n  (dec y))\n";
        final String out = StructuralApi.edit(doubled, "clojure", Op.INSERT_AFTER, "alpha", null, "(defn gamma\n  [z]\n  (* z z))");
        assertTrue(out.contains("(inc x))\n\n(defn gamma"), "one blank above gamma:\n" + out);
        assertTrue(out.contains("(* z z))\n\n(defn beta"), "one blank below gamma:\n" + out);
        assertFalse(out.contains("\n\n\n"), "no triple newline (double blank) remains:\n" + out);
    }

    @Test
    @DisplayName("insert_before collapses a pre-existing double blank above to a single one")
    void insertBeforeNormalisesDoubleBlankAbove() throws Exception {
        // beta is already preceded by TWO blank lines.
        final String doubled = "(defn alpha\n  [x]\n  (inc x))\n\n\n(defn beta\n  [y]\n  (dec y))\n";
        final String out = StructuralApi.edit(doubled, "clojure", Op.INSERT_BEFORE, "beta", null, "(defn gamma\n  [z]\n  (* z z))");
        assertTrue(out.contains("(inc x))\n\n(defn gamma"), "one blank above gamma:\n" + out);
        assertTrue(out.contains("(* z z))\n\n(defn beta"), "one blank below gamma:\n" + out);
        assertFalse(out.contains("\n\n\n"), "no triple newline (double blank) remains:\n" + out);
    }

    @Test
    @DisplayName("inserted code with its own leading/trailing blank lines is stripped to one separator")
    void insertStripsCodeEdgeBlankLines() throws Exception {
        final String out = StructuralApi.edit(CLJ, "clojure", Op.INSERT_AFTER, "alpha", null, "\n\n(defn gamma\n  [z]\n  (* z z))\n\n");
        assertTrue(out.contains("(inc x))\n\n(defn gamma"), "one blank above gamma:\n" + out);
        assertTrue(out.contains("(* z z))\n\n(defn beta"), "one blank below gamma:\n" + out);
        assertFalse(out.contains("\n\n\n"), "no triple newline (double blank) remains:\n" + out);
    }

    @Test
    @DisplayName("append collapses a pre-existing double blank at end of file")
    void appendNormalisesTrailingDoubleBlank() throws Exception {
        final String doubled = "(ns foo.bar)\n\n(defn alpha\n  [x]\n  (inc x))\n\n\n";
        final String out = StructuralApi.edit(doubled, "clojure", Op.APPEND, null, null, "(def end 3)");
        assertTrue(out.contains("(inc x))\n\n(def end 3)"), "one blank before appended node:\n" + out);
        assertTrue(out.endsWith("(def end 3)\n"), "trailing newline kept:\n" + out);
        assertFalse(out.contains("\n\n\n"), "no triple newline remains:\n" + out);
    }

    @Test
    @DisplayName("a non-append edit with no target reports the missing locator, not 'null'")
    void missingTargetReportsActionableError() {
        final StructuralApi.EditException ex = assertThrows(StructuralApi.EditException.class,
                () -> StructuralApi.edit(CLJ, "clojure", Op.INSERT_AFTER, null, null, "(defn gamma\n  [z]\n  (* z z))"));
        assertFalse(ex.getMessage().contains("'null'"), "must not name a 'null' definition:\n" + ex.getMessage());
        assertTrue(ex.getMessage().contains("target"), "names the missing locator:\n" + ex.getMessage());
    }
}
