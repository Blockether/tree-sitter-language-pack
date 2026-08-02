use std::cell::RefCell;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::sync::{LazyLock, Mutex};

use ahash::{AHashMap, AHasher};

use crate::Error;

thread_local! {
    static PARSER_CACHE: RefCell<AHashMap<String, tree_sitter::Parser>> = RefCell::new(AHashMap::new());
}

const TREE_CACHE_MAX_ENTRIES: usize = 256;
const TREE_CACHE_MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;

struct CachedTree {
    language: Box<str>,
    source: Box<[u8]>,
    tree: tree_sitter::Tree,
}

#[derive(Default)]
struct TreeCache {
    entries: AHashMap<u64, CachedTree>,
    order: VecDeque<u64>,
    source_bytes: usize,
}

static TREE_CACHE: LazyLock<Mutex<TreeCache>> = LazyLock::new(|| Mutex::new(TreeCache::default()));

impl TreeCache {
    /// Drop every entry, restoring the invariant between `entries`, `order` and
    /// `source_bytes`. Used to recover a cache a panicking caller left mid-update.
    fn reset(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.source_bytes = 0;
    }
}

/// Take the process-wide cache lock, surviving poisoning.
///
/// One process serves many concurrent callers — a long-lived host runs every
/// session against this single library — so one panicking parse must not become a
/// permanent, process-wide outage where every later call fails with
/// `LockPoisoned`. The panic can only have interrupted the cache's own
/// bookkeeping, which is pure derived state: clear it, clear the poison, carry on.
fn lock_tree_cache() -> std::sync::MutexGuard<'static, TreeCache> {
    match TREE_CACHE.lock() {
        Ok(cache) => cache,
        Err(poisoned) => {
            let mut cache = poisoned.into_inner();
            cache.reset();
            TREE_CACHE.clear_poison();
            cache
        }
    }
}

#[inline]
fn tree_cache_key(language: &str, source: &[u8]) -> u64 {
    let mut hasher = AHasher::default();
    language.hash(&mut hasher);
    source.hash(&mut hasher);
    hasher.finish()
}

/// Parse source code with a pre-loaded `Language`, reusing both parsers and exact
/// recently parsed trees.
///
/// Trees are immutable and keyed by language plus the complete source bytes, so a
/// cache hit is content-addressed and cannot return stale syntax after an edit.
/// The bounded process-wide tree cache is also the serialization point required by
/// third-party scanners with process-global mutable state. Parser instances remain
/// thread-local because tree-sitter parsers themselves are not shared between threads.
pub(crate) fn parse_with_language(
    language_name: &str,
    language: &tree_sitter::Language,
    source: &[u8],
) -> Result<tree_sitter::Tree, Error> {
    let key = tree_cache_key(language_name, source);
    let mut trees = lock_tree_cache();
    if let Some(cached) = trees.entries.get(&key)
        && cached.language.as_ref() == language_name
        && cached.source.as_ref() == source
    {
        return Ok(cached.tree.clone());
    }

    let tree = PARSER_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(parser) = cache.get_mut(language_name) {
            return parser.parse(source, None).ok_or(Error::ParseFailed);
        }
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(language)
            .map_err(|e| Error::ParserSetup(format!("{e}")))?;
        let tree = parser.parse(source, None).ok_or(Error::ParseFailed)?;
        cache.insert(language_name.to_string(), parser);
        Ok(tree)
    })?;

    if source.len() <= TREE_CACHE_MAX_SOURCE_BYTES {
        if let Some(collided) = trees.entries.remove(&key) {
            trees.source_bytes = trees.source_bytes.saturating_sub(collided.source.len());
            trees.order.retain(|cached_key| *cached_key != key);
        }
        while trees.entries.len() >= TREE_CACHE_MAX_ENTRIES
            || trees.source_bytes.saturating_add(source.len()) > TREE_CACHE_MAX_SOURCE_BYTES
        {
            if let Some(oldest_key) = trees.order.pop_front() {
                if let Some(oldest) = trees.entries.remove(&oldest_key) {
                    trees.source_bytes = trees.source_bytes.saturating_sub(oldest.source.len());
                }
            } else {
                break;
            }
        }
        trees.source_bytes += source.len();
        trees.order.push_back(key);
        trees.entries.insert(
            key,
            CachedTree {
                language: language_name.into(),
                source: source.into(),
                tree: tree.clone(),
            },
        );
    }

    Ok(tree)
}

#[cfg(test)]
pub(crate) fn cached_parser_count_for_tests() -> usize {
    PARSER_CACHE.with(|cache| cache.borrow().len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skip_if_no_languages() -> bool {
        crate::available_languages().is_empty()
    }

    fn parse_for_test(language_name: &str, source: &[u8]) -> Result<tree_sitter::Tree, Error> {
        let language = crate::get_language(language_name)?;
        parse_with_language(language_name, &language, source)
    }

    #[test]
    fn test_parse_with_language_success() {
        if skip_if_no_languages() {
            return;
        }
        let langs = crate::available_languages();
        let first = &langs[0];
        let tree = parse_for_test(first, b"x");
        assert!(tree.is_ok(), "parse_with_language should succeed for '{first}'");
    }

    #[test]
    fn test_get_language_invalid_language() {
        let result = crate::get_language("nonexistent_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_language_reuses_cache() {
        if skip_if_no_languages() {
            return;
        }
        let langs = crate::available_languages();
        let first = &langs[0];
        let lang = crate::get_language(first).unwrap();
        let _ = parse_with_language(first, &lang, b"parser_cache_test_one").unwrap();
        let after_first = cached_parser_count_for_tests();
        let _ = parse_with_language(first, &lang, b"parser_cache_test_two").unwrap();
        let after_second = cached_parser_count_for_tests();
        assert_eq!(after_first, after_second, "second call should reuse cached parser");
    }

    #[test]
    fn test_different_languages_get_separate_cache_entries() {
        let langs = crate::available_languages();
        if langs.len() < 2 {
            return;
        }
        let before = cached_parser_count_for_tests();
        let _ = parse_for_test(&langs[0], b"x").unwrap();
        let _ = parse_for_test(&langs[1], b"x").unwrap();
        let after = cached_parser_count_for_tests();
        assert!(
            after >= before + 2,
            "different languages should create separate cache entries"
        );
    }
    #[test]
    fn exact_source_hits_tree_cache_and_edited_source_does_not() {
        if skip_if_no_languages() {
            return;
        }
        let langs = crate::available_languages();
        let first = &langs[0];
        let lang = crate::get_language(first).unwrap();
        let source = b"tree_cache_exact_source";
        let edited = b"tree_cache_edited_source";

        let first_tree = parse_with_language(first, &lang, source).unwrap();
        let entries_after_first = TREE_CACHE.lock().unwrap().entries.len();
        let second_tree = parse_with_language(first, &lang, source).unwrap();
        let entries_after_exact_hit = TREE_CACHE.lock().unwrap().entries.len();
        let edited_tree = parse_with_language(first, &lang, edited).unwrap();
        let entries_after_edit = TREE_CACHE.lock().unwrap().entries.len();

        assert_eq!(first_tree.root_node().to_sexp(), second_tree.root_node().to_sexp());
        assert_eq!(entries_after_first, entries_after_exact_hit);
        assert!(entries_after_edit >= entries_after_exact_hit);
        assert_ne!(tree_cache_key(first, source), tree_cache_key(first, edited));
        assert_ne!(second_tree.root_node().end_byte(), edited_tree.root_node().end_byte());
    }

    /// One caller dying under the lock must not take every other caller with it:
    /// a poisoned mutex would fail EVERY later parse in the process for good.
    #[test]
    fn a_panic_under_the_lock_does_not_break_later_callers() {
        if skip_if_no_languages() {
            return;
        }
        let died = std::thread::spawn(|| {
            let _guard = lock_tree_cache();
            panic!("deliberate panic while holding the tree cache");
        })
        .join();
        assert!(died.is_err(), "the helper thread was supposed to panic");

        let langs = crate::available_languages();
        let first = &langs[0];
        let lang = crate::get_language(first).unwrap();
        assert!(
            parse_with_language(first, &lang, b"parse_after_a_poisoned_lock").is_ok(),
            "a poisoned lock must not outlive the caller that poisoned it"
        );
        let cache = lock_tree_cache();
        assert_eq!(
            cache.entries.len(),
            cache.order.len(),
            "recovery must leave the cache's bookkeeping consistent"
        );
    }
}
