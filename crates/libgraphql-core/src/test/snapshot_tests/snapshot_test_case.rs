use crate::test::snapshot_tests::utils;
use crate::test::snapshot_tests::ExpectedErrorPattern;
use crate::test::snapshot_tests::OperationSnapshotTestCase;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Represents a complete snapshot test case (schema + operations)
#[derive(Debug, Clone)]
pub struct SchemaSnapshotTestCase {
    pub name: String,
    pub schema_paths: Vec<PathBuf>,
    pub schema_expected_errors: Vec<ExpectedErrorPattern>,
    pub valid_operations: Vec<OperationSnapshotTestCase>,
    pub invalid_operations: Vec<OperationSnapshotTestCase>,
}

impl SchemaSnapshotTestCase {
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
                    eprintln!("ERROR: Unexpected file in valid_schemas/: {}", path.display());
                    eprintln!("       Only directories are allowed in valid_schemas/");
                    eprintln!("       Each directory represents a test suite with schema files.");
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

                if path.is_file() && utils::extension_matches_ignore_case(&path, "graphql") {
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
                    && utils::ends_with_ignore_case(file_name, ".schema.graphql")
                    && !utils::eq_ignore_case(file_name, "schema.graphql")
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

                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if !ext.eq_ignore_ascii_case("graphql") && !ext.eq_ignore_ascii_case("disabled") {
                            eprintln!("ERROR: Unexpected file extension in operations directory: {}", path.display());
                            eprintln!("       Only .graphql and .disabled files are allowed.");
                            return None;
                        }
                    } else {
                        eprintln!("ERROR: File without extension in operations directory: {}", path.display());
                        return None;
                    }

                    if utils::extension_matches_ignore_case(&path, "graphql") {
                        let expected_errors =
                            OperationSnapshotTestCase::parse_expected_errors(&path);

                        return Some(OperationSnapshotTestCase {
                            path,
                            expected_errors,
                        });
                    }
                }
                None
            })
            .collect()
    }
}
