# Hand-written capsule-passthrough test (not fixture-generated).
# Verifies that `get_language` returns a standalone-`tree_sitter` `Language`
# usable with the upstream `tree_sitter.Parser` — host-native passthrough (#143).
import tree_sitter
from tree_sitter_language_pack import get_language


def test_get_language_returns_native_tree_sitter_language() -> None:
    language = get_language("python")
    assert isinstance(language, tree_sitter.Language)

    parser = tree_sitter.Parser(language)
    tree = parser.parse(b"def greet(name):\n    return name\n")

    assert tree.root_node.type == "module"
    assert tree.root_node.children[0].type == "function_definition"


def test_distinct_languages_parse_their_own_syntax() -> None:
    javascript = get_language("javascript")
    assert isinstance(javascript, tree_sitter.Language)

    parser = tree_sitter.Parser(javascript)
    tree = parser.parse(b"const x = 1;\n")

    assert tree.root_node.type == "program"
