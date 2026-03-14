//! Result type for parsing operations that may produce partial results.

use crate::GraphQLParseError;
use crate::SourceMap;

/// The result of a parsing operation.
///
/// Unlike `Result<T, E>`, `ParseResult` can represent a *recovered* parse:
/// one that produced an AST **and** encountered errors. This enables error
/// recovery — the parser can report multiple errors while still producing
/// as much AST as possible.
///
/// # Variants
///
/// | Variant     | AST | Errors | Meaning                                  |
/// |-------------|-----|--------|------------------------------------------|
/// | `Ok`        | yes | no     | Completely successful parse               |
/// | `Recovered` | yes | yes    | Best-effort AST with errors encountered   |
///
/// An AST is **always** present — there is no "complete failure" variant.
/// Even when the parser encounters errors, it produces a partial/recovered
/// AST via error recovery.
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
/// # SourceMap
///
/// Every `ParseResult` carries a [`SourceMap`] that maps byte offsets
/// (stored in [`ByteSpan`](crate::ByteSpan)s on AST nodes, tokens, and
/// errors) to line/column positions on demand. This avoids eagerly computing
/// positions during parsing while ensuring they are always recoverable.
///
/// # Accessing the AST
///
/// - [`valid_ast()`](Self::valid_ast) — Returns the AST only if parsing was
///   completely successful (no errors). Use this when you need guaranteed-valid
///   input.
///
/// - [`into_valid_ast()`](Self::into_valid_ast) — Consuming version of
///   `valid_ast()`.
///
/// - [`into_ast()`](Self::into_ast) — Extracts the AST unconditionally,
///   consuming the `ParseResult`. Use this for tools that want best-effort
///   results (formatters, IDE features, linters).
///
/// - **Pattern matching** — For borrowing the AST unconditionally while
///   retaining access to errors, match on the enum variants directly.
///
/// # Example
///
/// ```
/// # use libgraphql_parser::ast;
/// # use libgraphql_parser::GraphQLParser;
/// # use libgraphql_parser::ParseResult;
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
/// // Best-effort mode: match to borrow the AST unconditionally
/// let ast = match &result {
///     ParseResult::Ok { ast, .. }
///     | ParseResult::Recovered { ast, .. } => ast,
/// };
/// provide_ide_completions(ast);
///
/// // Report any errors
/// if result.has_errors() {
///     for error in result.errors() {
///         eprintln!("{}", error.format_detailed(result.source_map().source()));
///     }
/// }
/// ```
#[derive(Debug)]
pub enum ParseResult<'src, TAst> {
    /// Completely successful parse — the AST is valid and no errors were
    /// encountered.
    Ok {
        /// The parsed AST.
        ast: TAst,

        /// Maps byte offsets to line/column positions.
        source_map: SourceMap<'src>,
    },

    /// Recovered parse — an AST was produced via error recovery, but errors
    /// were encountered. The AST may be incomplete or contain placeholder
    /// values.
    ///
    /// Invariant: `errors` is always non-empty for this variant.
    Recovered {
        /// The recovered AST.
        ast: TAst,

        /// Errors encountered during parsing (always non-empty).
        errors: Vec<GraphQLParseError>,

        /// Maps byte offsets to line/column positions.
        source_map: SourceMap<'src>,
    },
}

impl<'src, TAst> ParseResult<'src, TAst> {
    /// Creates a successful parse result with no errors.
    pub(crate) fn ok(ast: TAst, source_map: SourceMap<'src>) -> Self {
        Self::Ok { ast, source_map }
    }

    /// Creates a recovered parse result with both AST and errors.
    ///
    /// The AST was produced via error recovery and may be incomplete or
    /// contain placeholder values.
    ///
    /// # Panics (debug only)
    ///
    /// Debug-asserts that `errors` is non-empty.
    pub(crate) fn recovered(
        ast: TAst,
        errors: Vec<GraphQLParseError>,
        source_map: SourceMap<'src>,
    ) -> Self {
        debug_assert!(
            !errors.is_empty(),
            "ParseResult::recovered() called with empty errors vec; \
             use ParseResult::ok() instead",
        );
        Self::Recovered { ast, errors, source_map }
    }

    /// Returns the AST only if parsing was completely successful (no errors).
    ///
    /// Use this when you need guaranteed-valid input, such as when compiling
    /// a schema or executing a query.
    ///
    /// Returns `None` if parsing succeeded but with errors (recovered AST).
    pub fn valid_ast(&self) -> Option<&TAst> {
        match self {
            Self::Ok { ast, .. } => Some(ast),
            Self::Recovered { .. } => None,
        }
    }

    /// Takes ownership of the AST only if parsing was completely successful.
    ///
    /// This is the consuming version of [`valid_ast()`](Self::valid_ast).
    pub fn into_valid_ast(self) -> Option<TAst> {
        match self {
            Self::Ok { ast, .. } => Some(ast),
            Self::Recovered { .. } => None,
        }
    }

    /// Takes ownership of the AST unconditionally.
    ///
    /// An AST is always present in a `ParseResult`, even when parsing errors
    /// may have occurred. Use this for tools that want best-effort results:
    /// - IDE features (completions, hover info)
    /// - Formatters (format what we can parse)
    /// - Linters (report issues even in partially-valid documents)
    ///
    /// Check [`has_errors()`](Self::has_errors) before calling if you need
    /// to know whether the AST was produced via error recovery.
    pub fn into_ast(self) -> TAst {
        match self {
            Self::Ok { ast, .. } => ast,
            Self::Recovered { ast, .. } => ast,
        }
    }

    /// Returns a reference to the [`SourceMap`] for resolving byte offsets
    /// to line/column positions.
    pub fn source_map(&self) -> &SourceMap<'src> {
        match self {
            Self::Ok { source_map, .. } => source_map,
            Self::Recovered { source_map, .. } => source_map,
        }
    }

    /// Returns `true` if any errors were encountered during parsing.
    pub fn has_errors(&self) -> bool {
        matches!(self, Self::Recovered { .. })
    }

    /// Returns the errors encountered during parsing.
    ///
    /// Returns an empty slice for `Ok`, or the non-empty error list for
    /// `Recovered`.
    pub fn errors(&self) -> &[GraphQLParseError] {
        match self {
            Self::Ok { .. } => &[],
            Self::Recovered { errors, .. } => errors,
        }
    }

    /// Formats all errors as a single string for display.
    ///
    /// Uses the bundled `SourceMap`'s source text (if available)
    /// for snippet extraction.
    pub fn format_errors(&self) -> String {
        let source = self.source_map().source();
        self.errors()
            .iter()
            .map(|e| e.format_detailed(source))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl<'src, TAst> From<ParseResult<'src, TAst>>
    for Result<(TAst, SourceMap<'src>), Vec<GraphQLParseError>>
{
    /// Converts to a standard `Result`, treating recovered ASTs as errors.
    ///
    /// Returns `Ok((ast, source_map))` only if there were no errors.
    /// Otherwise returns `Err(errors)`, discarding the recovered AST.
    fn from(result: ParseResult<'src, TAst>) -> Self {
        match result {
            ParseResult::Ok { ast, source_map } => Ok((ast, source_map)),
            ParseResult::Recovered { errors, .. } => Err(errors),
        }
    }
}
