use std::{path::{Path, PathBuf}, sync::OnceLock};

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

pub fn get_fixtures_dir() -> &'static Path {
    static FIXTURES_DIR: OnceLock<PathBuf> = OnceLock::new();
    FIXTURES_DIR.get_or_init(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/test/snapshot_tests/fixtures")
    })
}
