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
        {
            let kind = node.kind();
            if kind != "string" && kind != "comment" {
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
}
