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
    @Test
    @DisplayName("a doc comment is found even when modifiers precede the definition on its line")
    void docCommentIsFoundBehindModifiers() throws Exception {
        final String ts = "// Alpha adds.\nexport function alpha(x: number): number {\n  return x + 1;\n}\n";
        final String out = StructuralApi.edit(ts, "typescript", Op.REPLACE_DOC, "alpha", null, "// Alpha increments.");
        assertTrue(out.contains("// Alpha increments."), "replacement not applied:\n" + out);
        assertFalse(out.contains("Alpha adds."), "old doc survived:\n" + out);
        assertTrue(out.contains("export function alpha"), "definition damaged:\n" + out);
        assertThrows(StructuralApi.EditException.class,
                () -> StructuralApi.edit(ts, "typescript", Op.ADD_DOC, "alpha", null, "// stacked"));
    }

    @Test
    @DisplayName("replacing a doc comment keeps the newline that terminates it")
    void replaceDocKeepsTheCommentTerminator() throws Exception {
        final String rs = "/// Alpha.\nfn alpha() -> i32 { 1 }\n";
        final String out = StructuralApi.edit(rs, "rust", Op.REPLACE_DOC, "alpha", null, "/// Alpha returns one.");
        assertEquals("/// Alpha returns one.\nfn alpha() -> i32 { 1 }\n", out);
    }
    @Test
    @DisplayName("a doc comment inside an unparsed <script> body still counts as the doc")
    void docCommentInsideARawTextRegion() throws Exception {
        final String svelte = "<script>\n  // Alpha holds one.\n  let alpha = 1;\n</script>\n\n<div>{alpha}</div>\n";
        final String out = StructuralApi.edit(svelte, "svelte", Op.REPLACE_DOC, "alpha", null, "// Alpha holds a number.");
        assertTrue(out.contains("// Alpha holds a number."), "replacement not applied:\n" + out);
        assertFalse(out.contains("holds one"), "old doc survived:\n" + out);
        assertTrue(out.contains("let alpha = 1;"), "definition damaged:\n" + out);
        assertEquals(svelte.lines().count(), out.lines().count(), "line count changed:\n" + out);
        assertThrows(StructuralApi.EditException.class,
                () -> StructuralApi.edit(svelte, "svelte", Op.ADD_DOC, "alpha", null, "// stacked"));
    }

    @Test
    @DisplayName("code that merely starts like a comment is not mistaken for one")
    void privateFieldIsNotADocComment() throws Exception {
        final String svelte = "<script>\n  let beta = 2;\n  let alpha = 1;\n</script>\n\n<div>{alpha}</div>\n";
        assertThrows(StructuralApi.EditException.class,
                () -> StructuralApi.edit(svelte, "svelte", Op.REPLACE_DOC, "alpha", null, "// nope"));
        final String out = StructuralApi.edit(svelte, "svelte", Op.ADD_DOC, "alpha", null, "// Alpha holds one.");
        assertTrue(out.contains("// Alpha holds one.\n  let alpha = 1;"), "doc not hugging the def:\n" + out);
        assertTrue(out.contains("let beta = 2;"), "neighbour damaged:\n" + out);
    }
}
