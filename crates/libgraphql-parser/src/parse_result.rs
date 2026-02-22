//! Result type for parsing operations that may produce partial results.

use crate::GraphQLParseError;

/// The result of a parsing operation.
///
/// Unlike `Result<T, E>`, `ParseResult` can contain both a partial AST and
/// errors. This enables error recovery: the parser can report multiple errors
/// while still producing as much AST as possible.
///
/// # Design Rationale
///
/// Traditional `Result<T, E>` forces a binary choice: either success with a
/// value, or failure with an error. But GraphQL tooling often benefits from
/// having both:
///
/// - **IDE integration**: Show syntax errors while still providing completions
///   based on the partially-parsed document
/// - **Batch error reporting**: Report all syntax errors in one pass rather
///   than stopping at the first error
/// - **Graceful degradation**: Process as much of a document as possible even
///   when parts are invalid
///
/// # Accessing the AST
///
/// Two methods are provided for accessing the AST, depending on your use case:
///
/// - [`valid_ast()`](Self::valid_ast) - Returns the AST only if parsing was
///   completely successful (no errors). Use this when you need guaranteed-valid
///   input.
///
/// - [`ast()`](Self::ast) - Returns the AST if present, regardless of errors.
///   Use this for tools that want best-effort results (formatters, IDE
///   features, linters).
///
/// # Example
///
/// ```
/// # use libgraphql_parser::ast;
/// # use libgraphql_parser::GraphQLParser;
/// #
/// # let source = "type Query { foo: String }";
/// # let parser = GraphQLParser::new(source);
/// #
/// # fn analyze_schema(schema: &ast::Document) { }
/// # fn provide_ide_completions(schema: &ast::Document) { }
/// #
/// let result = parser.parse_schema_document();
///
/// // Strict mode: only accept fully valid documents
/// if let Some(doc) = result.valid_ast() {
///     analyze_schema(doc);
/// }
///
/// // Best-effort mode: work with whatever we got
/// if let Some(doc) = result.ast() {
///     provide_ide_completions(doc);
/// }
///
/// // Report any errors
/// if result.has_errors() {
///     for error in &result.errors {
///         eprintln!("{}", error.format_detailed(Some(source)));
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ParseResult<TAst> {
    /// The parsed AST, if parsing produced any result.
    ///
    /// This may be `Some` even when `errors` is non-empty, representing a
    /// recovered/partial parse result.
    ast: Option<TAst>,

    /// Errors encountered during parsing.
    ///
    /// Empty if parsing was completely successful.
    pub errors: Vec<GraphQLParseError>,
}

impl<TAst> ParseResult<TAst> {
    /// Creates a successful parse result with no errors.
    pub(crate) fn ok(ast: TAst) -> Self {
        Self {
            ast: Some(ast),
            errors: Vec::new(),
        }
    }

    /// Creates a failed parse result with errors but no AST.
    #[cfg(test)]
    pub(crate) fn err(errors: Vec<GraphQLParseError>) -> Self {
        Self { ast: None, errors }
    }

    /// Creates a recovered parse result with both AST and errors.
    ///
    /// The AST was produced via error recovery and may be incomplete or contain
    /// placeholder values.
    pub(crate) fn recovered(ast: TAst, errors: Vec<GraphQLParseError>) -> Self {
        Self {
            ast: Some(ast),
            errors,
        }
    }

    /// Returns the AST only if parsing was completely successful (no errors).
    ///
    /// Use this when you need guaranteed-valid input, such as when compiling
    /// a schema or executing a query.
    ///
    /// Returns `None` if:
    /// - Parsing failed entirely (no AST produced)
    /// - Parsing succeeded but with errors (recovered AST)
    pub fn valid_ast(&self) -> Option<&TAst> {
        if self.errors.is_empty() {
            self.ast.as_ref()
        } else {
            None
        }
    }

    /// Returns the AST if present, regardless of whether errors occurred.
    ///
    /// Use this for tools that want best-effort results:
    /// - IDE features (completions, hover info)
    /// - Formatters (format what we can parse)
    /// - Linters (report issues even in partially-valid documents)
    ///
    /// Check [`has_errors()`](Self::has_errors) to determine if the AST was
    /// produced via error recovery.
    pub fn ast(&self) -> Option<&TAst> {
        self.ast.as_ref()
    }

    /// Takes ownership of the AST only if parsing was completely successful.
    ///
    /// This is the consuming version of [`valid_ast()`](Self::valid_ast).
    pub fn into_valid_ast(self) -> Option<TAst> {
        if self.errors.is_empty() {
            self.ast
        } else {
            None
        }
    }

    /// Takes ownership of the AST regardless of errors.
    ///
    /// This is the consuming version of [`ast()`](Self::ast).
    pub fn into_ast(self) -> Option<TAst> {
        self.ast
    }

    /// Returns `true` if parsing was completely successful (has AST, no
    /// errors).
    pub fn is_ok(&self) -> bool {
        self.ast.is_some() && self.errors.is_empty()
    }

    /// Returns `true` if any errors were encountered during parsing.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Formats all errors as a single string for display.
    ///
    /// # Arguments
    /// - `source`: Optional source text for snippet extraction in error
    ///   messages.
    pub fn format_errors(&self, source: Option<&str>) -> String {
        self.errors
            .iter()
            .map(|e| e.format_detailed(source))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl<TAst> From<ParseResult<TAst>> for Result<TAst, Vec<GraphQLParseError>> {
    /// Converts to a standard `Result`, treating recovered ASTs as errors.
    ///
    /// Returns `Ok(ast)` only if there were no errors. Otherwise returns
    /// `Err(errors)`, even if a recovered AST was available.
    fn from(result: ParseResult<TAst>) -> Self {
        if result.errors.is_empty() {
            match result.ast {
                Some(ast) => Ok(ast),
                None => Err(Vec::new()),
            }
        } else {
            Err(result.errors)
        }
    }
}
