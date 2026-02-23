/// Configuration for [`GraphQLParser`](crate::GraphQLParser)
/// controlling parser behavior.
///
/// All flags default to their full-fidelity values. Set individual
/// flags to `false` to discard specific elements for leaner output.
///
/// # Example
///
/// ```rust
/// use libgraphql_parser::GraphQLParserConfig;
///
/// // Full-fidelity (default)
/// let full = GraphQLParserConfig::default();
/// assert!(full.retain_syntax);
///
/// // Lean mode: skip populating syntax structs
/// let lean = GraphQLParserConfig::lean();
/// assert!(!lean.retain_syntax);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLParserConfig {
    /// Whether the parser should populate `*Syntax` structs on AST
    /// nodes with the concrete tokens that make up each construct
    /// (punctuation, keywords, etc.).
    ///
    /// When `false`, all `syntax` fields on AST nodes remain `None`,
    /// saving allocations when only semantic data is needed.
    pub retain_syntax: bool,
}

impl GraphQLParserConfig {
    /// Returns a lean config that skips populating syntax structs.
    /// Useful when only semantic AST data is needed.
    pub fn lean() -> Self {
        Self {
            retain_syntax: false,
        }
    }
}

impl Default for GraphQLParserConfig {
    fn default() -> Self {
        Self {
            retain_syntax: true,
        }
    }
}
