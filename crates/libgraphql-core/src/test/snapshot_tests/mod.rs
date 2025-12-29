//! Snapshot testing framework for validating GraphQL schemas and operations.
//!
//! This module provides a file-based testing system that validates libgraphql
//! against known-good and known-invalid GraphQL schemas and operations.
//!
//! For detailed usage instructions, see the [README](snapshot_tests/fixtures/README.md).

mod snapshot_test_case;
mod test_runner;

#[cfg(test)]
mod tests {
    use super::test_runner;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    fn get_fixtures_dir() -> &'static Path {
        static FIXTURES_DIR: OnceLock<PathBuf> = OnceLock::new();
        FIXTURES_DIR.get_or_init(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/test/snapshot_tests/fixtures")
        })
    }

    #[test]
    fn verify_schema_snapshot_tests() {
        let fixtures_dir = get_fixtures_dir();

        let results = test_runner::run_schema_tests(fixtures_dir);

        let all_passed = results.all_passed();
        if !all_passed {
            eprintln!("{}", results.failure_report());
            eprintln!("\n{}", results.summary());
        } else {
            println!("{}", results.summary());
        }

        assert!(
            all_passed,
            "Schema snapshot tests failed:\n{}",
            results.failure_report()
        );
    }

    #[test]
    fn verify_operation_snapshot_tests() {
        let fixtures_dir = get_fixtures_dir();
        let results = test_runner::run_operation_tests(&fixtures_dir);

        if !results.all_passed() {
            eprintln!("{}", results.failure_report());
            eprintln!("\n{}", results.summary());
        } else {
            println!("{}", results.summary());
        }

        assert!(
            results.all_passed(),
            "Operation snapshot tests failed:\n{}",
            results.failure_report()
        );
    }
}
