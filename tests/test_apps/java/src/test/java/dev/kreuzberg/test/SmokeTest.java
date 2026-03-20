package dev.kreuzberg.test;

import com.google.gson.Gson;
import com.google.gson.reflect.TypeToken;
import io.github.treesitter.languagepack.TsPackRegistry;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestFactory;

import java.io.IOException;
import java.lang.reflect.Type;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.stream.Stream;

import static org.junit.jupiter.api.Assertions.*;

class SmokeTest {

    private static final Gson GSON = new Gson();
    private static final Type MAP_TYPE = new TypeToken<Map<String, Object>>() {}.getType();
    private static final Path FIXTURES_DIR = Path.of("..", "fixtures");
    private static TsPackRegistry registry;

    @BeforeAll
    static void setup() {
        registry = new TsPackRegistry();
        TsPackRegistry.download(List.of("python", "javascript", "rust", "go", "ruby", "java", "c", "cpp"));
    }

    @AfterAll
    static void teardown() {
        if (registry != null) {
            registry.close();
        }
    }

    @SuppressWarnings("unchecked")
    private static List<Map<String, Object>> loadFixtures(String name) throws IOException {
        String json = Files.readString(FIXTURES_DIR.resolve(name));
        Type type = new TypeToken<List<Map<String, Object>>>() {}.getType();
        return GSON.fromJson(json, type);
    }

    private Map<String, Object> processAndParse(String source, String configJson) {
        String resultJson = registry.process(source, configJson);
        return GSON.fromJson(resultJson, MAP_TYPE);
    }

    @TestFactory
    Stream<DynamicTest> basicFixtures() throws IOException {
        List<Map<String, Object>> fixtures = loadFixtures("basic.json");

        return fixtures.stream().map(fixture -> DynamicTest.dynamicTest(
            (String) fixture.get("name"),
            () -> {
                String test = (String) fixture.get("test");
                switch (test) {
                    case "language_count" -> {
                        int count = registry.languageCount();
                        int expectedMin = ((Number) fixture.get("expected_min")).intValue();
                        assertTrue(count >= expectedMin,
                            "language_count " + count + " < expected min " + expectedMin);
                    }
                    case "has_language" -> {
                        String language = (String) fixture.get("language");
                        boolean result = registry.hasLanguage(language);
                        boolean expected = (Boolean) fixture.get("expected");
                        assertEquals(expected, result,
                            "has_language(" + language + ") = " + result + ", expected " + expected);
                    }
                    case "available_languages" -> {
                        List<String> langs = registry.availableLanguages();
                        @SuppressWarnings("unchecked")
                        List<String> expectedContains = (List<String>) fixture.get("expected_contains");
                        for (String lang : expectedContains) {
                            assertTrue(langs.contains(lang),
                                "available_languages missing '" + lang + "'");
                        }
                    }
                    default -> fail("Unknown test type: " + test);
                }
            }
        ));
    }

    @TestFactory
    Stream<DynamicTest> processFixtures() throws IOException {
        List<Map<String, Object>> fixtures = loadFixtures("process.json");

        return fixtures.stream().map(fixture -> DynamicTest.dynamicTest(
            (String) fixture.get("name"),
            () -> {
                String source = (String) fixture.get("source");
                @SuppressWarnings("unchecked")
                Map<String, Object> configMap = (Map<String, Object>) fixture.get("config");
                @SuppressWarnings("unchecked")
                Map<String, Object> expected = (Map<String, Object>) fixture.get("expected");

                String configJson = GSON.toJson(configMap);
                Map<String, Object> result = processAndParse(source, configJson);

                if (expected.containsKey("language")) {
                    assertEquals(expected.get("language"), result.get("language"));
                }
                if (expected.containsKey("structure_min")) {
                    @SuppressWarnings("unchecked")
                    List<Object> structure = (List<Object>) result.get("structure");
                    int min = ((Number) expected.get("structure_min")).intValue();
                    assertTrue(structure.size() >= min,
                        "structure count " + structure.size() + " < min " + min);
                }
                if (expected.containsKey("imports_min")) {
                    @SuppressWarnings("unchecked")
                    List<Object> imports = (List<Object>) result.get("imports");
                    int min = ((Number) expected.get("imports_min")).intValue();
                    assertTrue(imports.size() >= min,
                        "imports count " + imports.size() + " < min " + min);
                }
                if (expected.containsKey("error_count")) {
                    @SuppressWarnings("unchecked")
                    Map<String, Object> metrics = (Map<String, Object>) result.get("metrics");
                    int errorCount = ((Number) metrics.get("error_count")).intValue();
                    int expectedCount = ((Number) expected.get("error_count")).intValue();
                    assertEquals(expectedCount, errorCount);
                }
                if (expected.containsKey("metrics_total_lines_min")) {
                    @SuppressWarnings("unchecked")
                    Map<String, Object> metrics = (Map<String, Object>) result.get("metrics");
                    int totalLines = ((Number) metrics.get("total_lines")).intValue();
                    int min = ((Number) expected.get("metrics_total_lines_min")).intValue();
                    assertTrue(totalLines >= min,
                        "total_lines " + totalLines + " < min " + min);
                }
            }
        ));
    }

    @TestFactory
    Stream<DynamicTest> chunkingFixtures() throws IOException {
        List<Map<String, Object>> fixtures = loadFixtures("chunking.json");

        return fixtures.stream().map(fixture -> DynamicTest.dynamicTest(
            (String) fixture.get("name"),
            () -> {
                String source = (String) fixture.get("source");
                @SuppressWarnings("unchecked")
                Map<String, Object> configMap = (Map<String, Object>) fixture.get("config");
                @SuppressWarnings("unchecked")
                Map<String, Object> expected = (Map<String, Object>) fixture.get("expected");

                String configJson = GSON.toJson(configMap);
                Map<String, Object> result = processAndParse(source, configJson);

                if (expected.containsKey("chunks_min")) {
                    @SuppressWarnings("unchecked")
                    List<Object> chunks = (List<Object>) result.get("chunks");
                    int min = ((Number) expected.get("chunks_min")).intValue();
                    assertTrue(chunks.size() >= min,
                        "chunks count " + chunks.size() + " < min " + min);
                }
            }
        ));
    }

    @Test
    void downloadedLanguagesReturnsArray() {
        List<String> langs = TsPackRegistry.downloadedLanguages();
        assertNotNull(langs);
    }

    @Test
    void manifestLanguagesReturnsArrayWith50Plus() {
        List<String> langs = TsPackRegistry.manifestLanguages();
        assertNotNull(langs);
        assertTrue(langs.size() > 50, "manifestLanguages should return 50+ languages");
    }

    @Test
    void cacheDirReturnsNonEmptyString() {
        String dir = TsPackRegistry.cacheDir();
        assertNotNull(dir);
        assertFalse(dir.isEmpty(), "cacheDir should return non-empty string");
    }

    @Test
    void initDoesNotThrow() {
        assertDoesNotThrow(() -> TsPackRegistry.init("{}"));
    }

    @Test
    void parsesPythonCode() {
        var tree = registry.parseString("python", "def hello(): pass\n");
        assertNotNull(tree);
        assertEquals("module", tree.rootNodeType());
        assertTrue(tree.rootChildCount() >= 1);
        assertFalse(tree.hasErrorNodes());
        tree.close();
    }

    @Test
    void errorsOnInvalidLanguage() {
        assertThrows(Exception.class, () ->
            registry.parseString("nonexistent_xyz_123", "code"));
    }

    @Test
    void hasLanguageReturnsFalseForNonexistent() {
        boolean result = registry.hasLanguage("nonexistent_xyz_123");
        assertFalse(result);
    }
}
