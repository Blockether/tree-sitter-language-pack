package dev.kreuzberg.treesitterlanguagepack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Direct tests for {@link JsonCodec}, the charred-backed (Jackson-free) FFI JSON
 * codec. These exercise the read/write paths WITHOUT the native library, so they
 * run anywhere and pin the snake_case wire shape + reflection-free DTO building.
 */
final class JsonCodecTest {

  @Test
  @DisplayName("writeConfig emits snake_case keys and omits unset (null) fields")
  void writeProcessConfig() {
    final ProcessConfig cfg = ProcessConfig.builder()
        .withLanguage("python")
        .withStructure(true)
        .withChunkMaxSize(2048L)
        .build();
    final String json = JsonCodec.writeConfig(cfg);
    assertTrue(json.contains("\"language\":\"python\""), json);
    assertTrue(json.contains("\"structure\":true"), json);
    assertTrue(json.contains("\"chunk_max_size\":2048"), json);
  }

  @Test
  @DisplayName("writeConfig(PackConfig) renders cache_dir as a string and the lists")
  void writePackConfig() {
    final PackConfig cfg = PackConfig.builder()
        .withLanguages(List.of("python", "clojure"))
        .build();
    final String json = JsonCodec.writeConfig(cfg);
    assertTrue(json.contains("\"languages\":[\"python\",\"clojure\"]"), json);
  }

  @Test
  @DisplayName("readProcessResult builds nested DTOs + enums reflection-free")
  void readProcessResultRoundTrip() {
    final String json = "{\"language\":\"python\",\"structure\":["
        + "{\"kind\":\"class\",\"name\":\"Foo\",\"span\":{\"start_byte\":0,\"end_byte\":40,"
        + "\"start_line\":1,\"start_column\":0,\"end_line\":4,\"end_column\":0},\"children\":["
        + "{\"kind\":\"function\",\"name\":\"m\",\"span\":{\"start_byte\":12,\"end_byte\":38,"
        + "\"start_line\":2,\"start_column\":4,\"end_line\":3,\"end_column\":0}}]}],"
        + "\"docstrings\":[{\"text\":\"d\",\"format\":\"pythontriplequote\",\"span\":"
        + "{\"start_byte\":1,\"end_byte\":3,\"start_line\":1,\"start_column\":1,\"end_line\":1,"
        + "\"end_column\":3},\"associated_item\":\"Foo\"}],"
        + "\"diagnostics\":[{\"message\":\"oops\",\"severity\":\"error\",\"span\":"
        + "{\"start_byte\":0,\"end_byte\":1,\"start_line\":1,\"start_column\":0,\"end_line\":1,"
        + "\"end_column\":1}}]}";
    final ProcessResult r = JsonCodec.readProcessResult(json);

    assertEquals("python", r.language());
    assertEquals(1, r.structure().size());
    final StructureItem foo = r.structure().get(0);
    assertEquals("Foo", foo.name());
    assertEquals(StructureKind.Class, foo.kind());
    assertEquals(0L, foo.span().startByte());
    assertEquals(40L, foo.span().endByte());
    // nested child preserved with its own kind/span
    assertEquals(1, foo.children().size());
    assertEquals("m", foo.children().get(0).name());
    assertEquals(StructureKind.Function, foo.children().get(0).kind());
    assertEquals(12L, foo.children().get(0).span().startByte());

    assertEquals(1, r.docstrings().size());
    assertEquals("Foo", r.docstrings().get(0).associatedItem());
    assertEquals(DocstringFormat.PythonTripleQuote, r.docstrings().get(0).format());

    assertEquals(1, r.diagnostics().size());
    assertEquals(DiagnosticSeverity.Error, r.diagnostics().get(0).severity());
    assertEquals("oops", r.diagnostics().get(0).message());
  }

  @Test
  @DisplayName("absent optional lists deserialize to null (not empty)")
  void readProcessResultAbsentFields() {
    final ProcessResult r = JsonCodec.readProcessResult("{\"language\":\"go\"}");
    assertEquals("go", r.language());
    assertNull(r.structure());
    assertNull(r.docstrings());
  }

  @Test
  @DisplayName("readStringList parses a JSON array of strings")
  void readStringListParses() {
    assertEquals(List.of("a", "b", "c"), JsonCodec.readStringList("[\"a\",\"b\",\"c\"]"));
  }

  @Test
  @DisplayName("readByteRange / readPoint parse their flat shapes")
  void readByteRangeAndPoint() {
    final ByteRange br = JsonCodec.readByteRange("{\"start\":3,\"end\":9}");
    assertEquals(3L, br.start());
    assertEquals(9L, br.end());
    final Point p = JsonCodec.readPoint("{\"row\":2,\"column\":5}");
    assertEquals(2L, p.row());
    assertEquals(5L, p.column());
  }
}
