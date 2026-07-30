package dev.kreuzberg.treesitterlanguagepack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * {@link StructuralApi#replaceNode} matches a whole syntax node <em>or</em> a
 * contiguous run of sibling nodes. The sibling-run cases guard the regression
 * where a snippet spanning several adjacent forms (e.g. a couple of let-binding
 * pairs) was rejected with "No node matching the snippet".
 */
final class StructuralApiReplaceNodeTest {

  private static final String CREATE =
      "(ns foo)\n\n"
          + "(defn create!\n"
          + "  \"doc\"\n"
          + "  [opts]\n"
          + "  (let [cert\n"
          + "        (make-cert opts)\n\n"
          + "        ca-file\n"
          + "        (write-ca-pem cert)\n\n"
          + "        java-trust\n"
          + "        (write-java-truststore cert)]\n"
          + "    (merge {:ca-cert cert})))\n";

  @Test
  @DisplayName("replaces a contiguous run of sibling forms scoped to a definition")
  void replacesSiblingRunInsideTarget() throws Exception {
    final String match =
        "ca-file (write-ca-pem cert)\n  java-trust (write-java-truststore cert)";
    final String code =
        "roots (default-root-certificates)\n"
            + "        ca-file (write-ca-pem cert roots)\n"
            + "        java-trust (write-java-truststore cert roots)";
    final String out = StructuralApi.replaceNode(CREATE, "clojure", match, code, "create!", null);
    assertTrue(out.contains("roots (default-root-certificates)"), out);
    assertTrue(out.contains("(write-ca-pem cert roots)"), out);
    assertTrue(out.contains("(write-java-truststore cert roots)"), out);
    assertFalse(out.contains("(write-ca-pem cert)\n"), "old single-form binding left behind:\n" + out);
  }

  @Test
  @DisplayName("replaces a run of adjacent top-level forms")
  void replacesTopLevelRun() throws Exception {
    final String out = StructuralApi.replaceNode("(a)\n(b)\n(c)\n", "clojure", "(a)\n(b)", "(ab)", null, null);
    assertEquals("(ab)\n(c)\n", out);
  }

  @Test
  @DisplayName("still replaces a single whole node")
  void replacesSingleNode() throws Exception {
    final String out =
        StructuralApi.replaceNode(CREATE, "clojure", "(make-cert opts)", "(make-cert opts extra)", "create!", null);
    assertTrue(out.contains("(make-cert opts extra)"), out);
  }

  @Test
  @DisplayName("an ambiguous sibling run is refused rather than guessed")
  void refusesAmbiguousRun() {
    final StructuralApi.EditException ex =
        assertThrows(
            StructuralApi.EditException.class,
            () -> StructuralApi.replaceNode("(do (a) (b) (c) (a) (b) (c))\n", "clojure", "(a) (b)", "(z)", null, null));
    assertTrue(ex.getMessage().contains("nodes match"), ex.getMessage());
  }

  @Test
  @DisplayName("a snippet that matches nothing still reports no match")
  void reportsNoMatch() {
    final StructuralApi.EditException ex =
        assertThrows(
            StructuralApi.EditException.class,
            () -> StructuralApi.replaceNode(CREATE, "clojure", "nonexistent-token-xyz", "x", "create!", null));
    assertTrue(ex.getMessage().contains("No node matching"), ex.getMessage());
  }

  @Test
  @DisplayName("a run replacement that would break syntax is rejected")
  void rejectsSyntaxBreak() {
    final StructuralApi.EditException ex =
        assertThrows(
            StructuralApi.EditException.class,
            () -> StructuralApi.replaceNode("(a)\n(b)\n(c)\n", "clojure", "(a)\n(b)", "(oops", null, null));
    assertTrue(ex.getMessage().contains("syntax error"), ex.getMessage());
  }

  @Test
  @DisplayName("replace_node with a node's own text is a byte-identity edit (Groovy)")
  void replaceNodeIdentityKeepsSeparators() throws Exception {
    final String src = "class Alpha {\n    String hi(String n) { \"hi $n\" }\n}\n\ninterface Beta { int m() }\n";
    final String own = "class Alpha {\n    String hi(String n) { \"hi $n\" }\n}";
    assertEquals(src, StructuralApi.replaceNode(src, "groovy", own, own, null, null),
        "replacing a node with its own text must not touch the whitespace around it");
  }

  @Test
  @DisplayName("replace_node keeps the final newline when the match is the last definition")
  void replaceNodeKeepsFinalNewline() throws Exception {
    final String src = "def alpha() { 1 }\n\ndef beta() { 2 }\n";
    final String out = StructuralApi.replaceNode(src, "groovy", "def beta() { 2 }", "def beta() { 3 }", null, null);
    assertEquals("def alpha() { 1 }\n\ndef beta() { 3 }\n", out);
  }
}
