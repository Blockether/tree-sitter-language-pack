package dev.kreuzberg.treesitterlanguagepack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import dev.kreuzberg.treesitterlanguagepack.StructuralApi.EditException;
import dev.kreuzberg.treesitterlanguagepack.StructuralApi.Op;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Doc add/replace operations of {@link StructuralApi}, focused on Python (the
 * "doc in body" placement) where indentation and inline suites are easy to get
 * wrong. Every "OK" result is re-parsed and asserted to have no syntax errors.
 */
final class StructuralApiDocTest {

  private static String edit(final String src, final Op op, final String target, final String code)
      throws Exception {
    return StructuralApi.edit(src, "python", op, target, null, code);
  }

  /** Re-parse {@code src} and assert tree-sitter reports no ERROR/MISSING nodes. */
  private static void assertNoSyntaxErrors(final String src) throws Exception {
    final ProcessConfig cfg = ProcessConfig.builder().withLanguage("python").withDiagnostics(true).build();
    final ProcessResult res = TreeSitterLanguagePack.process(src, cfg);
    final List<Diagnostic> diags = res.diagnostics();
    if (diags != null) {
      assertTrue(diags.isEmpty(), "expected no diagnostics, got: " + diags + "\nin:\n" + src);
    }
  }

  @Test
  @DisplayName("add_doc places a docstring as the first body statement of a function")
  void addDocFunction() throws Exception {
    final String out = edit("def greet(name):\n    return 'hi ' + name\n", Op.ADD_DOC, "greet",
        "\"\"\"Greet someone.\"\"\"");
    assertEquals("def greet(name):\n    \"\"\"Greet someone.\"\"\"\n    return 'hi ' + name\n", out);
    assertNoSyntaxErrors(out);
  }

  @Test
  @DisplayName("add_doc rewrites an inline suite onto an indented block (was: 'return' outside function)")
  void addDocInlineSuite() throws Exception {
    // Regression: `def f(): return 1` is a single-line suite. A naive splice
    // dedents the body to column 0 — tree-sitter parses it, but Python rejects
    // it with "'return' outside function". The body must move under the header.
    final String out = edit("def f(): return 1\n", Op.ADD_DOC, "f", "\"\"\"Doc.\"\"\"");
    assertEquals("def f(): \n    \"\"\"Doc.\"\"\"\n    return 1\n", out);
    assertNoSyntaxErrors(out);
  }

  @Test
  @DisplayName("add_doc on a nested method uses the method's (deeper) indentation")
  void addDocNestedMethod() throws Exception {
    final String out = edit("class Foo:\n    def m(self, x):\n        return x * 2\n", Op.ADD_DOC, "m",
        "\"\"\"Double x.\"\"\"");
    assertEquals(
        "class Foo:\n    def m(self, x):\n        \"\"\"Double x.\"\"\"\n        return x * 2\n", out);
    assertNoSyntaxErrors(out);
  }

  @Test
  @DisplayName("add_doc on a class places the docstring before the first member")
  void addDocClass() throws Exception {
    final String out = edit("class Foo:\n    def m(self):\n        pass\n", Op.ADD_DOC, "Foo",
        "\"\"\"A foo.\"\"\"");
    assertEquals("class Foo:\n    \"\"\"A foo.\"\"\"\n    def m(self):\n        pass\n", out);
    assertNoSyntaxErrors(out);
  }

  @Test
  @DisplayName("add_doc preserves tab indentation")
  void addDocTabIndented() throws Exception {
    final String out = edit("def f():\n\treturn 1\n", Op.ADD_DOC, "f", "\"\"\"Doc.\"\"\"");
    assertEquals("def f():\n\t\"\"\"Doc.\"\"\"\n\treturn 1\n", out);
    assertNoSyntaxErrors(out);
  }

  @Test
  @DisplayName("add_doc with a null target adds a module-level docstring at the top")
  void addDocModule() throws Exception {
    final String out = edit("import os\n\n\ndef f():\n    pass\n", Op.ADD_DOC, null, "\"\"\"Module.\"\"\"");
    assertEquals("\"\"\"Module.\"\"\"\n\nimport os\n\n\ndef f():\n    pass\n", out);
    assertNoSyntaxErrors(out);
  }

  @Test
  @DisplayName("add_doc with a null target inserts the module docstring after a shebang")
  void addDocModuleAfterShebang() throws Exception {
    final String out = edit("#!/usr/bin/env python\nimport os\n", Op.ADD_DOC, null, "\"\"\"Mod.\"\"\"");
    assertEquals("#!/usr/bin/env python\n\"\"\"Mod.\"\"\"\n\nimport os\n", out);
    assertNoSyntaxErrors(out);
  }

  @Test
  @DisplayName("add_doc refuses when the target already has a docstring")
  void addDocRefusesExisting() {
    final EditException e = assertThrows(EditException.class, () ->
        edit("def f():\n    \"\"\"Old.\"\"\"\n    return 1\n", Op.ADD_DOC, "f", "\"\"\"Dup.\"\"\""));
    assertTrue(e.getMessage().contains("already has a doc string"), e.getMessage());
  }

  @Test
  @DisplayName("add_doc refuses an ambiguous (duplicated) name")
  void addDocRefusesAmbiguous() {
    assertThrows(EditException.class, () ->
        edit("def f():\n    def f():\n        pass\n    return f\n", Op.ADD_DOC, "f", "\"\"\"D.\"\"\""));
  }

  @Test
  @DisplayName("replace_doc swaps the existing docstring in place")
  void replaceDoc() throws Exception {
    final String out = edit("def f(x):\n    \"\"\"Old.\"\"\"\n    return x\n", Op.REPLACE_DOC, "f",
        "\"\"\"New.\"\"\"");
    assertEquals("def f(x):\n    \"\"\"New.\"\"\"\n    return x\n", out);
    assertNoSyntaxErrors(out);
  }

  @Test
  @DisplayName("replace_doc accepts a multi-line replacement docstring")
  void replaceDocMultiline() throws Exception {
    final String out = edit("def f(x):\n    \"\"\"Old.\"\"\"\n    return x\n", Op.REPLACE_DOC, "f",
        "\"\"\"One.\n\n    Args:\n        x: a value.\n    \"\"\"");
    assertNoSyntaxErrors(out);
    assertTrue(out.contains("Args:"), out);
    assertTrue(out.contains("    return x"), out);
  }

  @Test
  @DisplayName("replace_doc refuses when there is no docstring to replace")
  void replaceDocRefusesMissing() {
    final EditException e = assertThrows(EditException.class, () ->
        edit("def f():\n    return 1\n", Op.REPLACE_DOC, "f", "\"\"\"New.\"\"\""));
    assertTrue(e.getMessage().contains("No existing doc string"), e.getMessage());
  }

  @Test
  @DisplayName("add_doc / replace_doc work on a Clojure defn (doc after the name)")
  void clojureDocRoundTrip() throws Exception {
    final String added =
        StructuralApi.edit("(defn f [x] x)", "clojure", Op.ADD_DOC, "f", null, "\"Doc.\"");
    assertTrue(added.contains("\"Doc.\""), added);
    final String replaced =
        StructuralApi.edit("(defn f \"Old.\" [x] x)", "clojure", Op.REPLACE_DOC, "f", null, "\"New.\"");
    assertTrue(replaced.contains("\"New.\"") && !replaced.contains("Old."), replaced);
  }
}
