package dev.kreuzberg.treesitterlanguagepack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * The batch-of-FILES scan: {@link StructuralApi#findReferences(List, java.util.Collection)} and the worker pool it runs on,
 * {@link StructuralApi#mapParallel(List, java.util.function.Function)}.
 *
 * <p>
 * A repo-wide trace is the whole point of these two, so what is pinned here is what such a caller depends on: rows come back in REQUEST
 * order, every file gets exactly one row, one bad file is an error row rather than a thrown batch, and running many files at once never
 * changes an answer (the parallel result must equal the file-by-file one).
 */
class StructuralApiBatchScanTest {

    private static final String CLOJURE = "clojure";
    private static final List<String> NAMES = List.of("add", "mul", "absent");

    private static StructuralApi.FileSource file(final int i) {
        return new StructuralApi.FileSource("f" + i + ".clj", CLOJURE, "(defn add" + i + " [a b] (add (mul a b)))\n");
    }

    private static List<StructuralApi.FileSource> files(final int n) {
        final List<StructuralApi.FileSource> out = new ArrayList<>(n);
        for (int i = 0; i < n; i++) {
            out.add(file(i));
        }
        return out;
    }

    @Test
    @DisplayName("one row per file, in REQUEST order, matching the file-by-file answer")
    void batchMatchesSerial() throws Exception {
        final List<StructuralApi.FileSource> files = files(40);
        final List<StructuralApi.FileReferences> rows = StructuralApi.findReferences(files, NAMES);

        assertEquals(files.size(), rows.size());
        for (int i = 0; i < files.size(); i++) {
            final StructuralApi.FileSource src = files.get(i);
            final StructuralApi.FileReferences row = rows.get(i);
            assertEquals(src.path(), row.path(), "row " + i + " is out of request order");
            assertNull(row.error());
            assertFalse(row.isFailed());
            // Byte-for-byte the single-file answer: the fan-out must not change a scan.
            assertEquals(StructuralApi.findReferences(src.source(), CLOJURE, NAMES), row.references());
            // `add0` is its OWN token, so only the call to `add` counts as a reference.
            assertEquals(1, row.references().get("add").size());
            assertEquals(1, row.references().get("mul").size());
            assertTrue(row.references().get("absent").isEmpty());
        }
    }

    @Test
    @DisplayName("TOTAL: an unknown language is ONE error row, not a failed batch")
    void oneBadFileIsOneErrorRow() {
        final List<StructuralApi.FileReferences> rows = StructuralApi
                .findReferences(List.of(new StructuralApi.FileSource("ok.clj", CLOJURE, "(defn add [a b] (+ a b))\n"),
                        new StructuralApi.FileSource("weird.zzz", "no-such-language", "add add\n"),
                        new StructuralApi.FileSource("also-ok.clj", CLOJURE, "(add 1 2)\n")), List.of("add"));

        assertEquals(List.of("ok.clj", "weird.zzz", "also-ok.clj"), rows.stream().map(StructuralApi.FileReferences::path).toList());
        assertNull(rows.get(0).error());
        assertEquals(1, rows.get(0).references().get("add").size());

        assertTrue(rows.get(1).isFailed());
        assertNotNull(rows.get(1).error());
        assertFalse(rows.get(1).error().isBlank());
        assertTrue(rows.get(1).references().isEmpty());

        assertNull(rows.get(2).error(), "a later file still scans after an earlier one failed");
        assertEquals(1, rows.get(2).references().get("add").size());
    }

    @Test
    @DisplayName("no files is an empty batch; no name is a refusal")
    void degenerateInputs() {
        assertEquals(List.of(), StructuralApi.findReferences(List.<StructuralApi.FileSource>of(), NAMES));
        assertEquals(List.of(), StructuralApi.findReferences(null, NAMES));
        assertThrows(StructuralApi.EditException.class, () -> StructuralApi.findReferences(files(2), List.of()));
        assertThrows(StructuralApi.EditException.class, () -> StructuralApi.findReferences(files(2), List.of("  ")));
    }

    @Test
    @DisplayName("duplicate and blank names collapse to one bucket each")
    void namesAreNormalized() {
        final Map<String, List<StructuralApi.ReferenceHit>> refs = StructuralApi
                .findReferences(List.of(new StructuralApi.FileSource("a.clj", CLOJURE, "(defn add [a b] (add a b))\n")),
                        java.util.Arrays.asList("add", " add ", "", null))
                .get(0).references();

        assertEquals(List.of("add"), List.copyOf(refs.keySet()));
        assertEquals(2, refs.get("add").size());
    }

    @Test
    @DisplayName("mapParallel keeps REQUEST order and applies the function exactly once per item")
    void mapParallelOrder() {
        final List<Integer> items = new ArrayList<>();
        for (int i = 0; i < 500; i++) {
            items.add(i);
        }
        final AtomicInteger calls = new AtomicInteger();
        final List<Integer> doubled = StructuralApi.mapParallel(items, i -> {
            calls.incrementAndGet();
            return i * 2;
        });

        assertEquals(items.stream().map(i -> i * 2).toList(), doubled);
        assertEquals(items.size(), calls.get());
        assertEquals(List.of(), StructuralApi.mapParallel(List.<Integer>of(), i -> i));
        assertEquals(List.of(7), StructuralApi.mapParallel(List.of(7), i -> i));
        assertEquals(List.of(), StructuralApi.mapParallel(null, i -> i));
    }

    @Test
    @DisplayName("mapParallel rethrows the worker's OWN exception, never an ExecutionException")
    void mapParallelRethrowsAsThrown() {
        final IllegalStateException boom = new IllegalStateException("boom");
        final List<Integer> items = new ArrayList<>();
        for (int i = 0; i < 64; i++) {
            items.add(i);
        }

        final IllegalStateException thrown = assertThrows(IllegalStateException.class, () -> StructuralApi.mapParallel(items, i -> {
            if (i == 7) {
                throw boom;
            }
            return i;
        }));
        assertSame(boom, thrown);
    }
}
