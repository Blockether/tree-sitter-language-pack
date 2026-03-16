#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include "ts_pack_ffi.h"

#define ASSERT(cond, msg) do { \
    if (!(cond)) { \
        fprintf(stderr, "FAIL: %s\n", msg); \
        failures++; \
    } else { \
        printf("  PASS: %s\n", msg); \
    } \
} while (0)

int main(void) {
    int failures = 0;

    printf("=== C Smoke Tests ===\n");

    /* Create registry */
    TsPackRegistry *registry = ts_pack_registry_new();
    ASSERT(registry != NULL, "registry creation");

    /* Language count */
    size_t count = ts_pack_registry_language_count(registry);
    ASSERT(count >= 100, "language_count >= 100");

    /* Has language */
    ASSERT(ts_pack_registry_has_language(registry, "python") == true, "has_language(python)");
    ASSERT(ts_pack_registry_has_language(registry, "javascript") == true, "has_language(javascript)");
    ASSERT(ts_pack_registry_has_language(registry, "rust") == true, "has_language(rust)");
    ASSERT(ts_pack_registry_has_language(registry, "go") == true, "has_language(go)");
    ASSERT(ts_pack_registry_has_language(registry, "nonexistent_xyz") == false, "has_language(nonexistent) == false");

    /* Available languages */
    size_t lang_count = 0;
    const char *const *langs = ts_pack_registry_available_languages(registry, &lang_count);
    ASSERT(lang_count >= 100, "available_languages count >= 100");
    ASSERT(langs != NULL, "available_languages not null");

    /* Parse Python code */
    TsPackTree *tree = ts_pack_parse_string(registry, "python", "def hello(): pass\n");
    ASSERT(tree != NULL, "parse_string(python) returns tree");

    const char *node_type = ts_pack_tree_root_node_type(tree);
    ASSERT(node_type != NULL && strcmp(node_type, "module") == 0, "root node type == module");

    size_t child_count = ts_pack_tree_root_child_count(tree);
    ASSERT(child_count >= 1, "root child count >= 1");

    bool has_errors = ts_pack_tree_has_error_nodes(tree);
    ASSERT(has_errors == false, "no error nodes");

    ts_pack_tree_free(tree);

    /* Invalid language returns NULL */
    TsPackTree *bad_tree = ts_pack_parse_string(registry, "nonexistent_xyz_123", "code");
    ASSERT(bad_tree == NULL, "parse_string(invalid) returns NULL");

    /* Cleanup */
    ts_pack_registry_free(registry);

    printf("\n%s (%d failures)\n", failures == 0 ? "All tests passed!" : "Some tests failed!", failures);
    return failures == 0 ? EXIT_SUCCESS : EXIT_FAILURE;
}
