use crate::schema::SchemaBuilder;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use super::snapshot_test_case::SnapshotTestCase;

/// Result of a single snapshot test
#[derive(Debug)]
pub struct SnapshotTestResult {
    pub test_name: String,
    pub passed: bool,
    pub error_message: Option<String>,
    pub file_path: PathBuf,
    pub file_snippet: Option<String>,
}

/// Collection of snapshot test results
#[derive(Debug)]
pub struct SnapshotTestResults {
    pub results: Vec<SnapshotTestResult>,
}

impl SnapshotTestResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.passed)
    }

    pub fn add(&mut self, result: SnapshotTestResult) {
        self.results.push(result);
    }

    pub fn failure_report(&self) -> String {
        let failures: Vec<_> = self.results.iter().filter(|r| !r.passed).collect();
        let failures_len = failures.len();
        let results_len = self.results.len();
        let failures_text = failures
            .iter()
            .map(|r| format_detailed_failure(r))
            .collect::<Vec<_>>()
            .join("\n\n");

        format!("{failures_len} of {results_len} snapshot tests failed:\n\n{failures_text}")
    }

    pub fn summary(&self) -> String {
        let all_passed = self.all_passed();
        let emoji = if all_passed { "✅" } else { "❌" };
        let banner = format!("{emoji} ========================================");
        let header = format!("{emoji} GOLDEN TEST SUMMARY");
        let total = self.results.len();

        if all_passed {
            format!("{banner}\n{header}\n{banner}\nTotal tests: {total}\nPassed: {total}\nFailed: 0\n\nAll snapshot tests passed!\n{banner}")
        } else {
            let failures: Vec<_> = self.results.iter().filter(|r| !r.passed).collect();
            let failures_len = failures.len();
            let passed = total - failures_len;
            let failed_list = failures
                .iter()
                .map(|r| {
                    let test_name = &r.test_name;
                    let error = r.error_message.as_deref().unwrap_or("error");
                    format!("  - {test_name} ({error})")
                })
                .collect::<Vec<_>>()
                .join("\n");

            format!("{banner}\n{header}\n{banner}\nTotal tests: {total}\nPassed: {passed}\nFailed: {failures_len}\n\nFailed snapshot tests:\n{failed_list}\n\nSee details above for each failure.\n{banner}")
        }
    }
}

impl std::default::Default for SnapshotTestResults {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a detailed failure message with file path, snippet, and error location
fn format_detailed_failure(result: &SnapshotTestResult) -> String {
    let mut output = String::new();
    let test_name = &result.test_name;
    let file_path = result.file_path.display();

    // Header with test name
    output.push_str(&format!("❌ {test_name}\n"));

    // Absolute file path
    output.push_str(&format!("   File: {file_path}\n"));

    // Error message
    if let Some(msg) = &result.error_message {
        output.push_str(&format!("   {msg}\n"));
    }

    // File snippet with line numbers and error markers
    if let Some(snippet) = &result.file_snippet {
        output.push('\n');
        output.push_str(snippet);
    }

    output
}

/// Run all schema validation snapshot tests
pub fn run_schema_tests(fixtures_dir: &Path) -> SnapshotTestResults {
    let mut results = SnapshotTestResults::new();

    let test_cases = SnapshotTestCase::discover_all(fixtures_dir);

    for test_case in test_cases {
        if test_case.schema_expected_errors.is_empty() {
            results.add(test_valid_schema(&test_case));
        } else {
            results.add(test_invalid_schema(&test_case));
        }
    }

    results
}

/// Test a single valid schema (single or multi-file)
fn test_valid_schema(test_case: &SnapshotTestCase) -> SnapshotTestResult {
    let name = &test_case.name;
    let test_name = format!("{name}/schema");

    // Build schema from all schema files
    let mut builder = SchemaBuilder::new();

    for schema_path in &test_case.schema_paths {
        builder = match builder.load_file(schema_path) {
            Ok(b) => b,
            Err(e) => {
                return SnapshotTestResult {
                    test_name,
                    passed: false,
                    error_message: Some(format!("Expected: Valid schema\nGot: {e:?}")),
                    file_path: schema_path.clone(),
                    file_snippet: None,
                };
            }
        };
    }

    match builder.build() {
        Ok(_) => SnapshotTestResult {
            test_name,
            passed: true,
            error_message: None,
            file_path: test_case.schema_paths[0].clone(),
            file_snippet: None,
        },
        Err(e) => {
            let error_str = format!("{e:?}");
            let file_path = test_case.schema_paths[0].clone();
            let snippet = extract_snippet_with_error_marker(&file_path, 3).ok();

            SnapshotTestResult {
                test_name,
                passed: false,
                error_message: Some(format!("Expected: Valid schema\nGot: {error_str:?}")),
                file_path,
                file_snippet: snippet,
            }
        }
    }
}

/// Test a single invalid schema (single or multi-file)
fn test_invalid_schema(test_case: &SnapshotTestCase) -> SnapshotTestResult {
    let name = &test_case.name;
    let test_name = format!("{name}/schema");

    // Build schema from all schema files
    let mut builder = SchemaBuilder::new();

    for schema_path in &test_case.schema_paths {
        builder = match builder.load_file(schema_path) {
            Ok(b) => b,
            Err(e) => {
                // Early error during loading - check if it matches expected patterns
                let error_str = format!("{e:?}");
                let errors = [error_str.clone()];

                if !test_case.schema_expected_errors.is_empty() {
                    let all_match = test_case
                        .schema_expected_errors
                        .iter()
                        .all(|pattern| errors.iter().any(|e| e.contains(pattern)));

                    if all_match {
                        return SnapshotTestResult {
                            test_name,
                            passed: true,
                            error_message: None,
                            file_path: schema_path.clone(),
                            file_snippet: None,
                        };
                    }
                }

                // Error occurred but didn't match expected
                return SnapshotTestResult {
                    test_name,
                    passed: false,
                    error_message: Some(format!(
                        "Expected: Specific error patterns\nGot: Error occurred but didn't match expected patterns\n\nActual error:\n{error_str}"
                    )),
                    file_path: schema_path.clone(),
                    file_snippet: None,
                };
            }
        };
    }

    match builder.build() {
        Ok(_) => {
            // Schema should have failed but didn't
            let file_path = test_case.schema_paths[0].clone();
            let snippet = create_missing_error_snippet(&file_path).ok();

            SnapshotTestResult {
                test_name,
                passed: false,
                error_message: Some(
                    "Expected: Should fail validation\nGot: Schema built successfully (false negative!)"
                        .to_string(),
                ),
                file_path,
                file_snippet: snippet,
            }
        }
        Err(e) => {
            // Schema failed - check if error matches expected patterns
            let error_str = format!("{e:?}");
            let errors = [error_str.clone()];

            if test_case.schema_expected_errors.is_empty() {
                // No specific expectation - any error is fine
                return SnapshotTestResult {
                    test_name,
                    passed: true,
                    error_message: None,
                    file_path: test_case.schema_paths[0].clone(),
                    file_snippet: None,
                };
            }

            // Check if all expected patterns match
            let all_match = test_case
                .schema_expected_errors
                .iter()
                .all(|pattern| {
                    if pattern == "*" {
                        return true;
                    }
                    errors.iter().any(|e| e.contains(pattern))
                });

            if all_match {
                SnapshotTestResult {
                    test_name,
                    passed: true,
                    error_message: None,
                    file_path: test_case.schema_paths[0].clone(),
                    file_snippet: None,
                }
            } else {
                // Not all patterns matched
                let unmatched: Vec<_> = test_case
                    .schema_expected_errors
                    .iter()
                    .filter(|pattern| !errors.iter().any(|e| e.contains(*pattern)))
                    .collect();
                let unmatched_list = unmatched
                    .iter()
                    .map(|p| format!("  ✗ {p}"))
                    .collect::<Vec<_>>()
                    .join("\n");

                let file_path = test_case.schema_paths[0].clone();
                let snippet = create_missing_error_snippet(&file_path).ok();

                SnapshotTestResult {
                    test_name,
                    passed: false,
                    error_message: Some(format!(
                        "Expected: All error patterns must match\nGot: Not all expected errors matched\n\nUnmatched patterns:\n{unmatched_list}\n\nActual error:\n{error_str}"
                    )),
                    file_path,
                    file_snippet: snippet,
                }
            }
        }
    }
}

/// Extract a code snippet from a file with line numbers and error markers
fn extract_snippet_with_error_marker(
    file_path: &Path,
    context_lines: usize,
) -> Result<String, std::io::Error> {
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();

    let start_line = 0;
    let end_line = context_lines.min(lines.len());

    let mut snippet = String::new();
    let line_num_width = (end_line + 1).to_string().len();

    for (idx, line) in lines[start_line..end_line].iter().enumerate() {
        let line_num = start_line + idx + 1;
        snippet.push_str(&format!("   {line_num:>line_num_width$} │ {line}\n"));
    }

    Ok(snippet)
}

/// Create a snippet showing where an error was expected but didn't occur
fn create_missing_error_snippet(file_path: &Path) -> Result<String, std::io::Error> {
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();

    let mut snippet = String::new();
    snippet.push_str("   Expected errors based on comments:\n");

    for (idx, line) in lines.iter().enumerate().take(10) {
        let line_num = idx + 1;
        if line.trim_start().starts_with("# EXPECTED_ERROR:") {
            snippet.push_str(&format!("   {line_num:>3} → {line} ⚠️ (error not raised!)\n"));
        } else if idx < 5 {
            snippet.push_str(&format!("   {line_num:>3} │ {line}\n"));
        }
    }

    Ok(snippet)
}
