//! PHP bindings for tree-sitter-language-pack.
//!
//! This module exposes the Rust core parsing API to PHP using ext-php-rs.
//!
//! # Architecture
//!
//! - All parsing logic is in the Rust core (ts-pack-core)
//! - PHP is a thin wrapper that adds language-specific features
//! - Zero duplication of core functionality

#![cfg_attr(windows, feature(abi_vectorcall))]

use ext_php_rs::prelude::*;

/// Get the library version.
///
/// # Returns
///
/// Version string in semver format (e.g., "1.0.0-rc.1")
///
/// # Example
///
/// ```php
/// $version = ts_pack_version();
/// echo "Version: $version\n";
/// ```
#[php_function]
pub fn ts_pack_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get a list of all available language names.
///
/// # Returns
///
/// Array of language name strings sorted alphabetically.
///
/// # Example
///
/// ```php
/// $languages = ts_pack_available_languages();
/// foreach ($languages as $lang) {
///     echo "$lang\n";
/// }
/// ```
#[php_function]
pub fn ts_pack_available_languages() -> Vec<String> {
    ts_pack_core::available_languages()
}

/// Check whether a language is available.
///
/// # Arguments
///
/// * `name` - The language name to check.
///
/// # Returns
///
/// `true` if the language is available, `false` otherwise.
///
/// # Example
///
/// ```php
/// if (ts_pack_has_language("python")) {
///     echo "Python is available!\n";
/// }
/// ```
#[php_function]
pub fn ts_pack_has_language(name: String) -> bool {
    ts_pack_core::has_language(&name)
}

/// Get the number of available languages.
///
/// # Returns
///
/// The count of available languages as an integer.
///
/// # Example
///
/// ```php
/// $count = ts_pack_language_count();
/// echo "Available languages: $count\n";
/// ```
#[php_function]
pub fn ts_pack_language_count() -> i64 {
    ts_pack_core::language_count() as i64
}

/// Process source code and extract metadata + chunks as a JSON string.
///
/// The config JSON must contain at least `"language"`. Optional fields:
/// - `structure` (bool, default true): Extract structural items (functions, classes, etc.)
/// - `imports` (bool, default true): Extract import statements
/// - `exports` (bool, default true): Extract export statements
/// - `comments` (bool, default false): Extract comments
/// - `docstrings` (bool, default false): Extract docstrings
/// - `symbols` (bool, default false): Extract symbol definitions
/// - `diagnostics` (bool, default false): Include parse diagnostics
/// - `chunk_max_size` (int or null, default null): Maximum chunk size in bytes
///
/// # Arguments
///
/// * `source` - The source code to process.
/// * `config_json` - JSON string with processing configuration.
///
/// # Returns
///
/// JSON string with extraction results.
///
/// # Throws
///
/// Throws an exception if the config JSON is invalid, the language is unknown,
/// or processing fails.
///
/// # Example
///
/// ```php
/// $result = ts_pack_process("def hello(): pass", '{"language":"python"}');
/// $data = json_decode($result, true);
/// echo "Functions: " . count($data['structure']) . "\n";
/// ```
#[php_function]
pub fn ts_pack_process(source: String, config_json: String) -> PhpResult<String> {
    let core_config: ts_pack_core::ProcessConfig = serde_json::from_str(&config_json)
        .map_err(|e| PhpException::default(format!("invalid config JSON: {e}")))?;

    let result = ts_pack_core::process(&source, &core_config)
        .map_err(|e| PhpException::default(format!("{e}")))?;

    serde_json::to_string(&result)
        .map_err(|e| PhpException::default(format!("serialization failed: {e}")))
}

/// tree-sitter-language-pack PHP extension module.
#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
