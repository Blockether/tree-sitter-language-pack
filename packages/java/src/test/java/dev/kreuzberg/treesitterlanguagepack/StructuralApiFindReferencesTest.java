package dev.kreuzberg.treesitterlanguagepack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import dev.kreuzberg.treesitterlanguagepack.StructuralApi.ReferenceHit;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * The batch {@link StructuralApi#findReferences(String, String, java.util.Collection)} must be a pure speed-up: for every name it has to
 * return exactly what the single-name form returns, from ONE parse instead of one per name.
 */
final class StructuralApiFindReferencesTest {

    private static final String CLJ = "(ns demo.core)\n" + "\n" + "(defn alpha [x] x)\n" + "\n" + "(defn beta [y]\n"
            + "  (alpha (alpha y)))\n";

    @Test
    @DisplayName("batch findReferences returns exactly the single-name hits, per name")
    void batchMatchesSingleName() throws Exception {
        final List<String> names = List.of("alpha", "beta", "x", "y", "nowhere");
        final Map<String, List<ReferenceHit>> batch = StructuralApi.findReferences(CLJ, "clojure", names);
        assertEquals(names, List.copyOf(batch.keySet()), "one entry per name, in input order");
        for (final String name : names) {
            assertEquals(StructuralApi.findReferences(CLJ, "clojure", name), batch.get(name), name);
        }
    }

    @Test
    @DisplayName("hits carry 1-based line and 0-based column in source order")
    void hitsCarryLineAndColumn() throws Exception {
        final List<ReferenceHit> alpha = StructuralApi.findReferences(CLJ, "clojure", List.of("alpha")).get("alpha");
        assertEquals(3, alpha.size(), "definition plus two uses: " + alpha);
        assertEquals(3, alpha.get(0).line(), "definition line");
        assertEquals(6, alpha.get(0).column(), "definition column");
        assertEquals(6, alpha.get(1).line(), "first use line");
        assertEquals(3, alpha.get(1).column(), "first use column");
        assertEquals(6, alpha.get(2).line(), "second use line");
        assertEquals(10, alpha.get(2).column(), "second use column");
        assertEquals("alpha", CLJ.substring(alpha.get(2).startByte(), alpha.get(2).endByte()));
    }

    @Test
    @DisplayName("a name that never occurs maps to an empty list, not a missing key")
    void absentNameMapsToEmptyList() throws Exception {
        final Map<String, List<ReferenceHit>> hits = StructuralApi.findReferences(CLJ, "clojure", List.of("nowhere"));
        assertTrue(hits.containsKey("nowhere"), "key must be present");
        assertTrue(hits.get("nowhere").isEmpty(), "and its hit list empty");
    }

    @Test
    @DisplayName("blank entries and duplicates are dropped, names are stripped")
    void blanksAndDuplicatesAreDropped() throws Exception {
        final Map<String, List<ReferenceHit>> hits = StructuralApi.findReferences(CLJ, "clojure",
                java.util.Arrays.asList("alpha", " alpha ", "  ", null));
        assertEquals(List.of("alpha"), List.copyOf(hits.keySet()));
        assertEquals(3, hits.get("alpha").size());
    }

    @Test
    @DisplayName("a batch with nothing to look for is refused")
    void emptyBatchIsRefused() {
        assertThrows(StructuralApi.EditException.class, () -> StructuralApi.findReferences(CLJ, "clojure", List.of()));
        assertThrows(StructuralApi.EditException.class, () -> StructuralApi.findReferences(CLJ, "clojure", List.of("   ")));
    }

    @Test
    @DisplayName("line and column stay correct deep into a large file")
    void lineIndexScalesToLateHits() throws Exception {
        final StringBuilder big = new StringBuilder("(ns demo.big)\n");
        for (int i = 0; i < 2000; i++) {
            big.append("(defn f").append(i).append(" [] :filler)\n");
        }
        big.append("(defn tail [] (f0))\n");
        final String source = big.toString();
        final List<ReferenceHit> tail = StructuralApi.findReferences(source, "clojure", List.of("tail")).get("tail");
        assertEquals(1, tail.size());
        assertEquals(2002, tail.get(0).line(), "1 ns line + 2000 defs, then the tail");
        assertEquals(6, tail.get(0).column());
        assertEquals("tail", source.substring(tail.get(0).startByte(), tail.get(0).endByte()));
    }
}
