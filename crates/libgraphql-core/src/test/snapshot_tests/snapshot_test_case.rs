use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Represents a single operation test case (valid or invalid)
#[derive(Debug, Clone)]
pub struct OperationSnapshotTestCase {
    pub path: PathBuf,
    pub should_be_valid: bool,
    pub expected_errors: Vec<String>,
}

impl OperationSnapshotTestCase {
    /// Parses EXPECTED_ERROR comments from a GraphQL file
    pub fn parse_expected_errors(path: &Path) -> Vec<String> {
        let Ok(content) = fs::read_to_string(path) else {
            return Vec::new();
        };

        content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim_start();
                if trimmed.starts_with("# EXPECTED_ERROR:") {
                    Some(
                        trimmed
                            .strip_prefix("# EXPECTED_ERROR:")
                            .unwrap()
                            .trim()
                            .to_string(),
                    )
                } else {
                    None
                }
            })
            .collect()
    }

    /// Checks if all expected error patterns match the actual errors
    pub fn all_expected_errors_match(&self, actual_errors: &[String]) -> bool {
        if self.expected_errors.is_empty() {
            // No specific expectation - just verify that some error occurred
            return !actual_errors.is_empty();
        }

        // All expected patterns must have at least one matching actual error
        self.expected_errors.iter().all(|expected_pattern| {
            if expected_pattern == "*" {
                return true; // Wildcard always matches
            }
            // Check if any actual error contains this expected pattern
            actual_errors
                .iter()
                .any(|actual_error| actual_error.contains(expected_pattern))
        })
    }
}

/// Represents a complete snapshot test case (schema + operations)
#[derive(Debug, Clone)]
pub struct SnapshotTestCase {
    pub name: String,
    pub schema_paths: Vec<PathBuf>,
    pub schema_expected_errors: Vec<String>,
    pub valid_operations: Vec<GoldenOperationTestCase>,
    pub invalid_operations: Vec<GoldenOperationTestCase>,
}

impl SnapshotTestCase {
    /// Discovers all snapshot test cases from the fixtures directory
    pub fn discover_all(fixtures_dir: &Path) -> Vec<Self> {
        let mut cases = Vec::new();

        // Discover valid schema test suites
        cases.extend(Self::discover_valid_schemas(fixtures_dir));

        // Discover invalid schema tests
        cases.extend(Self::discover_invalid_schemas(fixtures_dir));

        cases
    }

    /// Discovers valid schema test suites
    fn discover_valid_schemas(fixtures_dir: &Path) -> Vec<Self> {
        let valid_schemas_dir = fixtures_dir.join("valid_schemas");
        if !valid_schemas_dir.exists() {
            return Vec::new();
        }

        let Ok(entries) = fs::read_dir(&valid_schemas_dir) else {
            return Vec::new();
        };

        entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();

                if !path.is_dir() {
                    return None;
                }

                let suite_name = path.file_name()?.to_str()?.to_string();

                // Discover schema files
                let schema_paths = Self::discover_schema_files(&path)?;

                // Parse expected errors from all schema files
                let schema_expected_errors = schema_paths
                    .iter()
                    .flat_map(|p| OperationSnapshotTestCase::parse_expected_errors(p))
                    .collect();

                // Discover operations
                let (valid_operations, invalid_operations) = Self::discover_operations(&path);

                Some(Self {
                    name: suite_name,
                    schema_paths,
                    schema_expected_errors,
                    valid_operations,
                    invalid_operations,
                })
            })
            .collect()
    }

    /// Discovers invalid schema tests
    fn discover_invalid_schemas(fixtures_dir: &Path) -> Vec<Self> {
        let invalid_schemas_dir = fixtures_dir.join("invalid_schemas");
        if !invalid_schemas_dir.exists() {
            return Vec::new();
        }

        let Ok(entries) = fs::read_dir(&invalid_schemas_dir) else {
            return Vec::new();
        };

        entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();

                if path.is_file() && path.extension()? == "graphql" {
                    // Single-file invalid schema
                    let name = path.file_stem()?.to_str()?.to_string();
                    let expected_errors =
                        OperationSnapshotTestCase::parse_expected_errors(&path);

                    Some(Self {
                        name,
                        schema_paths: vec![path],
                        schema_expected_errors: expected_errors,
                        valid_operations: Vec::new(),
                        invalid_operations: Vec::new(),
                    })
                } else if path.is_dir() {
                    // Multi-file invalid schema
                    let name = path.file_name()?.to_str()?.to_string();
                    let schema_paths = Self::discover_schema_files(&path)?;
                    let schema_expected_errors = schema_paths
                        .iter()
                        .flat_map(|p| OperationSnapshotTestCase::parse_expected_errors(p))
                        .collect();

                    Some(Self {
                        name,
                        schema_paths,
                        schema_expected_errors,
                        valid_operations: Vec::new(),
                        invalid_operations: Vec::new(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Discovers schema files in a directory
    /// Returns schema.graphql AND any *.schema.graphql files
    fn discover_schema_files(dir: &Path) -> Option<Vec<PathBuf>> {
        let mut schema_files = Vec::new();

        // Check for single schema.graphql file
        let single_schema = dir.join("schema.graphql");
        if single_schema.exists() {
            schema_files.push(single_schema);
        }

        // Look for *.schema.graphql files
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file()
                    && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                    && file_name.ends_with(".schema.graphql")
                    && file_name != "schema.graphql"
                {
                    schema_files.push(path);
                }
            }
        }

        if schema_files.is_empty() {
            None
        } else {
            Some(schema_files)
        }
    }

    /// Discovers operation files in a test suite directory
    /// Returns (valid_operations, invalid_operations)
    fn discover_operations(suite_dir: &Path) -> (Vec<GoldenOperationTestCase>, Vec<GoldenOperationTestCase>) {
        let mut valid_operations = Vec::new();
        let mut invalid_operations = Vec::new();

        // Discover valid operations
        let valid_ops_dir = suite_dir.join("valid_operations");
        if valid_ops_dir.exists() {
            if let Ok(entries) = fs::read_dir(&valid_ops_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && path.extension().and_then(|s| s.to_str()) == Some("graphql")
                        && !path.file_name().and_then(|s| s.to_str()).map_or(false, |s| s.ends_with(".disabled"))
                    {
                        valid_operations.push(GoldenOperationTestCase {
                            path,
                            should_be_valid: true,
                            expected_errors: Vec::new(),
                        });
                    }
                }
            }
        }

        // Discover invalid operations
        let invalid_ops_dir = suite_dir.join("invalid_operations");
        if invalid_ops_dir.exists() {
            if let Ok(entries) = fs::read_dir(&invalid_ops_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && path.extension().and_then(|s| s.to_str()) == Some("graphql")
                        && !path.file_name().and_then(|s| s.to_str()).map_or(false, |s| s.ends_with(".disabled"))
                    {
                        let expected_errors = GoldenOperationTestCase::parse_expected_errors(&path);
                        invalid_operations.push(GoldenOperationTestCase {
                            path,
                            should_be_valid: false,
                            expected_errors,
                        });
                    }
                }
            }
        }

        (valid_operations, invalid_operations)
    }

}
