use std::collections::HashSet;
use std::fs;

use crate::caching::main::cache_key::cache_key;
use crate::caching::models::ResultCache;
use crate::caching::tests::test_types::{CacheKeyTestCase, CacheRunTestCase};

const SEEDED: &str = r#"{"a":{"path":"a.sql","value":1},"b":{"path":"b.sql","value":2},"repo":{"path":null,"value":3}}"#;

#[test]
fn given_a_cache_run_when_reopening_then_only_entries_for_unread_or_rewritten_files_remain()
-> Result<(), String> {
    let test_cases = [
        CacheRunTestCase {
            description: "a fresh cache keeps what it stored",
            existing_cache: "",
            enabled: true,
            puts: &[("a", "a.sql", 1), ("b", "b.sql", 2)],
            read_paths: &["a.sql", "b.sql"],
            gets: &[],
            expected_file_rewritten: true,
            expected_reopened: &[("a", Some(1)), ("b", Some(2))],
        },
        CacheRunTestCase {
            description: "an entry read again survives and an unread file keeps its entry",
            existing_cache: SEEDED,
            enabled: true,
            puts: &[],
            read_paths: &["a.sql"],
            gets: &["a"],
            expected_file_rewritten: true,
            expected_reopened: &[("a", Some(1)), ("b", Some(2)), ("repo", None)],
        },
        CacheRunTestCase {
            description: "a read file whose entry was not used is dropped",
            existing_cache: SEEDED,
            enabled: true,
            puts: &[],
            read_paths: &["a.sql", "b.sql"],
            gets: &["b"],
            expected_file_rewritten: true,
            expected_reopened: &[("a", None), ("b", Some(2))],
        },
        CacheRunTestCase {
            description: "a damaged cache file reads as empty",
            existing_cache: "{not json",
            enabled: true,
            puts: &[],
            read_paths: &[],
            gets: &["a"],
            expected_file_rewritten: true,
            expected_reopened: &[("a", None)],
        },
        CacheRunTestCase {
            description: "a disabled cache neither stores nor writes",
            existing_cache: "",
            enabled: false,
            puts: &[("a", "a.sql", 1)],
            read_paths: &["a.sql"],
            gets: &["a"],
            expected_file_rewritten: false,
            expected_reopened: &[("a", None)],
        },
    ];
    for test_case in &test_cases {
        let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
        let cache_file = directory.path().join(".sqlclean/cache/results-v1.json");
        fs::create_dir_all(directory.path().join(".sqlclean/cache"))
            .map_err(|error| error.to_string())?;
        fs::write(&cache_file, test_case.existing_cache).map_err(|error| error.to_string())?;
        let mut cache = ResultCache::open(directory.path(), test_case.enabled);
        for key in test_case.gets {
            let _value: Option<i32> = cache.get(key);
        }
        for (key, path, value) in test_case.puts {
            cache.put(key, Some(path), value);
        }
        let read: HashSet<String> = test_case
            .read_paths
            .iter()
            .map(|path| (*path).to_owned())
            .collect();
        cache.save(&read)?;
        let stored: String = fs::read_to_string(&cache_file).map_err(|error| error.to_string())?;
        assert_eq!(
            stored != test_case.existing_cache,
            test_case.expected_file_rewritten,
            "{}",
            test_case.description
        );
        let mut reopened = ResultCache::open(directory.path(), true);
        for (key, expected) in test_case.expected_reopened {
            assert_eq!(
                reopened.get::<i32>(key),
                *expected,
                "{}: {key}",
                test_case.description
            );
        }
    }
    Ok(())
}

#[test]
fn given_key_parts_when_hashing_then_part_boundaries_change_the_key() {
    let test_cases = [
        CacheKeyTestCase {
            description: "moving a character between parts changes the key",
            left: &["a", "bc"],
            right: &["ab", "c"],
            expected_equal: false,
        },
        CacheKeyTestCase {
            description: "the same parts give the same key",
            left: &["a", "bc"],
            right: &["a", "bc"],
            expected_equal: true,
        },
    ];
    for test_case in &test_cases {
        assert_eq!(
            cache_key(test_case.left) == cache_key(test_case.right),
            test_case.expected_equal,
            "{}",
            test_case.description
        );
    }
}
