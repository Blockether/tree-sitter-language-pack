//! File extension to language name mapping.
//!
//! Mappings are auto-generated from `sources/language_definitions.json` by `build.rs`.
//! To add or modify extension mappings, edit that JSON file and rebuild.

/// Detect language name from a file extension (without leading dot).
///
/// Returns `None` for unrecognized extensions. The match is case-insensitive.
///
/// ```
/// use tree_sitter_language_pack::detect_language_from_extension;
/// assert_eq!(detect_language_from_extension("py"), Some("python"));
/// assert_eq!(detect_language_from_extension("RS"), Some("rust"));
/// assert_eq!(detect_language_from_extension("xyz"), None);
/// ```
#[inline]
pub fn detect_language_from_extension(ext: &str) -> Option<&'static str> {
    include!(concat!(env!("OUT_DIR"), "/extensions_generated.rs"))
}

/// Detect language name from a file path.
///
/// Extracts the file extension and looks it up. Returns `None` if the
/// path has no extension or the extension is not recognized.
///
/// ```
/// use tree_sitter_language_pack::detect_language_from_path;
/// assert_eq!(detect_language_from_path("src/main.rs"), Some("rust"));
/// assert_eq!(detect_language_from_path("README.md"), Some("markdown"));
/// assert_eq!(detect_language_from_path("Makefile"), None);
/// ```
pub fn detect_language_from_path(path: &str) -> Option<&'static str> {
    let ext = std::path::Path::new(path).extension()?.to_str()?;
    detect_language_from_extension(ext)
}

/// Check if a file extension is ambiguous — i.e. it could reasonably belong to
/// multiple languages.
///
/// Returns `Some((assigned_language, alternatives))` if the extension is known
/// to be ambiguous, where `assigned_language` is what [`detect_language_from_extension`]
/// returns and `alternatives` lists other languages it could also belong to.
///
/// Returns `None` if the extension is unambiguous or unrecognized.
///
/// ```
/// use tree_sitter_language_pack::extension_ambiguity;
/// // .m is assigned to objc but could also be matlab
/// if let Some((assigned, alternatives)) = extension_ambiguity("m") {
///     assert_eq!(assigned, "objc");
///     assert!(alternatives.contains(&"matlab"));
/// }
/// // .py is unambiguous
/// assert!(extension_ambiguity("py").is_none());
/// ```
pub fn extension_ambiguity(ext: &str) -> Option<(&'static str, &'static [&'static str])> {
    let mut buf = [0u8; 32];
    let ext_lower = if ext.len() <= buf.len() && ext.is_ascii() {
        for (i, b) in ext.bytes().enumerate() {
            buf[i] = b.to_ascii_lowercase();
        }
        std::str::from_utf8(&buf[..ext.len()]).ok()?
    } else {
        return None;
    };

    include!(concat!(env!("OUT_DIR"), "/ambiguities_generated.rs"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_extensions() {
        assert_eq!(detect_language_from_extension("py"), Some("python"));
        assert_eq!(detect_language_from_extension("pyi"), Some("python"));
        assert_eq!(detect_language_from_extension("rs"), Some("rust"));
        assert_eq!(detect_language_from_extension("js"), Some("javascript"));
        assert_eq!(detect_language_from_extension("ts"), Some("typescript"));
        assert_eq!(detect_language_from_extension("c"), Some("c"));
        assert_eq!(detect_language_from_extension("h"), Some("c"));
        assert_eq!(detect_language_from_extension("cpp"), Some("cpp"));
        assert_eq!(detect_language_from_extension("go"), Some("go"));
        assert_eq!(detect_language_from_extension("rb"), Some("ruby"));
        assert_eq!(detect_language_from_extension("java"), Some("java"));
        assert_eq!(detect_language_from_extension("cs"), Some("csharp"));
        assert_eq!(detect_language_from_extension("tsx"), Some("tsx"));
        assert_eq!(detect_language_from_extension("html"), Some("html"));
        assert_eq!(detect_language_from_extension("css"), Some("css"));
        assert_eq!(detect_language_from_extension("json"), Some("json"));
        assert_eq!(detect_language_from_extension("yaml"), Some("yaml"));
        assert_eq!(detect_language_from_extension("toml"), Some("toml"));
        assert_eq!(detect_language_from_extension("sql"), Some("sql"));
        assert_eq!(detect_language_from_extension("md"), Some("markdown"));
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(detect_language_from_extension("PY"), Some("python"));
        assert_eq!(detect_language_from_extension("Rs"), Some("rust"));
        assert_eq!(detect_language_from_extension("JS"), Some("javascript"));
        assert_eq!(detect_language_from_extension("CPP"), Some("cpp"));
        assert_eq!(detect_language_from_extension("Tsx"), Some("tsx"));
    }

    #[test]
    fn test_unknown() {
        assert_eq!(detect_language_from_extension("xyz"), None);
        assert_eq!(detect_language_from_extension(""), None);
        assert_eq!(detect_language_from_extension("abcdef"), None);
    }

    #[test]
    fn test_path_detection() {
        assert_eq!(detect_language_from_path("src/main.rs"), Some("rust"));
        assert_eq!(detect_language_from_path("/path/to/file.py"), Some("python"));
        assert_eq!(detect_language_from_path("README.md"), Some("markdown"));
        assert_eq!(detect_language_from_path("app.test.tsx"), Some("tsx"));
        assert_eq!(detect_language_from_path("Cargo.toml"), Some("toml"));
    }

    #[test]
    fn test_path_no_extension() {
        assert_eq!(detect_language_from_path("Makefile"), None);
        assert_eq!(detect_language_from_path(""), None);
        assert_eq!(detect_language_from_path("/usr/bin/env"), None);
    }

    #[test]
    fn test_long_extension_rejected() {
        let long = "a".repeat(33);
        assert_eq!(detect_language_from_extension(&long), None);
    }

    #[test]
    fn test_ambiguity_known() {
        // .m is ambiguous: assigned to objc, but could be matlab
        let result = extension_ambiguity("m");
        assert!(result.is_some(), ".m should be flagged as ambiguous");
        let (assigned, alternatives) = result.unwrap();
        assert_eq!(assigned, "objc");
        assert!(alternatives.contains(&"matlab"));

        // .h is ambiguous: assigned to c, but could be cpp or objc
        let result = extension_ambiguity("h");
        assert!(result.is_some(), ".h should be flagged as ambiguous");
        let (assigned, alternatives) = result.unwrap();
        assert_eq!(assigned, "c");
        assert!(alternatives.contains(&"cpp"));

        // .v is ambiguous: assigned to v, but could be verilog
        let result = extension_ambiguity("v");
        assert!(result.is_some(), ".v should be flagged as ambiguous");
        let (assigned, alternatives) = result.unwrap();
        assert_eq!(assigned, "v");
        assert!(alternatives.contains(&"verilog"));
    }

    #[test]
    fn test_ambiguity_unambiguous() {
        // .py is not ambiguous
        assert!(extension_ambiguity("py").is_none());
        // .rs is not ambiguous
        assert!(extension_ambiguity("rs").is_none());
        // unknown extension is not ambiguous
        assert!(extension_ambiguity("xyz").is_none());
    }

    #[test]
    fn test_ambiguity_case_insensitive() {
        assert!(extension_ambiguity("M").is_some());
        assert!(extension_ambiguity("H").is_some());
    }

    /// Validate that JSON definitions match generated code by round-tripping.
    /// Loads language_definitions.json at test time and checks every extension
    /// resolves correctly via the generated lookup.
    #[test]
    fn test_roundtrip_json_to_generated() {
        let json_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../sources/language_definitions.json");
        let json_str = match std::fs::read_to_string(json_path) {
            Ok(s) => s,
            Err(_) => return, // Skip when sources/ not available (e.g. crates.io install)
        };
        let defs: std::collections::BTreeMap<String, serde_json::Value> =
            serde_json::from_str(&json_str).expect("Failed to parse language_definitions.json");

        for (lang_name, def) in &defs {
            if let Some(extensions) = def.get("extensions").and_then(|v| v.as_array()) {
                for ext_val in extensions {
                    let ext = ext_val.as_str().expect("extension must be a string");
                    let result = detect_language_from_extension(ext);
                    assert_eq!(
                        result,
                        Some(lang_name.as_str()),
                        "Extension '{ext}' should map to '{lang_name}' but got {result:?}"
                    );
                }
            }
        }
    }
}
