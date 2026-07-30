package dev.kreuzberg.treesitterlanguagepack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import dev.kreuzberg.treesitterlanguagepack.StructuralApi.Op;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Most languages carry a definition's doc as a COMMENT above it, which the native docstring extractor (string-node based) never reports.
 * Doc edits must still see it: otherwise {@code add_doc} stacks a second doc on a documented definition and {@code replace_doc} can never
 * edit one.
 */
final class StructuralApiDocCommentTest {

    private static final String JAVA = "class Alpha {\n" + "  int gamma() {\n" + "    return 1;\n" + "  }\n" + "}\n";

    private static final String GO = "package main\n" + "\n" + "// Alpha does a thing.\n" + "func Alpha() int {\n" + "\treturn 1\n" + "}\n";

    @Test
    @DisplayName("add_doc refuses to stack a second doc comment on a documented definition")
    void addDocRefusesWhenACommentDocExists() throws Exception {
        final String once = StructuralApi.edit(JAVA, "java", Op.ADD_DOC, "gamma", null, "/** First. */");
        assertTrue(once.contains("/** First. */"), "doc not added:\n" + once);
        final StructuralApi.EditException e = assertThrows(StructuralApi.EditException.class,
                () -> StructuralApi.edit(once, "java", Op.ADD_DOC, "gamma", null, "/** Second. */"));
        assertTrue(e.getMessage().contains("replace_doc"), "unhelpful message: " + e.getMessage());
    }

    @Test
    @DisplayName("replace_doc rewrites a leading comment doc in place")
    void replaceDocEditsACommentDoc() throws Exception {
        final String once = StructuralApi.edit(JAVA, "java", Op.ADD_DOC, "gamma", null, "/** First. */");
        final String twice = StructuralApi.edit(once, "java", Op.REPLACE_DOC, "gamma", null, "/** Second. */");
        assertTrue(twice.contains("/** Second. */"), "replacement not applied:\n" + twice);
        assertFalse(twice.contains("First."), "old doc survived:\n" + twice);
        assertEquals(once.lines().count(), twice.lines().count(), "line count changed:\n" + twice);
        assertTrue(twice.contains("int gamma()"), "definition damaged:\n" + twice);
    }

    @Test
    @DisplayName("a pre-existing line comment counts as the doc, in any language")
    void replaceDocEditsAPreExistingLineComment() throws Exception {
        final String out = StructuralApi.edit(GO, "go", Op.REPLACE_DOC, "Alpha", null, "// Alpha returns one.");
        assertTrue(out.contains("// Alpha returns one."), "replacement not applied:\n" + out);
        assertFalse(out.contains("does a thing"), "old doc survived:\n" + out);
        assertTrue(out.contains("func Alpha() int {"), "definition damaged:\n" + out);
    }

    @Test
    @DisplayName("a comment separated by a blank line is not treated as the doc")
    void detachedCommentIsNotADoc() throws Exception {
        final String src = "package main\n\n// unrelated note\n\nfunc Alpha() int {\n\treturn 1\n}\n";
        assertThrows(StructuralApi.EditException.class, () -> StructuralApi.edit(src, "go", Op.REPLACE_DOC, "Alpha", null, "// nope"));
        final String out = StructuralApi.edit(src, "go", Op.ADD_DOC, "Alpha", null, "// Alpha returns one.");
        assertTrue(out.contains("// unrelated note"), "unrelated comment lost:\n" + out);
        assertTrue(out.contains("// Alpha returns one.\nfunc Alpha()"), "doc not hugging the def:\n" + out);
    }
}
