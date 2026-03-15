//! Result type for parsing operations that may produce partial results.

use crate::ast;
use crate::token::GraphQLTriviaToken;
use crate::ByteSpan;
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
/// (stored in [`ByteSpan`](crate::ByteSpan)s on AST nodes and tokens)
/// to line/column positions on demand, and provides access to the
/// source text via [`source()`](SourceMap::source). Parse errors carry
/// pre-resolved [`SourceSpan`](crate::SourceSpan)s and do not need the
/// `SourceMap` for position resolution.
///
/// # Accessing the AST
///
/// - [`recovered()`](Self::recovered) - Returns the error-recovered AST, a
///   `&[GraphQLParseError]`, and a [`SourceMap`] if there were 1 or more errors
///   parsing the AST. The AST is "recovered" AST in the sense that the parser
///   made a best-effort attempt at either guessing the intended AST or skipping
///   any portion of the input for which it could not make a reasonable guess.
///
/// - [`valid()`](Self::valid) — Returns the AST and [`SourceMap`] only if
///   parsing was completely successful (no errors). Use this when you need
///   guaranteed-valid input.
///
/// - [`into_recovered`](Self::into_recovered) - Consuming version of
///   [`recovered`](Self::recovered).
///
/// - [`into_valid()`](Self::into_valid) — Consuming version of
///   [`valid()`](Self::valid).
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
/// # use libgraphql_parser::SourceMap;
/// #
/// # let source = "type Query { foo: String }";
/// # let parser = GraphQLParser::new(source);
/// #
/// # fn analyze_schema(schema: &ast::Document<'_>, source_map: &SourceMap::<'_>) { }
/// # fn provide_ide_completions(schema: &ast::Document) { }
/// #
/// let result = parser.parse_schema_document();
///
/// // Strict mode: only accept fully valid documents
/// if let Some((doc, source_map)) = result.valid() {
///     analyze_schema(doc, source_map);
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
    pub fn ast(&self) -> &TAst {
        match self {
            Self::Ok { ast, .. } => ast,
            Self::Recovered { ast, .. } => ast,
        }
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
    pub fn formatted_errors(&self) -> String {
        let source = self.source_map().source();
        self.errors()
            .iter()
            .map(|e| e.format_detailed(source))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns `true` if any errors were encountered during parsing.
    pub fn has_errors(&self) -> bool {
        matches!(self, Self::Recovered { .. })
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

    pub fn into_recovered(self) -> Option<(TAst, Vec<GraphQLParseError>, SourceMap<'src>)> {
        match self {
            Self::Ok { .. } => None,
            Self::Recovered { ast, errors, source_map } => Some((ast, errors, source_map)),
        }
    }

    /// Takes ownership of the AST only if parsing was completely successful.
    ///
    /// This is the consuming version of [`valid()`](Self::valid).
    pub fn into_valid(self) -> Option<(TAst, SourceMap<'src>)> {
        match self {
            Self::Ok { ast, source_map } => Some((ast, source_map)),
            Self::Recovered { .. } => None,
        }
    }

    /// Creates a successful parse result with no errors.
    pub(crate) fn new_ok(ast: TAst, source_map: SourceMap<'src>) -> Self {
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
    pub(crate) fn new_recovered(
        ast: TAst,
        errors: Vec<GraphQLParseError>,
        source_map: SourceMap<'src>,
    ) -> Self {
        debug_assert!(
            !errors.is_empty(),
            "ParseResult::new_recovered() called with empty errors vec; \
             use ParseResult::new_ok() instead",
        );
        Self::Recovered { ast, errors, source_map }
    }

    pub fn recovered(&self) -> Option<(&TAst, &[GraphQLParseError], &SourceMap<'src>)> {
        match self {
            Self::Ok { .. } => None,
            Self::Recovered { ast, errors, source_map } => Some((ast, errors, source_map)),
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

    /// Returns the AST & [`SourceMap`] only if parsing was completely
    /// successful (no errors).
    ///
    /// Use this when you need guaranteed-valid input, such as when compiling
    /// a schema or executing a query.
    ///
    /// Returns `None` if parsing succeeded but with errors (recovered AST).
    pub fn valid(&self) -> Option<(&TAst, &SourceMap<'src>)> {
        match self {
            Self::Ok { ast, source_map } => Some((ast, source_map)),
            Self::Recovered { .. } => None,
        }
    }
}

impl<'src> ParseResult<'src, ast::Document<'src>> {
    /// Convenience function for accessing the
    /// [`Document::definitions`](ast::Document::definitions) field after
    /// parsing a [`Document`](ast::Document).
    pub fn definitions(&self) -> &[ast::Definition<'src>] {
        &self.ast().definitions
    }

    /// Convenience function for calling
    /// [`Document::executable_definitions()`](ast::Document::executable_definitions)
    /// after parsing a [`Document`](ast::Document).
    pub fn executable_definitions(&self) -> impl Iterator<Item = &ast::Definition<'src>> {
        self.ast().executable_definitions()
    }

    /// Convenience function for calling
    /// [`Document::schema_definitions()`](ast::Document::schema_definitions)
    /// after parsing a [`Document`](ast::Document).
    pub fn schema_definitions(&self) -> impl Iterator<Item = &ast::Definition<'src>> {
        self.ast().schema_definitions()
    }

    /// Convenience function for accessing the
    /// [`Document::span`](ast::Document::span) field after parsing
    /// a [`Document`](ast::Document).
    pub fn span(&self) -> ByteSpan {
        self.ast().span
    }

    /// Convenience function for accessing the
    /// [`Document::syntax`](ast::Document::syntax) field after parsing
    /// a [`Document`](ast::Document).
    pub fn syntax(&self) -> &Option<Box<ast::DocumentSyntax<'src>>> {
        &self.ast().syntax
    }

    /// Convenience function for calling
    /// [`Document::trailing_trivia()`](ast::Document::trailing_trivia) after
    /// parsing a [`Document`](ast::Document).
    pub fn trailing_trivia(&self) -> Option<&Vec<GraphQLTriviaToken<'src>>> {
        self.ast().trailing_trivia()
    }
}

impl<'src, TAst> From<ParseResult<'src, TAst>>
    for Result<(TAst, SourceMap<'src>), Vec<GraphQLParseError>> {
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

