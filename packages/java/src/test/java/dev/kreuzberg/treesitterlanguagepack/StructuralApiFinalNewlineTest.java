package dev.kreuzberg.treesitterlanguagepack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import dev.kreuzberg.treesitterlanguagepack.StructuralApi.Op;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * A structural edit must never strip the file's terminating newline. INSERT_AFTER on the
 * LAST definition trims the whole tail away, and the tail of a newline-terminated file is
 * exactly the empty element {@code String.split("\n", -1)} leaves behind — so the result
 * used to end mid-line. For most grammars that is a silent POSIX/diff regression; Groovy,
 * whose grammar requires a terminating newline, failed the syntax gate outright and made
 * "insert after the last definition" impossible.
 */
final class StructuralApiFinalNewlineTest {

  private static final String PYTHON = "def alpha(x):\n    return x + 1\n\n\ndef beta():\n    return 2\n";

  private static final String GROOVY =
      "class Greeter {\n"
          + "    String hi(String n) { \"hi $n\" }\n"
          + "}\n"
          + "\n"
          + "def add(a, b) { a + b }\n";

  @Test
  @DisplayName("insert_after the last definition keeps the final newline")
  void insertAfterLastKeepsFinalNewline() throws Exception {
    final String out = StructuralApi.edit(PYTHON, "python", Op.INSERT_AFTER, "beta", null, "def gamma():\n    return 3");
    assertTrue(out.endsWith("\n"), "final newline was stripped:\n" + out);
    assertTrue(out.contains("def gamma():"), "insertion missing:\n" + out);
    assertTrue(out.contains("def alpha(x):"), "earlier definition damaged:\n" + out);
    assertEquals("def gamma():\n    return 3\n", out.substring(out.indexOf("def gamma():")),
        "the inserted node must sit at the end, newline-terminated:\n" + out);
  }

  @Test
  @DisplayName("insert_after the last Groovy definition is not rejected by the syntax gate")
  void insertAfterLastGroovyDefinition() throws Exception {
    final String out = StructuralApi.edit(GROOVY, "groovy", Op.INSERT_AFTER, "add", null, "def sub(a, b) { a - b }");
    assertTrue(out.endsWith("\n"), "final newline was stripped:\n" + out);
    assertTrue(out.contains("def sub(a, b)"), "insertion missing:\n" + out);
    assertTrue(out.contains("def add(a, b)"), "target definition was lost:\n" + out);
  }

  @Test
  @DisplayName("a file without a final newline does not gain one from a plain edit")
  void sourceWithoutFinalNewlineIsLeftAlone() throws Exception {
    final String src = "def alpha(x):\n    return x + 1";
    final String out = StructuralApi.edit(src, "python", Op.REPLACE, "alpha", null, "def alpha(x):\n    return x + 2");
    assertTrue(!out.endsWith("\n"), "a final newline was invented:\n" + out);
  }

  @Test
  @DisplayName("a CRLF file keeps CRLF endings on inserted and separator lines")
  void crlfFileStaysCrlf() throws Exception {
    final String src = PYTHON.replace("\n", "\r\n");
    final String out = StructuralApi.edit(src, "python", Op.INSERT_AFTER, "alpha", null, "def gamma():\n    return 3");
    assertTrue(out.endsWith("\r\n"), "final CRLF was stripped:\n" + out.replace("\r", "<CR>"));
    assertTrue(out.contains("def gamma():\r\n"), "inserted line is LF-only:\n" + out.replace("\r", "<CR>"));
    assertEquals(0, out.replace("\r\n", "").chars().filter(c -> c == '\n' || c == '\r').count(),
        "mixed line endings after the edit:\n" + out.replace("\r", "<CR>"));
  }
}
