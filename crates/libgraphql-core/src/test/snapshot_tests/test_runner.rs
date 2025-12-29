use crate::ast;
use crate::operation::ExecutableDocumentBuilder;
use crate::operation::FragmentRegistryBuilder;
use crate::schema::Schema;
use crate::schema::SchemaBuilder;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use super::snapshot_test_case::SnapshotTestCase;

#[cfg(test)]

/// Number of context lines to show in schema error snippets
const SCHEMA_ERROR_SNIPPET_LINES: usize = 3;

/// Number of context lines to show in operation error snippets
const OPERATION_ERROR_SNIPPET_LINES: usize = 5;

/// Number of lines to preview when showing expected errors that didn't occur
const EXPECTED_ERROR_PREVIEW_LINES: usize = 10;

/// Format an expected vs actual error message
fn format_expected_actual_error(expected: &str, actual: &str) -> String {
    format!("Expected: {expected}\nGot: {actual}")
}

/// Format a false negative error with expected patterns
fn format_false_negative_error(what: &str, expected_patterns: &[String]) -> String {
    let patterns_text = if expected_patterns.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nExpected error patterns:\n{}",
            expected_patterns
                .iter()
                .map(|p| format!("  - {p}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    format!("Expected: Should fail validation{patterns_text}\nGot: {what} validated successfully (false negative!)")
}

/// Format an unmatched patterns error
fn format_unmatched_patterns_error(unmatched: &[&String], actual_errors: &[String]) -> String {
    format!(
        "Expected: All error patterns must match\nGot: Not all expected errors matched\n\nUnmatched patterns:\n{}\n\nActual errors:\n{}",
        unmatched
            .iter()
            .map(|p| format!("  ✗ {p}"))
            .collect::<Vec<_>>()
            .join("\n"),
        actual_errors
            .iter()
            .map(|e| format!("  - {e}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

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

    pub fn extend(&mut self, results: Vec<SnapshotTestResult>) {
        self.results.extend(results);
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
    let test_cases = SnapshotTestCase::discover_all(fixtures_dir);

    #[cfg(test)]
    let test_results: Vec<SnapshotTestResult> = test_cases
        .par_iter()
        .map(|test_case| {
            if test_case.schema_expected_errors.is_empty() {
                test_valid_schema(test_case)
            } else {
                test_invalid_schema(test_case)
            }
        })
        .collect();

    #[cfg(not(test))]
    let test_results: Vec<SnapshotTestResult> = test_cases
        .iter()
        .map(|test_case| {
            if test_case.schema_expected_errors.is_empty() {
                test_valid_schema(test_case)
            } else {
                test_invalid_schema(test_case)
            }
        })
        .collect();

    let mut results = SnapshotTestResults::new();
    results.extend(test_results);
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
                    error_message: Some(format_expected_actual_error("Valid schema", &format!("{e:?}"))),
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
            let snippet = extract_snippet_with_error_marker(&file_path, SCHEMA_ERROR_SNIPPET_LINES).ok();

            SnapshotTestResult {
                test_name,
                passed: false,
                error_message: Some(format_expected_actual_error("Valid schema", &error_str)),
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
                error_message: Some(format_false_negative_error("Schema", &test_case.schema_expected_errors)),
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
                        "Expected: All error patterns must match\nGot: Not all expected errors matched\n\nUnmatched patterns:\n{}\n\nActual error:\n{error_str}",
                        unmatched.iter().map(|p| format!("  ✗ {p}")).collect::<Vec<_>>().join("\n")
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

    for (idx, line) in lines.iter().enumerate().take(EXPECTED_ERROR_PREVIEW_LINES) {
        let line_num = idx + 1;
        if line.trim_start().starts_with("# EXPECTED_ERROR:") {
            snippet.push_str(&format!("   {line_num:>3} → {line} ⚠️ (error not raised!)\n"));
        } else if idx < 5 {
            snippet.push_str(&format!("   {line_num:>3} │ {line}\n"));
        }
    }

    Ok(snippet)
}

/// Run all operation validation snapshot tests
pub fn run_operation_tests(fixtures_dir: &Path) -> SnapshotTestResults {
    let test_cases = SnapshotTestCase::discover_all(fixtures_dir);

    #[cfg(test)]
    let test_results: Vec<SnapshotTestResult> = test_cases
        .par_iter()
        .filter(|test_case| test_case.schema_expected_errors.is_empty())
        .filter_map(|test_case| {
            // Build the schema first
            let schema = try_build_schema(&test_case.schema_paths)?;

            // Test valid operations
            let mut results = test_valid_operations(test_case, &schema);

            // Test invalid operations
            let invalid_results = test_invalid_operations(test_case, &schema);
            results.extend(invalid_results);

            Some(results)
        })
        .flatten()
        .collect();

    #[cfg(not(test))]
    let test_results: Vec<SnapshotTestResult> = test_cases
        .iter()
        .filter(|test_case| test_case.schema_expected_errors.is_empty())
        .filter_map(|test_case| {
            // Build the schema first
            let schema = try_build_schema(&test_case.schema_paths)?;

            // Test valid operations
            let mut results = test_valid_operations(test_case, &schema);

            // Test invalid operations
            let invalid_results = test_invalid_operations(test_case, &schema);
            results.extend(invalid_results);

            Some(results)
        })
        .flatten()
        .collect();

    let mut results = SnapshotTestResults::new();
    results.extend(test_results);
    results
}

/// Helper to try building a schema from multiple files
fn try_build_schema(schema_paths: &[PathBuf]) -> Option<Schema> {
    let mut builder = SchemaBuilder::new();

    for schema_path in schema_paths {
        builder = match builder.load_file(schema_path) {
            Ok(b) => b,
            Err(_) => return None,
        };
    }

    builder.build().ok()
}

/// Test valid operations against a schema
fn test_valid_operations(test_case: &SnapshotTestCase, schema: &Schema) -> Vec<SnapshotTestResult> {
    let mut results = Vec::new();

    if test_case.valid_operations.is_empty() {
        return results;
    }

    // Test each valid operation with its own fragment registry
    for op_test in &test_case.valid_operations {
        let test_name = format!(
            "{}/valid_operations/{}",
            test_case.name,
            op_test.path.file_name().unwrap().to_str().unwrap()
        );

        // Build fragment registry from just this operation file
        let fragment_registry = match build_fragment_registry(schema, &[&op_test.path]) {
            Ok(reg) => reg,
            Err(err) => {
                results.push(SnapshotTestResult {
                    test_name,
                    passed: false,
                    error_message: Some(format!("Failed to build fragment registry: {err:?}")),
                    file_path: op_test.path.clone(),
                    file_snippet: None,
                });
                continue;
            }
        };

        let exec_doc_result =
            ExecutableDocumentBuilder::from_file(schema, &fragment_registry, &op_test.path);

        match exec_doc_result {
            Ok(_) => {
                results.push(SnapshotTestResult {
                    test_name,
                    passed: true,
                    error_message: None,
                    file_path: op_test.path.clone(),
                    file_snippet: None,
                });
            }
            Err(errors) => {
                let error_str = format!("{errors:?}");
                let snippet = extract_snippet_with_error_marker(&op_test.path, OPERATION_ERROR_SNIPPET_LINES).ok();

                results.push(SnapshotTestResult {
                    test_name,
                    passed: false,
                    error_message: Some(format_expected_actual_error("Valid operation", &error_str)),
                    file_path: op_test.path.clone(),
                    file_snippet: snippet,
                });
            }
        }
    }

    results
}

/// Test invalid operations against a schema
fn test_invalid_operations(
    test_case: &SnapshotTestCase,
    schema: &Schema,
) -> Vec<SnapshotTestResult> {
    let mut results = Vec::new();

    if test_case.invalid_operations.is_empty() {
        return results;
    }

    // Test each invalid operation with its own fragment registry
    for op_test in &test_case.invalid_operations {
        let test_name = format!(
            "{}/invalid_operations/{}",
            test_case.name,
            op_test.path.file_name().unwrap().to_str().unwrap()
        );

        // Try to build fragment registry from this operation file
        let fragment_registry = match build_fragment_registry(schema, &[&op_test.path]) {
            Ok(reg) => reg,
            Err(err) => {
                // Fragment registry build failed - use empty registry
                eprintln!("Warning: Failed to build fragment registry for invalid operation: {err:?}");
                match FragmentRegistryBuilder::new().build() {
                    Ok(reg) => reg,
                    Err(empty_err) => {
                        results.push(SnapshotTestResult {
                            test_name,
                            passed: false,
                            error_message: Some(
                                format!("Failed to create empty fragment registry: {empty_err:?}"),
                            ),
                            file_path: op_test.path.clone(),
                            file_snippet: None,
                        });
                        continue;
                    }
                }
            }
        };

        let exec_doc_result =
            ExecutableDocumentBuilder::from_file(schema, &fragment_registry, &op_test.path);

        match exec_doc_result {
            Ok(_) => {
                // Operation should have failed but didn't
                let snippet = create_missing_error_snippet(&op_test.path).ok();

                results.push(SnapshotTestResult {
                    test_name,
                    passed: false,
                    error_message: Some(format_false_negative_error("Operation", &op_test.expected_errors)),
                    file_path: op_test.path.clone(),
                    file_snippet: snippet,
                });
            }
            Err(errors) => {
                // Operation failed - check if expected errors match
                let error_strs: Vec<String> = errors.iter().map(|e| format!("{e:?}")).collect();

                if op_test.all_expected_errors_match(&error_strs) {
                    results.push(SnapshotTestResult {
                        test_name,
                        passed: true,
                        error_message: None,
                        file_path: op_test.path.clone(),
                        file_snippet: None,
                    });
                } else {
                    // Errors don't match expected
                    let unmatched: Vec<_> = op_test
                        .expected_errors
                        .iter()
                        .filter(|pattern| !error_strs.iter().any(|e| e.contains(*pattern)))
                        .collect();

                    let snippet = create_missing_error_snippet(&op_test.path).ok();

                    results.push(SnapshotTestResult {
                        test_name,
                        passed: false,
                        error_message: Some(format!(
                            "Expected: All error patterns must match\nGot: Not all expected errors matched\n\nUnmatched patterns:\n{}\n\nActual errors:\n{error_strs:?}",
                            unmatched.iter().map(|p| format!("  ✗ {p}")).collect::<Vec<_>>().join("\n")
                        )),
                        file_path: op_test.path.clone(),
                        file_snippet: snippet,
                    });
                }
            }
        }
    }

    results
}

/// Build FragmentRegistry from a collection of operation files
fn build_fragment_registry<'schema>(
    schema: &'schema Schema,
    operation_files: &[&PathBuf],
) -> Result<crate::operation::FragmentRegistry<'schema>, String> {
    let mut registry_builder = FragmentRegistryBuilder::new();

    for file_path in operation_files {
        // Read the file content
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;

        // Parse as AST
        let ast_doc = graphql_parser::query::parse_query::<String>(&content)
            .map_err(|e| format!("Failed to parse GraphQL in {}: {}", file_path.display(), e))?
            .into_static();

        // Add fragments from this document
        registry_builder
            .add_from_document_ast(
                schema,
                &ast::operation::Document::from(ast_doc),
                Some(file_path),
            )
            .map_err(|e| {
                format!(
                    "Failed to add fragments from {}: {:?}",
                    file_path.display(),
                    e
                )
            })?;
    }

    registry_builder
        .build()
        .map_err(|e| format!("Failed to build registry: {e:?}"))
}
