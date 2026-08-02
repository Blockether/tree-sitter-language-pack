package dev.kreuzberg.treesitterlanguagepack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * {@link StructuralApi#rename(String, String, String, String)} rewrites CODE occurrences only, and must stay byte-exact on multi-byte text
 * however many occurrences it rewrites.
 */
final class StructuralApiRenameTest {

    @Test
    @DisplayName("rename skips strings and comments but follows interpolated identifiers")
    void renameRewritesCodeOnly() throws Exception {
        final String source = "def alpha(x):\n" + "    # alpha stays put in this comment\n" + "    return \"alpha\" + f\"{alpha(x)}\"\n";
        final String renamed = StructuralApi.rename(source, "python", "alpha", "beta");
        assertEquals("def beta(x):\n" + "    # alpha stays put in this comment\n" + "    return \"alpha\" + f\"{beta(x)}\"\n", renamed);
    }

    @Test
    @DisplayName("rename stays byte-exact over many occurrences and multi-byte text")
    void renameIsByteExactOnMultiByteSource() throws Exception {
        final int lines = 200;
        final StringBuilder source = new StringBuilder("# héllo wörld — ünicode ✅\n");
        for (int i = 0; i < lines; i++) {
            source.append("total = alpha + alpha\n");
        }
        final String renamed = StructuralApi.rename(source.toString(), "python", "alpha", "a_considerably_longer_name");

        assertTrue(renamed.startsWith("# héllo wörld — ünicode ✅\n"), "multi-byte prefix survives");
        assertFalse(renamed.contains("alpha"), "no occurrence left behind");
        assertEquals(2 * lines, renamed.split("a_considerably_longer_name", -1).length - 1, "every occurrence rewritten");
        assertEquals(source.toString(), StructuralApi.rename(renamed, "python", "a_considerably_longer_name", "alpha"),
                "renaming back reproduces the original byte for byte");
    }
}
