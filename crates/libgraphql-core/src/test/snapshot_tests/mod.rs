//! Snapshot testing framework for validating GraphQL schemas and operations.
//!
//! This module provides a file-based testing system that validates libgraphql
//! against known-good and known-invalid GraphQL schemas and operations.
//!
//! For detailed usage instructions, see the [README](snapshot_tests/fixtures/README.md).

mod expected_error_pattern;
mod operation_snapshot_test_case;
mod snapshot_test_case;
mod test_runner;
mod utils;

pub use expected_error_pattern::ExpectedErrorPattern;
pub use operation_snapshot_test_case::OperationSnapshotTestCase;

#[cfg(test)]
mod tests {
    use crate::test::snapshot_tests::test_runner;
    use crate::test::snapshot_tests::utils;

    #[test]
    fn verify_schema_snapshot_tests() {
        let fixtures_dir = utils::get_fixtures_dir();

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
        let fixtures_dir = utils::get_fixtures_dir();
        let results = test_runner::run_operation_tests(fixtures_dir);

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
