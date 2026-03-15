<?php

declare(strict_types=1);

namespace TreeSitterLanguagePack;

/**
 * Configuration for source code processing.
 *
 * Encapsulates the configuration options passed to the Rust core's process function.
 * At minimum, a language name is required. All other options have sensible defaults.
 */
final class ProcessConfig
{
    /**
     * @param string $language      Language name (required)
     * @param bool   $structure     Extract structural items (functions, classes, etc.)
     * @param bool   $imports       Extract import statements
     * @param bool   $exports       Extract export statements
     * @param bool   $comments      Extract comments
     * @param bool   $docstrings    Extract docstrings
     * @param bool   $symbols       Extract symbol definitions
     * @param bool   $diagnostics   Include parse diagnostics
     * @param int|null $chunkMaxSize Maximum chunk size in bytes (null disables chunking)
     */
    public function __construct(
        public readonly string $language,
        public readonly bool $structure = true,
        public readonly bool $imports = true,
        public readonly bool $exports = true,
        public readonly bool $comments = false,
        public readonly bool $docstrings = false,
        public readonly bool $symbols = false,
        public readonly bool $diagnostics = false,
        public readonly ?int $chunkMaxSize = null,
    ) {}

    /**
     * Convert the configuration to an associative array suitable for JSON encoding.
     *
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        $config = ['language' => $this->language];

        foreach (['structure', 'imports', 'exports', 'comments', 'docstrings', 'symbols', 'diagnostics'] as $field) {
            $config[$field] = $this->$field;
        }

        if ($this->chunkMaxSize !== null) {
            $config['chunk_max_size'] = $this->chunkMaxSize;
        }

        return $config;
    }
}
