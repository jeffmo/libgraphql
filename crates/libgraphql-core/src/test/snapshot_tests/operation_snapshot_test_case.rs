use crate::test::snapshot_tests::ExpectedErrorPattern;
use std::path::Path;
use std::path::PathBuf;

/// Represents a single operation test case (valid or invalid)
#[derive(Debug, Clone)]
pub struct OperationSnapshotTestCase {
    pub path: PathBuf,
    pub expected_errors: Vec<ExpectedErrorPattern>,
}

impl OperationSnapshotTestCase {
    /// Parses EXPECTED_ERROR_TYPE and EXPECTED_ERROR_CONTAINS comments from a GraphQL file.
    ///
    /// Supports two syntaxes:
    /// - `# EXPECTED_ERROR_TYPE: TypeName` - Matches exact error type
    /// - `# EXPECTED_ERROR_CONTAINS: text` - Matches substring in error message
    pub fn parse_expected_errors(path: &Path) -> Vec<ExpectedErrorPattern> {
        let Ok(content) = std::fs::read_to_string(path) else {
            return Vec::new();
        };

        content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim_start();

                if let Some(type_pattern) = trimmed.strip_prefix("# EXPECTED_ERROR_TYPE:") {
                    Some(ExpectedErrorPattern::ExactType(
                        type_pattern.trim().to_string(),
                    ))
                } else {
                    trimmed.strip_prefix("# EXPECTED_ERROR_CONTAINS:").map(|contains_pattern| {
                        ExpectedErrorPattern::Contains(contains_pattern.trim().to_string())
                    })
                }
            })
            .collect()
    }

    /// Checks if all expected error patterns match the actual errors.
    ///
    /// Used for validating that invalid operations/schemas fail as expected.
    ///
    /// Returns true if either:
    /// - No expected error patterns are specified and at least one error occurred
    ///   (validates that something failed, even if we don't care what specific error)
    /// - All expected patterns have at least one matching error (case-sensitive)
    pub fn all_expected_errors_match(&self, actual_errors: &[String]) -> bool {
        if self.expected_errors.is_empty() {
            // No specific expectation - just verify that some error occurred
            return !actual_errors.is_empty();
        }

        // All expected patterns must have at least one matching actual error
        self.expected_errors.iter().all(|expected_pattern| {
            actual_errors.iter().any(|actual_error| match expected_pattern {
                ExpectedErrorPattern::ExactType(type_name) => {
                    // Match if error Debug output contains the exact type name
                    // Case-sensitive match
                    actual_error.contains(type_name)
                }
                ExpectedErrorPattern::Contains(substring) => {
                    // Case-sensitive substring match
                    actual_error.contains(substring)
                }
            })
        })
    }
}

