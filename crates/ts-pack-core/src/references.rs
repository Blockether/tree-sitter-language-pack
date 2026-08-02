//! Identifier occurrence scanning: every leaf token whose text equals a wanted name.
//!
//! The whole batch is served by ONE parse and ONE tree walk, so tracing a file for
//! a thousand identifiers costs the same walk as tracing it for one.

use ahash::AHashMap;

use crate::Error;
use crate::intel::types::{ReferenceHit, Span};

/// Every occurrence of each identifier in `names`, in source order.
///
/// A match is a leaf token whose text equals one of the names, so occurrences sit
/// at real token boundaries: never inside a larger identifier, and never inside a
/// string or comment token. There is no scope resolution — a shadowed or unrelated
/// same-named identifier is reported too.
///
/// One parse and one tree walk serve the entire batch: cost is `O(nodes + hits)`
/// and barely moves with the number of names, where calling this once per name
/// would re-parse the file every time. Blank names and duplicates are ignored; an
/// empty (or all-blank) `names` yields an empty result.
///
/// # Errors
///
/// Returns [`Error::LanguageNotFound`] if the language is unknown, or
/// [`Error::ParseFailed`] if the source cannot be parsed.
///
/// # Example
///
/// ```no_run
/// use tree_sitter_language_pack::find_references;
///
/// let hits = find_references("def hello():\n    hello()\n", "python", &["hello"])?;
/// assert_eq!(hits.len(), 2);
/// assert_eq!(hits[0].span.start_line, 0);
/// # Ok::<(), tree_sitter_language_pack::Error>(())
/// ```
pub fn find_references(source: &str, language: &str, names: &[&str]) -> Result<Vec<ReferenceHit>, Error> {
    // Needle table keyed by raw bytes: the walk never has to decode a token to test it.
    let needles: AHashMap<&[u8], &str> = names
        .iter()
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .map(|name| (name.as_bytes(), name))
        .collect();
    if needles.is_empty() {
        return Ok(Vec::new());
    }
    // Byte-length sieve: `sieve[len]` is false for a length no name has, which rejects
    // nearly every leaf token before it is hashed or compared.
    let max_len = needles.keys().map(|needle| needle.len()).max().unwrap_or(0);
    let mut sieve = vec![false; max_len + 1];
    for needle in needles.keys() {
        sieve[needle.len()] = true;
    }

    let language_handle = crate::get_language(language)?;
    let bytes = source.as_bytes();
    let tree = crate::parse::parse_with_language(language, &language_handle, bytes)?;

    let mut hits = Vec::new();
    let mut cursor = tree.walk();
    // Iterative depth-first walk: descend while there are children, and every node
    // that has none is a leaf token — the granularity at which identifiers live.
    'walk: loop {
        if cursor.goto_first_child() {
            continue;
        }
        let node = cursor.node();
        // `get`, never an index: an out-of-range span — a grammar reporting a range
        // past the end of the source — must not panic here. This runs behind an
        // `extern "C"` boundary, where an unwind aborts the entire host process and
        // takes every other session running in it down too.
        if let Some(text) = bytes.get(node.start_byte()..node.end_byte())
            && let Some(name) = sieved_lookup(&needles, &sieve, text)
            && !is_literal_text(&node)
        {
            let start_point = node.start_position();
            let end_point = node.end_position();
            hits.push(ReferenceHit {
                name: name.to_owned(),
                span: Span {
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    start_line: start_point.row,
                    start_column: start_point.column,
                    end_line: end_point.row,
                    end_column: end_point.column,
                },
            });
        }
        loop {
            if cursor.goto_next_sibling() {
                continue 'walk;
            }
            if !cursor.goto_parent() {
                break 'walk;
            }
        }
    }
    Ok(hits)
}

/// Needle lookup behind the byte-length sieve: a length no name has is rejected
/// before the token is ever hashed, which is almost every leaf in a real file.
#[inline]
fn sieved_lookup<'a>(needles: &AHashMap<&'a [u8], &'a str>, sieve: &[bool], text: &[u8]) -> Option<&'a str> {
    if text.len() < sieve.len() && sieve[text.len()] {
        needles.get(text).copied()
    } else {
        None
    }
}

/// Is this leaf the TEXT of a string or a comment, rather than code?
///
/// Grammars disagree about where that text lives. Some keep the whole literal in
/// one token (`comment`, `string_literal`), and those never match a bare
/// identifier anyway because the quotes and the `#` ride along. Most split the
/// body out into its own leaf — `string_content` (Python, Ruby, Rust, C, Bash,
/// Kotlin, Lua, PHP), `string_fragment` (JavaScript, TypeScript, Java),
/// `interpreted_string_literal_content` (Go), `comment_content` (Lua),
/// `doc_comment` (Rust) — and THAT leaf is exactly `alpha` for `"alpha"`. Testing
/// only `kind == "string"` let every one of those through, so `s = "alpha"`
/// counted as a reference to `alpha` in most of the pack while Clojure, whose
/// string is one token, correctly reported none.
///
/// Swift names its body `line_str_text`, saying nothing about strings, so the
/// parent kind is consulted too. Interpolations survive both rules: `f"{alpha}"`
/// and `${alpha}` put the identifier under an `interpolation` /
/// `template_substitution` node, and where a grammar hangs it straight off the
/// string (Kotlin's `interpolated_identifier`) the identifier-ish kind wins — an
/// interpolated name IS a real reference.
fn is_literal_text(node: &tree_sitter::Node) -> bool {
    let kind = node.kind();
    if is_literal_kind(kind) {
        return true;
    }
    if is_identifier_kind(kind) {
        return false;
    }
    node.parent().is_some_and(|parent| is_literal_kind(parent.kind()))
}

/// Kinds that name string or comment TEXT. Substrings, because the pack carries
/// 300+ third-party grammars and each spells its own variant (`raw_string_literal`,
/// `line_comment`, `heredoc_body`, `line_str_text`, …).
#[inline]
fn is_literal_kind(kind: &str) -> bool {
    kind.contains("string") || kind.contains("comment") || kind.contains("heredoc") || kind.contains("str_text")
}

/// Kinds that name an identifier, checked only to keep an interpolated name that
/// a grammar parents directly to its string literal.
#[inline]
fn is_identifier_kind(kind: &str) -> bool {
    kind.contains("identifier") || kind.ends_with("_name") || kind == "name" || kind == "word"
}

#[cfg(test)]
mod tests {
    use super::*;

    const PY: &str = "def alpha(beta):\n    return alpha(beta) + beta\n";

    fn python_available() -> bool {
        crate::has_language("python")
    }

    #[test]
    fn finds_every_occurrence_in_source_order() {
        if !python_available() {
            return;
        }
        let hits = find_references(PY, "python", &["alpha"]).unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|hit| hit.name == "alpha"));
        assert!(hits[0].span.start_byte < hits[1].span.start_byte);
        assert_eq!(hits[0].span.start_line, 0);
        assert_eq!(hits[0].span.start_column, 4);
        assert_eq!(hits[1].span.start_line, 1);
    }

    #[test]
    fn batch_equals_the_union_of_single_name_scans() {
        if !python_available() {
            return;
        }
        let batch = find_references(PY, "python", &["alpha", "beta"]).unwrap();
        let mut singles: Vec<ReferenceHit> = ["alpha", "beta"]
            .iter()
            .flat_map(|name| find_references(PY, "python", &[name]).unwrap())
            .collect();
        singles.sort_by_key(|hit| hit.span.start_byte);
        assert_eq!(batch, singles);
    }

    #[test]
    fn never_matches_inside_a_longer_token() {
        if !python_available() {
            return;
        }
        let hits = find_references("alphabet = 1\nalpha = 2\n", "python", &["alpha"]).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].span.start_byte, 13);
    }

    #[test]
    fn blank_and_duplicate_names_are_ignored() {
        if !python_available() {
            return;
        }
        assert!(find_references(PY, "python", &[]).unwrap().is_empty());
        assert!(find_references(PY, "python", &["   "]).unwrap().is_empty());
        let once = find_references(PY, "python", &["alpha"]).unwrap();
        let twice = find_references(PY, "python", &["alpha", "alpha", " alpha "]).unwrap();
        assert_eq!(once, twice);
    }

    #[test]
    fn a_name_that_never_occurs_yields_no_hits() {
        if !python_available() {
            return;
        }
        assert!(find_references(PY, "python", &["nowhere"]).unwrap().is_empty());
    }

    #[test]
    fn unknown_language_is_an_error() {
        assert!(find_references(PY, "nonexistent_xyz", &["alpha"]).is_err());
    }

    /// Many threads, one library. A shared host runs every session against this
    /// single process, so the batch scan has to be callable from all of them at
    /// once: threads here race over sources that also push the process-wide tree
    /// cache into eviction, and each must still get exactly what one thread alone
    /// computes.
    #[test]
    fn concurrent_callers_each_get_the_single_threaded_result() {
        if !python_available() {
            return;
        }
        const THREADS: usize = 16;
        const ROUNDS: usize = 40;
        // Distinct sources so the threads evict each other's cache entries, walked
        // at staggered offsets so they also collide on the SAME entry.
        let sources: Vec<String> = (0..24)
            .map(|i| format!("def alpha{i}(beta):\n    return alpha(beta) + beta{i}\n"))
            .collect();
        let names = ["alpha", "beta", "gamma"];
        let expected: Vec<Vec<ReferenceHit>> = sources
            .iter()
            .map(|source| find_references(source, "python", &names).unwrap())
            .collect();
        assert!(expected.iter().all(|hits| !hits.is_empty()));

        std::thread::scope(|scope| {
            for thread in 0..THREADS {
                let sources = &sources;
                let expected = &expected;
                scope.spawn(move || {
                    for round in 0..ROUNDS {
                        let which = (thread + round) % sources.len();
                        let hits = find_references(&sources[which], "python", &names).unwrap();
                        assert_eq!(hits, expected[which], "thread {thread}, round {round}");
                    }
                });
            }
        });
    }
    /// Every grammar spells a string body its own way, and testing `kind != "string"`
    /// only ever caught the one that spells it exactly that. A name inside a literal
    /// is TEXT in all of them, never a reference — `s = "alpha"` used to count.
    #[test]
    fn a_name_inside_a_string_or_comment_is_never_a_reference() {
        let cases: &[(&str, &str)] = &[
            ("python", "alpha = 1\ns = \"alpha\"\n# alpha\n"),
            ("javascript", "const alpha = 1;\nconst s = \"alpha\";\n// alpha\n"),
            (
                "typescript",
                "const alpha: number = 1;\nconst s = \"alpha\";\n// alpha\n",
            ),
            (
                "rust",
                "fn alpha() {}\nconst S: &str = \"alpha\";\n// alpha\n/// alpha\nfn b() {}\n",
            ),
            ("go", "package p\nfunc alpha() {}\nvar s = \"alpha\"\n// alpha\n"),
            ("java", "class C { int alpha = 1;\nString s = \"alpha\";\n// alpha\n}\n"),
            ("ruby", "alpha = 1\ns = \"alpha\"\n# alpha\n"),
            ("c", "int alpha = 1;\nchar *s = \"alpha\";\n// alpha\n"),
            ("bash", "alpha=1\ns=\"alpha\"\n# alpha\n"),
            ("swift", "let alpha = 1\nlet s = \"alpha\"\n// alpha\n"),
            ("kotlin", "val alpha = 1\nval s = \"alpha\"\n// alpha\n"),
            ("lua", "local alpha = 1\nlocal s = \"alpha\"\n-- alpha\n"),
            ("php", "<?php $alpha = 1; $s = \"alpha\"; // alpha\n"),
            ("clojure", "(def alpha 1)\n(def s \"alpha\")\n;; alpha\n"),
        ];
        let mut checked = 0;
        for (language, source) in cases {
            if !crate::has_language(language) {
                continue;
            }
            let hits = find_references(source, language, &["alpha"]).unwrap();
            assert_eq!(
                hits.len(),
                1,
                "{language}: only the declaration is a reference, got {hits:?}"
            );
            assert_eq!(
                &source[hits[0].span.start_byte..hits[0].span.end_byte],
                "alpha",
                "{language}: the hit has to span the declaration itself"
            );
            checked += 1;
        }
        assert!(checked > 0, "no language of the case table was available");
    }

    /// An interpolated identifier is code that happens to sit inside a literal, so
    /// it stays a reference — including in Kotlin, which hangs it straight off the
    /// string with no interpolation node in between.
    #[test]
    fn an_interpolated_identifier_is_still_a_reference() {
        let cases: &[(&str, &str)] = &[
            ("python", "def f(alpha):\n    return f\"{alpha}\"\n"),
            ("javascript", "const t = (alpha) => `${alpha}`;\n"),
            ("ruby", "alpha = 1\nt = \"#{alpha}\"\n"),
            ("bash", "alpha=1\nt=\"$alpha\"\n"),
            ("swift", "let alpha = 1\nlet t = \"\\(alpha)\"\n"),
            ("kotlin", "val alpha = 1\nval t = \"$alpha\"\n"),
        ];
        let mut checked = 0;
        for (language, source) in cases {
            if !crate::has_language(language) {
                continue;
            }
            let hits = find_references(source, language, &["alpha"]).unwrap();
            assert_eq!(
                hits.len(),
                2,
                "{language}: declaration plus the interpolated use, got {hits:?}"
            );
            checked += 1;
        }
        assert!(checked > 0, "no language of the case table was available");
    }
}
