<?php

declare(strict_types=1);

/**
 * Smoke test for tree-sitter-language-pack PHP extension.
 *
 * Run: php examples/php_smoke/convert.php
 */

// Check extension is loaded
if (!function_exists('ts_pack_version')) {
    echo "ERROR: ts_pack extension not loaded. Build and enable the extension first.\n";
    exit(1);
}

echo "tree-sitter-language-pack v" . ts_pack_version() . "\n";

// Language count
$count = ts_pack_language_count();
echo "Available languages: $count\n";
assert($count > 0, "Expected at least one language");

// Check specific language
$hasPython = ts_pack_has_language("python");
echo "Has Python: " . ($hasPython ? "yes" : "no") . "\n";
assert($hasPython, "Expected Python to be available");

// List some languages
$languages = ts_pack_available_languages();
echo "First 5 languages: " . implode(", ", array_slice($languages, 0, 5)) . "\n";

// Process source code
$source = "def hello():\n    pass\n";
$result = ts_pack_process($source, '{"language":"python"}');
$data = json_decode($result, true);
echo "Process result keys: " . implode(", ", array_keys($data)) . "\n";

echo "\nSmoke test passed.\n";
