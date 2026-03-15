<?php

declare(strict_types=1);

namespace TreeSitterLanguagePack;

/**
 * Thin wrapper around the tree-sitter-language-pack PHP extension.
 *
 * All parsing logic lives in Rust; this class provides a convenient OOP interface.
 */
final class TreeSitterLanguagePack
{
    /**
     * Get the library version.
     *
     * @return string Version string in semver format (e.g., "1.0.0-rc.1")
     */
    public static function version(): string
    {
        return \ts_pack_version();
    }

    /**
     * Get a list of all available language names.
     *
     * @return list<string> Sorted array of language name strings
     */
    public static function availableLanguages(): array
    {
        return \ts_pack_available_languages();
    }

    /**
     * Check whether a language is available.
     *
     * @param string $name The language name to check
     * @return bool True if the language is available
     */
    public static function hasLanguage(string $name): bool
    {
        return \ts_pack_has_language($name);
    }

    /**
     * Get the number of available languages.
     *
     * @return int The count of available languages
     */
    public static function languageCount(): int
    {
        return \ts_pack_language_count();
    }

    /**
     * Get a raw language pointer as an integer handle.
     *
     * @param string $name The language name to look up
     * @return int The raw language pointer as an integer
     * @throws \Exception If the language is not available
     */
    public static function getLanguage(string $name): int
    {
        return \ts_pack_get_language($name);
    }

    /**
     * Parse source code and return an S-expression representation.
     *
     * @param string $language The language name to use for parsing
     * @param string $source The source code to parse
     * @return string The S-expression string of the parsed tree
     * @throws \Exception If the language is not available or parsing fails
     */
    public static function parseString(string $language, string $source): string
    {
        return \ts_pack_parse_string($language, $source);
    }

    /**
     * Process source code and extract metadata + chunks.
     *
     * @param string $source The source code to process
     * @param ProcessConfig|array<string, mixed> $config Configuration with at least 'language' key.
     *   Can be a ProcessConfig object or an associative array.
     *   Optional keys: structure, imports, exports, comments, docstrings,
     *   symbols, diagnostics (bool), chunk_max_size (int|null).
     * @return array<string, mixed> Extraction results
     * @throws \RuntimeException If processing fails
     */
    public static function process(string $source, ProcessConfig|array $config): array
    {
        $configArray = $config instanceof ProcessConfig ? $config->toArray() : $config;
        $configJson = json_encode($configArray, JSON_THROW_ON_ERROR);
        $resultJson = \ts_pack_process($source, $configJson);

        return json_decode($resultJson, true, 512, JSON_THROW_ON_ERROR);
    }
}
