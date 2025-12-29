use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::test::snapshot_tests::ExpectedErrorPattern;

/// Check if a path's extension matches the given extension (case-insensitive)
pub fn extension_matches_ignore_case(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case(ext))
        .unwrap_or(false)
}

/// Check if a string ends with a suffix (case-insensitive)
pub fn ends_with_ignore_case(s: &str, suffix: &str) -> bool {
    s.len() >= suffix.len() && s[s.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}

/// Check if two strings are equal (case-insensitive)
pub fn eq_ignore_case(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Check if an error string matches an expected error pattern
pub fn error_matches_pattern(error: &str, pattern: &ExpectedErrorPattern) -> bool {
    match pattern {
        ExpectedErrorPattern::ExactType(type_name) => {
            // Match if error Debug output contains the exact type name
            error.contains(type_name)
        }
        ExpectedErrorPattern::Contains(substring) => {
            // Case-sensitive substring match
            error.contains(substring)
        }
    }
}

pub fn get_fixtures_dir() -> &'static Path {
    static FIXTURES_DIR: OnceLock<PathBuf> = OnceLock::new();
    FIXTURES_DIR.get_or_init(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/test/snapshot_tests/fixtures")
    })
}
