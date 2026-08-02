package dev.kreuzberg.treesitterlanguagepack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.Callable;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.function.Supplier;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * One process, many callers.
 *
 * <p>
 * A host embeds this library once and serves every session from that single process, so
 * {@link StructuralApi#findReferences(String, String, java.util.Collection)} is entered concurrently by unrelated callers. Behind it sit
 * shared things: a process-wide tree cache with a mutex, thread-local tree-sitter parsers, and a Panama downcall whose arena is confined to
 * the calling thread. Each of those is a place where concurrent use could return another caller's answer, or no answer at all.
 *
 * <p>
 * Every test here computes the answer alone first, then demands that exact answer back from many threads at once — over sources chosen to
 * overflow the tree cache, so the threads evict each other's entries while they read.
 */
class StructuralApiConcurrencyTest {

    private static final int THREADS = 16;
    private static final int ROUNDS = 25;
    private static final List<String> NAMES = List.of("alpha", "beta", "gamma");

    /** Distinct sources: enough of them that the threads fall out of any shared cache. */
    private static List<String> sources() {
        final List<String> sources = new ArrayList<>();
        for (int i = 0; i < 24; i++) {
            sources.add("(defn alpha" + i + " [beta]\n  (alpha (beta " + i + ")))\n");
        }
        return sources;
    }

    /** The single-threaded answer for every source — what concurrency must not change. */
    private static List<Map<String, List<StructuralApi.ReferenceHit>>> baseline(final List<String> sources)
            throws TreeSitterLanguagePackRsException {
        final List<Map<String, List<StructuralApi.ReferenceHit>>> baseline = new ArrayList<>();
        for (final String source : sources) {
            baseline.add(StructuralApi.findReferences(source, "clojure", NAMES));
        }
        return baseline;
    }

    /**
     * Run the same staggered walk over {@code sources} on THREADS threads of the given executor, failing if any thread ever disagrees with
     * the single-threaded baseline.
     */
    private static void hammer(final Supplier<ExecutorService> pool) throws Exception {
        final List<String> sources = sources();
        final List<Map<String, List<StructuralApi.ReferenceHit>>> baseline = baseline(sources);
        final List<Callable<Integer>> work = new ArrayList<>();
        for (int t = 0; t < THREADS; t++) {
            final int thread = t;
            work.add(() -> {
                int checked = 0;
                for (int round = 0; round < ROUNDS; round++) {
                    final int which = (thread + round) % sources.size();
                    assertEquals(baseline.get(which), StructuralApi.findReferences(sources.get(which), "clojure", NAMES),
                            "thread " + thread + ", round " + round);
                    checked++;
                }
                return checked;
            });
        }

        try (ExecutorService executor = pool.get()) {
            final List<Future<Integer>> futures = executor.invokeAll(work, 2, TimeUnit.MINUTES);
            int checked = 0;
            for (final Future<Integer> future : futures) {
                // get() rethrows whatever the thread hit — a wrong answer, a native crash, a timeout.
                checked += future.get();
            }
            assertEquals(THREADS * ROUNDS, checked);
        }
    }

    @Test
    @DisplayName("concurrent platform threads each get the single-threaded result")
    void concurrentPlatformThreadsAgreeWithTheBaseline() throws Exception {
        hammer(() -> Executors.newFixedThreadPool(THREADS));
    }

    @Test
    @DisplayName("concurrent virtual threads each get the single-threaded result")
    void concurrentVirtualThreadsAgreeWithTheBaseline() throws Exception {
        // Virtual threads are the interesting case: the confined Arena and the native
        // thread-locals behind the downcall are per-carrier, not per-task.
        hammer(Executors::newVirtualThreadPerTaskExecutor);
    }

    @Test
    @DisplayName("a batch scan is unaffected by other threads scanning the same source")
    void sameSourceFromEveryThreadStaysStable() throws Exception {
        final String source = "(defn alpha [beta]\n  (alpha beta)\n  (gamma alpha beta))\n";
        final Map<String, List<StructuralApi.ReferenceHit>> expected = StructuralApi.findReferences(source, "clojure", NAMES);
        assertTrue(expected.get("alpha").size() >= 3, "the fixture should have several hits");

        final List<Callable<Boolean>> work = new ArrayList<>();
        for (int t = 0; t < THREADS; t++) {
            work.add(() -> {
                for (int round = 0; round < ROUNDS; round++) {
                    assertEquals(expected, StructuralApi.findReferences(source, "clojure", NAMES));
                }
                return Boolean.TRUE;
            });
        }
        try (ExecutorService executor = Executors.newFixedThreadPool(THREADS)) {
            for (final Future<Boolean> future : executor.invokeAll(work, 2, TimeUnit.MINUTES)) {
                assertTrue(future.get());
            }
        }
    }
}
