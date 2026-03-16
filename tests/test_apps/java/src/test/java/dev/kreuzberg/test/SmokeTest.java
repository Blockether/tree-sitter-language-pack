package dev.kreuzberg.test;

import com.google.gson.Gson;
import com.google.gson.reflect.TypeToken;
import dev.kreuzberg.TreeSitterLanguagePack;
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
    private static final Path FIXTURES_DIR = Path.of("..", "fixtures");

    @SuppressWarnings("unchecked")
    private static List<Map<String, Object>> loadFixtures(String name) throws IOException {
        String json = Files.readString(FIXTURES_DIR.resolve(name));
        Type type = new TypeToken<List<Map<String, Object>>>() {}.getType();
        return GSON.fromJson(json, type);
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
                        int count = TreeSitterLanguagePack.languageCount();
                        int expectedMin = ((Number) fixture.get("expected_min")).intValue();
                        assertTrue(count >= expectedMin,
                            "language_count " + count + " < expected min " + expectedMin);
                    }
                    case "has_language" -> {
                        String language = (String) fixture.get("language");
                        boolean result = TreeSitterLanguagePack.hasLanguage(language);
                        boolean expected = (Boolean) fixture.get("expected");
                        assertEquals(expected, result,
                            "has_language(" + language + ") = " + result + ", expected " + expected);
                    }
                    case "available_languages" -> {
                        List<String> langs = TreeSitterLanguagePack.availableLanguages();
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

    @Test
    void parsesPythonCode() {
        var tree = TreeSitterLanguagePack.parseString("python", "def hello(): pass\n");
        assertNotNull(tree);
        assertEquals("module", tree.rootNodeType());
        assertTrue(tree.rootChildCount() >= 1);
        assertFalse(tree.hasErrorNodes());
    }

    @Test
    void errorsOnInvalidLanguage() {
        assertThrows(Exception.class, () ->
            TreeSitterLanguagePack.parseString("nonexistent_xyz_123", "code"));
    }
}
