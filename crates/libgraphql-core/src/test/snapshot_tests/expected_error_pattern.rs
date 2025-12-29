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

