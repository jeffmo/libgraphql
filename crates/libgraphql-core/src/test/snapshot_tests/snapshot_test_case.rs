use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Pattern for matching expected errors in snapshot tests.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpectedErrorPattern {
    /// Exact error type match (e.g., `# EXPECTED_ERROR_TYPE: SelectionSetBuildError`)
    ExactType(String),
    /// Substring match (e.g., `# EXPECTED_ERROR_CONTAINS: undefined field`)
    Contains(String),
}

impl std::fmt::Display for ExpectedErrorPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedErrorPattern::ExactType(type_name) => {
                write!(f, "ERROR_TYPE: {type_name}")
            }
            ExpectedErrorPattern::Contains(substring) => {
                write!(f, "ERROR_CONTAINS: {substring}")
            }
        }
    }
}

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
        let Ok(content) = fs::read_to_string(path) else {
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
                } else if let Some(contains_pattern) =
                    trimmed.strip_prefix("# EXPECTED_ERROR_CONTAINS:")
                {
                    Some(ExpectedErrorPattern::Contains(
                        contains_pattern.trim().to_string(),
                    ))
                } else {
                    None
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

/// Represents a complete snapshot test case (schema + operations)
#[derive(Debug, Clone)]
pub struct SnapshotTestCase {
    pub name: String,
    pub schema_paths: Vec<PathBuf>,
    pub schema_expected_errors: Vec<ExpectedErrorPattern>,
    pub valid_operations: Vec<OperationSnapshotTestCase>,
    pub invalid_operations: Vec<OperationSnapshotTestCase>,
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
                let valid_operations = Self::discover_operations(&path.join("valid_operations"));
                let invalid_operations =
                    Self::discover_operations(&path.join("invalid_operations"));

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

    /// Discovers operation files in a directory
    fn discover_operations(dir: &Path) -> Vec<OperationSnapshotTestCase> {
        if !dir.exists() {
            return Vec::new();
        }

        let Ok(entries) = fs::read_dir(dir) else {
            return Vec::new();
        };

        entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();

                if path.is_file() && path.extension()? == "graphql" {
                    let expected_errors =
                        OperationSnapshotTestCase::parse_expected_errors(&path);

                    Some(OperationSnapshotTestCase {
                        path,
                        expected_errors,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
