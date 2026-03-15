pub mod c;
pub mod csharp;
pub mod elixir;
pub mod go;
pub mod java;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod typescript;
pub mod wasm;

use crate::fixtures::Fixture;
use std::path::Path;

/// Trait implemented by each language generator.
pub trait Generator {
    /// Generate test files for the given fixtures into the output directory.
    fn generate(&self, fixtures: &[Fixture], output_dir: &Path) -> Result<(), String>;

    /// The name of the target language (for logging).
    fn name(&self) -> &'static str;
}

/// Generate tests for the specified language target.
pub fn generate_for_lang(lang: &str, fixtures: &[Fixture], output_dir: &Path) -> Result<(), String> {
    let generator: Box<dyn Generator> = match lang {
        "rust" => Box::new(rust::RustGenerator),
        "python" => Box::new(python::PythonGenerator),
        "typescript" => Box::new(typescript::TypeScriptGenerator),
        "go" => Box::new(go::GoGenerator),
        "java" => Box::new(java::JavaGenerator),
        "elixir" => Box::new(elixir::ElixirGenerator),
        "ruby" => Box::new(ruby::RubyGenerator),
        "c" => Box::new(c::CGenerator),
        "wasm" => Box::new(wasm::WasmGenerator),
        "php" => Box::new(php::PhpGenerator),
        "csharp" => Box::new(csharp::CSharpGenerator),
        _ => return Err(format!("Unknown language target: {lang}")),
    };

    eprintln!("Generating {} tests...", generator.name());
    generator.generate(fixtures, output_dir)?;
    eprintln!("  Done.");

    Ok(())
}

/// All supported language targets.
pub const ALL_TARGETS: &[&str] = &[
    "rust",
    "python",
    "typescript",
    "go",
    "java",
    "elixir",
    "ruby",
    "c",
    "wasm",
    "php",
    "csharp",
];
