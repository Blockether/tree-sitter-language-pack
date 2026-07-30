package dev.kreuzberg.treesitterlanguagepack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import dev.kreuzberg.treesitterlanguagepack.StructuralApi.Op;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Exclusive tree-sitter end positions must not leak into the LINE-BASED edit
 * splice. Grammars whose definition node swallows its terminating newline
 * (Groovy's {@code command}, among others) report the FOLLOWING line as
 * {@code endLine} at column 0; treating that as covered content made
 * {@link StructuralApi.Op#REPLACE} delete whatever definition started there.
 */
final class StructuralApiSpanEndTest {

  private static final String GROOVY =
      "class Greeter {\n"
          + "    String hi(String n) { \"hi $n\" }\n"
          + "}\n"
          + "\n"
          + "interface I { int m() }\n"
          + "\n"
          + "enum E { A, B }\n"
          + "\n"
          + "def add(a, b) { a + b }\n";

  @Test
  @DisplayName("replacing a node that ends at column 0 keeps the next definition")
  void replaceDoesNotSwallowFollowingDefinition() throws Exception {
    final String out = StructuralApi.edit(GROOVY, "groovy", Op.REPLACE, "I", null, "interface I { int m(); long n() }");
    assertTrue(out.contains("enum E { A, B }"), "enum E was deleted:\n" + out);
    assertTrue(out.contains("def add(a, b)"), "add was deleted:\n" + out);
    assertTrue(out.contains("long n()"), "replacement not applied:\n" + out);
    assertTrue(out.contains("class Greeter {"), "preceding class was damaged:\n" + out);
  }

  @Test
  @DisplayName("a span ending at column 0 reports the last content line, not the next one")
  void flattenReportsLastContentLine() throws Exception {
    final var targets = StructuralApi.outline(GROOVY, "groovy");
    final var i = targets.stream().filter(t -> "I".equals(t.name())).findFirst().orElseThrow();
    // "interface I { int m() }" is line 5 and occupies exactly that one line.
    assertEquals(5, i.startLine(), "start line");
    assertEquals(5, i.endLine(), "end line must not spill onto the blank line 6");
  }
}
