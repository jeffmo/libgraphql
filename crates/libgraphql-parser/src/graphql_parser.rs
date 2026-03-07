//! Recursive descent parser for GraphQL documents.
//!
//! This module provides [`GraphQLParser`], a generic parser that works with any
//! token source implementing [`GraphQLTokenSource`]. It supports parsing schema
//! documents, executable documents, and mixed documents.
//!
//! # Architecture
//!
//! The parser uses recursive descent with a delimiter stack for error recovery.
//! Most grammar rules have a corresponding `parse_*` method that returns
//! `Result<AstNode, ()>`, where `Err(())` indicates a parse error was recorded
//! and the caller should attempt recovery.
//!
//! # Error Recovery
//!
//! When an error is encountered:
//! 1. An error is recorded via `record_error()`
//! 2. The method returns `Err(())`
//! 3. The caller can attempt recovery (e.g., skip to next definition)
//!
//! This allows collecting multiple errors in a single parse pass.

use crate::ast;
use crate::DefinitionKind;
use crate::DocumentKind;
use crate::GraphQLParseError;
use crate::GraphQLParseErrorKind;
use crate::GraphQLParserConfig;
use crate::GraphQLSourceSpan;
use crate::GraphQLTokenStream;
use crate::ParseResult;
use crate::ReservedNameContext;
use crate::SourcePosition;
use crate::ValueParsingError;
use crate::token::GraphQLToken;
use crate::token::GraphQLTokenKind;
use crate::token_source::GraphQLTokenSource;
use crate::token_source::StrGraphQLTokenSource;
use smallvec::SmallVec;
use std::borrow::Cow;

// =============================================================================
// Delimiter tracking for error recovery
// =============================================================================

/// Context in which a delimiter was opened, for error messages.
#[derive(Debug, Clone, Copy)]
enum DelimiterContext {
    /// `schema { ... }`
    SchemaDefinition,
    /// `type Foo { ... }` (object type definitions)
    ObjectTypeDefinition,
    /// `interface Foo { ... }`
    InterfaceDefinition,
    /// `enum Foo { ... }`
    EnumDefinition,
    /// `input Foo { ... }`
    InputObjectDefinition,
    /// `{ field ... }` in operations/fragments
    SelectionSet,
    /// `(arg: value)` in field arguments
    FieldArguments,
    /// `@directive(arg: value)` in directive arguments
    DirectiveArguments,
    /// `($var: Type)` in operation variable definitions
    VariableDefinitions,
    /// `[Type]` in type annotations
    ListType,
    /// `[value, ...]` in list literals
    ListValue,
    /// `{ field: value }` in object literals
    ObjectValue,
    /// `(name: Type)` in field/directive argument definitions
    ArgumentDefinitions,
}

impl DelimiterContext {
    /// Returns a human-readable description of this context.
    fn description(&self) -> &'static str {
        match self {
            DelimiterContext::SchemaDefinition => "schema definition",
            DelimiterContext::ObjectTypeDefinition => "object type definition",
            DelimiterContext::InterfaceDefinition => "interface definition",
            DelimiterContext::EnumDefinition => "enum definition",
            DelimiterContext::InputObjectDefinition => "input object definition",
            DelimiterContext::SelectionSet => "selection set",
            DelimiterContext::FieldArguments => "field arguments",
            DelimiterContext::DirectiveArguments => "directive arguments",
            DelimiterContext::VariableDefinitions => "variable definitions",
            DelimiterContext::ListType => "list type annotation",
            DelimiterContext::ListValue => "list value",
            DelimiterContext::ObjectValue => "object value",
            DelimiterContext::ArgumentDefinitions => "argument definitions",
        }
    }
}

/// Tracks an open delimiter for error recovery.
#[derive(Debug, Clone)]
struct OpenDelimiter {
    /// Where the delimiter was opened
    span: GraphQLSourceSpan,
    /// The parsing context (also implicitly identifies the delimiter type)
    context: DelimiterContext,
}

/// Internal enum for recovery actions, used to avoid borrow conflicts.
enum RecoveryAction {
    /// Stop recovery, we found a valid definition start.
    Stop,
    /// Skip this token and continue looking.
    Skip,
    /// Check if this keyword starts a definition.
    CheckKeyword(String),
    /// Check if this string is a description before a definition.
    CheckDescription,
}

/// Context for parsing values, determining whether variables are allowed.
///
/// This enum replaces a simple `bool` to provide context-specific error
/// messages when variables appear in const-only contexts.
#[derive(Clone, Copy, Debug)]
enum ConstContext {
    /// Variables are allowed (e.g., field arguments in operations).
    AllowVariables,
    /// Parsing a default value for a variable definition.
    VariableDefaultValue,
    /// Parsing a directive argument in a const context.
    DirectiveArgument,
    /// Parsing a default value for an input field or argument definition.
    InputDefaultValue,
}

impl ConstContext {
    /// Returns a human-readable description for error messages.
    ///
    /// Only called when variables are disallowed, so `AllowVariables` is
    /// unreachable.
    fn description(&self) -> &'static str {
        match self {
            ConstContext::AllowVariables => {
                unreachable!("description() called on AllowVariables")
            }
            ConstContext::VariableDefaultValue => "variable default values",
            ConstContext::DirectiveArgument => "directive arguments",
            ConstContext::InputDefaultValue => "input field default values",
        }
    }
}

// =============================================================================
// Implements clause (syntax-carrying return type)
// =============================================================================

/// Intermediate result from `parse_ast_implements_interfaces`,
/// carrying the parsed interfaces and the tokens that make up
/// the clause so callers can populate their syntax structs.
struct ImplementsClause<'src> {
    ampersands: Vec<GraphQLToken<'src>>,
    implements_keyword: GraphQLToken<'src>,
    interfaces: Vec<ast::Name<'src>>,
    leading_ampersand: Option<GraphQLToken<'src>>,
}

/// Token-only portion of an `ImplementsClause`, separated from
/// the `interfaces` vec so the caller can move `interfaces`
/// into the AST node and keep the tokens for syntax population.
struct ImplementsClauseTokens<'src> {
    ampersands: Vec<GraphQLToken<'src>>,
    implements_keyword: GraphQLToken<'src>,
    leading_ampersand: Option<GraphQLToken<'src>>,
}

// =============================================================================
// Main parser struct
// =============================================================================

/// A recursive descent parser for GraphQL documents.
///
/// Generic over the token source, enabling parsing from both string input
/// (`StrGraphQLTokenSource`) and proc-macro input
/// (`RustMacroGraphQLTokenSource`).
///
/// # Usage
///
/// ```
/// use libgraphql_parser::ast;
/// use libgraphql_parser::GraphQLParser;
///
/// let source = "type Query { hello: String }";
/// let parser = GraphQLParser::new(source);
/// let result = parser.parse_schema_document();
///
/// assert!(!result.has_errors());
/// if let Some(doc) = result.valid_ast() {
///     assert!(matches!(
///         doc.definitions[0],
///         ast::Definition::TypeDefinition(_),
///     ));
/// }
/// ```
pub struct GraphQLParser<'src, TTokenSource: GraphQLTokenSource<'src>> {
    /// Parser configuration controlling behavior such as syntax
    /// struct population.
    config: GraphQLParserConfig,

    /// The underlying token stream with lookahead support.
    token_stream: GraphQLTokenStream<'src, TTokenSource>,

    /// Accumulated parse errors.
    errors: Vec<GraphQLParseError>,

    /// Stack of open delimiters for error recovery.
    ///
    /// Uses SmallVec to avoid heap allocation for typical nesting depths
    /// (most GraphQL documents nest fewer than 8 delimiters deep).
    delimiter_stack: SmallVec<[OpenDelimiter; 8]>,

    /// Current nesting depth for recursive value parsing.
    ///
    /// Shared recursion depth counter, incremented on entry to
    /// `parse_value`, `parse_selection_set`,
    /// `parse_type_annotation`;
    /// decremented on exit. Prevents stack overflow from deeply
    /// nested constructs (e.g., `[[[...` values,
    /// `{ f { f { ...` selection sets, `[[[String]]]` types).
    recursion_depth: usize,

    /// End position of the most recently consumed token, used by
    /// `eof_span()` to anchor EOF errors to the last known source
    /// location.
    last_end_position: Option<SourcePosition>,
}

impl<'src> GraphQLParser<'src, StrGraphQLTokenSource<'src>> {
    /// Creates a new parser from a string-like source.
    ///
    /// Accepts any type that can be referenced as a `str`,
    /// including `&str`, `&String`, and `&Cow<str>`.
    ///
    /// # Example
    ///
    /// ```
    /// use libgraphql_parser::GraphQLParser;
    ///
    /// let source = "type Query { hello: String }";
    /// let parser = GraphQLParser::new(source);
    /// let result = parser.parse_schema_document();
    /// assert!(!result.has_errors());
    /// ```
    pub fn new<S: AsRef<str> + ?Sized>(
        source: &'src S,
    ) -> Self {
        let token_source =
            StrGraphQLTokenSource::new(source.as_ref());
        Self::from_token_source(token_source)
    }

    /// Creates a new parser from a string-like source with the
    /// given configuration.
    ///
    /// # Example
    ///
    /// ```
    /// use libgraphql_parser::GraphQLParser;
    /// use libgraphql_parser::GraphQLParserConfig;
    ///
    /// let source = "type Query { hello: String }";
    /// let config = GraphQLParserConfig::lean();
    /// let parser = GraphQLParser::with_config(source, config);
    /// let result = parser.parse_schema_document();
    /// assert!(!result.has_errors());
    /// ```
    pub fn with_config<S: AsRef<str> + ?Sized>(
        source: &'src S,
        config: GraphQLParserConfig,
    ) -> Self {
        let token_source =
            StrGraphQLTokenSource::new(source.as_ref());
        Self::from_token_source_with_config(token_source, config)
    }
}

impl<'src, TTokenSource: GraphQLTokenSource<'src>> GraphQLParser<'src, TTokenSource> {
    /// Maximum nesting depth for recursive parsing (values, selection
    /// sets, and type annotations).
    ///
    /// Prevents stack overflow from adversarial inputs like `[[[[[...`
    /// with hundreds of unclosed brackets. 32 levels is far beyond any
    /// realistic GraphQL document (most real-world documents nest
    /// fewer than 15 levels) while staying safe on the default 2 MiB
    /// thread stack in debug builds where AST types and stack frames
    /// are larger than in release builds.
    const MAX_RECURSION_DEPTH: usize = 32;

    /// Creates a new parser from a token source with the default
    /// configuration.
    pub fn from_token_source(
        token_source: TTokenSource,
    ) -> Self {
        Self::from_token_source_with_config(
            token_source,
            GraphQLParserConfig::default(),
        )
    }

    /// Creates a new parser from a token source with the given
    /// configuration.
    pub fn from_token_source_with_config(
        token_source: TTokenSource,
        config: GraphQLParserConfig,
    ) -> Self {
        Self {
            config,
            token_stream: GraphQLTokenStream::new(token_source),
            errors: Vec::new(),
            delimiter_stack: SmallVec::new(),
            recursion_depth: 0,
            last_end_position: None,
        }
    }

    // =========================================================================
    // Error recording and recovery
    // =========================================================================

    /// Records a parse error.
    fn record_error(&mut self, error: GraphQLParseError) {
        self.errors.push(error);
    }

    /// Push an open delimiter onto the stack.
    fn push_delimiter(
        &mut self,
        span: GraphQLSourceSpan,
        context: DelimiterContext,
    ) {
        self.delimiter_stack.push(OpenDelimiter { span, context });
    }

    /// Pop the most recent open delimiter.
    fn pop_delimiter(&mut self) -> Option<OpenDelimiter> {
        self.delimiter_stack.pop()
    }

    /// Skip tokens until we find the start of a new definition.
    ///
    /// Definition keywords: `type`, `interface`, `union`, `enum`, `scalar`,
    /// `input`, `directive`, `schema`, `extend`, `query`, `mutation`,
    /// `subscription`, `fragment`, or `{` (anonymous query).
    fn recover_to_next_definition(&mut self) {
        loop {
            // Extract info from peek without holding the borrow
            let action = match self.token_stream.peek() {
                None => RecoveryAction::Stop,
                Some(token) => match &token.kind {
                    GraphQLTokenKind::Eof => RecoveryAction::Stop,
                    GraphQLTokenKind::CurlyBraceOpen => RecoveryAction::Stop,
                    GraphQLTokenKind::Name(name) => {
                        let name_owned = name.to_string();
                        RecoveryAction::CheckKeyword(name_owned)
                    }
                    GraphQLTokenKind::StringValue(_) => {
                        RecoveryAction::CheckDescription
                    }
                    _ => RecoveryAction::Skip,
                },
            };

            match action {
                RecoveryAction::Stop => break,
                RecoveryAction::Skip => {
                    self.consume_token();
                }
                RecoveryAction::CheckKeyword(keyword) => {
                    if self.looks_like_definition_start(&keyword) {
                        break;
                    }
                    self.consume_token();
                }
                RecoveryAction::CheckDescription => {
                    // Check if next token after string is a definition keyword
                    let is_description_for_def =
                        if let Some(next) = self.token_stream.peek_nth(1)
                            && let GraphQLTokenKind::Name(name) = &next.kind {
                            matches!(
                                name.as_ref(),
                                "type"
                                    | "interface"
                                    | "union"
                                    | "enum"
                                    | "scalar"
                                    | "input"
                                    | "directive"
                                    | "schema"
                                    | "extend"
                            )
                        } else {
                            false
                        };
                    if is_description_for_def {
                        break;
                    }
                    self.consume_token();
                }
            }
        }
        // Clear delimiter stack since we're starting fresh
        self.delimiter_stack.clear();
    }

    /// Checks if the current keyword looks like the start of a definition by
    /// peeking at the next token.
    ///
    /// This helps avoid false recovery points like "type: String" where `type`
    /// appears as a field name rather than a type definition keyword.
    fn looks_like_definition_start(&mut self, keyword: &str) -> bool {
        let next = self.token_stream.peek_nth(1);

        match keyword {
            // Type definitions: `type Name`, `interface Name`, etc.
            // Next token should be a Name (the type name)
            "type" | "interface" | "union" | "enum" | "scalar" | "input" => {
                matches!(
                    next.map(|t| &t.kind),
                    Some(
                        GraphQLTokenKind::Name(_)
                            | GraphQLTokenKind::True
                            | GraphQLTokenKind::False
                            | GraphQLTokenKind::Null
                    )
                )
            }

            // `directive @Name` - next should be @
            "directive" => {
                matches!(next.map(|t| &t.kind), Some(GraphQLTokenKind::At))
            }

            // `schema { ... }` or `schema @directive` - next should be { or @
            "schema" => {
                matches!(
                    next.map(|t| &t.kind),
                    Some(GraphQLTokenKind::CurlyBraceOpen | GraphQLTokenKind::At)
                )
            }

            // `extend type ...` - next should be a type keyword
            "extend" => {
                if let Some(next_token) = next {
                    if let GraphQLTokenKind::Name(n) = &next_token.kind {
                        matches!(
                            n.as_ref(),
                            "type"
                                | "interface"
                                | "union"
                                | "enum"
                                | "scalar"
                                | "input"
                                | "schema"
                        )
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            // Operations: `query Name`, `query {`, `query(`, `query @`
            "query" | "mutation" | "subscription" => {
                matches!(
                    next.map(|t| &t.kind),
                    Some(
                        GraphQLTokenKind::Name(_)
                            | GraphQLTokenKind::True
                            | GraphQLTokenKind::False
                            | GraphQLTokenKind::Null
                            | GraphQLTokenKind::CurlyBraceOpen
                            | GraphQLTokenKind::ParenOpen
                            | GraphQLTokenKind::At
                    )
                ) || next.is_none() // `query` at EOF is still a recovery point
            }

            // `fragment Name on Type` - next should be a name (not "on")
            "fragment" => {
                if let Some(next_token) = next {
                    if let GraphQLTokenKind::Name(n) = &next_token.kind {
                        // Fragment name cannot be "on"
                        n.as_ref() != "on"
                    } else {
                        matches!(
                            &next_token.kind,
                            GraphQLTokenKind::True
                                | GraphQLTokenKind::False
                                | GraphQLTokenKind::Null
                        )
                    }
                } else {
                    false
                }
            }

            _ => false,
        }
    }

    // =========================================================================
    // Token expectation helpers
    // =========================================================================

    /// Expects a specific token kind and consumes it.
    ///
    /// Returns the owned token if it matches, or records an error
    /// and returns `Err(())`.
    fn expect(
        &mut self,
        expected_kind: &GraphQLTokenKind,
    ) -> Result<GraphQLToken<'src>, ()> {
        // Check token kind via peek (scoped borrow). We extract
        // what we need for the error path before dropping the
        // borrow so that consume_token() can be called on the
        // success path without a clone.
        let mismatch_info = match self.token_stream.peek() {
            None => {
                let span = self.eof_span();
                self.record_error(GraphQLParseError::new(
                    format!(
                        "expected `{}`",
                        Self::token_kind_display(expected_kind),
                    ),
                    span,
                    GraphQLParseErrorKind::UnexpectedEof {
                        expected: vec![
                            Self::token_kind_display(
                                expected_kind,
                            ),
                        ],
                    },
                ));
                return Err(());
            },
            Some(token) => {
                if Self::token_kinds_match(
                    &token.kind,
                    expected_kind,
                ) {
                    None
                } else {
                    Some((
                        token.span.clone(),
                        Self::token_kind_display(&token.kind),
                    ))
                }
            },
        };
        // Peek borrow is dropped — safe to mutate.
        if let Some((span, found)) = mismatch_info {
            self.record_error(GraphQLParseError::new(
                format!(
                    "expected `{}`, found `{}`",
                    Self::token_kind_display(expected_kind),
                    found,
                ),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec![
                        Self::token_kind_display(expected_kind),
                    ],
                    found,
                },
            ));
            Err(())
        } else {
            Ok(self.consume_token().unwrap())
        }
    }

    /// Expects a name token and returns an `ast::Name`.
    ///
    /// Moves the span from the consumed token (zero-cost).
    /// On error, does NOT consume the mismatched token (see
    /// error recovery convention in plan).
    fn expect_ast_name(&mut self) -> Result<ast::Name<'src>, ()> {
        let mismatch = match self.token_stream.peek() {
            None => {
                let span = self.eof_span();
                self.record_error(GraphQLParseError::new(
                    "expected name",
                    span,
                    GraphQLParseErrorKind::UnexpectedEof {
                        expected: vec!["name".to_string()],
                    },
                ));
                return Err(());
            },
            Some(token) => match &token.kind {
                GraphQLTokenKind::Name(_)
                | GraphQLTokenKind::True
                | GraphQLTokenKind::False
                | GraphQLTokenKind::Null => None,
                _ => Some((token.span.clone(), Self::token_kind_display(&token.kind))),
            },
        };
        if let Some((span, found)) = mismatch {
            self.record_error(GraphQLParseError::new(
                format!("expected name, found `{found}`"),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec!["name".to_string()],
                    found,
                },
            ));
            return Err(());
        }
        let token = self.consume_token().unwrap();
        if self.config.retain_syntax {
            let value = match &token.kind {
                GraphQLTokenKind::Name(s) => s.clone(),
                GraphQLTokenKind::True => Cow::Borrowed("true"),
                GraphQLTokenKind::False => Cow::Borrowed("false"),
                GraphQLTokenKind::Null => Cow::Borrowed("null"),
                _ => unreachable!(),
            };
            let span = token.span.clone();
            Ok(ast::Name {
                span,
                syntax: Some(Box::new(ast::NameSyntax { token })),
                value,
            })
        } else {
            let value = match token.kind {
                GraphQLTokenKind::Name(s) => s,
                GraphQLTokenKind::True => Cow::Borrowed("true"),
                GraphQLTokenKind::False => Cow::Borrowed("false"),
                GraphQLTokenKind::Null => Cow::Borrowed("null"),
                _ => unreachable!(),
            };
            Ok(ast::Name {
                span: token.span,
                syntax: None,
                value,
            })
        }
    }

    /// Expects a specific keyword (a Name token with specific text).
    ///
    /// This is used for GraphQL structural keywords like `query`, `mutation`,
    /// `type`, `interface`, etc.
    ///
    /// # Note on `true`, `false`, `null`
    ///
    /// This function does **not** match `True`, `False`, or `Null` tokens.
    /// Those are lexed as distinct token kinds, not as `Name` tokens. This is
    /// intentional: `expect_keyword()` is for structural keywords, not for
    /// boolean/null literals. If you need to accept `true`/`false`/`null` as
    /// names, use [`expect_name()`](Self::expect_name) instead.
    // TODO: Ensure test coverage verifies expect_keyword("true") does NOT
    // match a True token.
    fn expect_keyword(
        &mut self,
        keyword: &str,
    ) -> Result<GraphQLToken<'src>, ()> {
        let mismatch = match self.token_stream.peek() {
            None => {
                let span = self.eof_span();
                self.record_error(GraphQLParseError::new(
                    format!("expected `{keyword}`"),
                    span,
                    GraphQLParseErrorKind::UnexpectedEof {
                        expected: vec![keyword.to_string()],
                    },
                ));
                return Err(());
            },
            Some(token) => {
                if let GraphQLTokenKind::Name(name) = &token.kind
                    && name.as_ref() == keyword {
                    None
                } else {
                    Some((
                        token.span.clone(),
                        Self::token_kind_display(
                            &token.kind,
                        ),
                    ))
                }
            },
        };
        if let Some((span, found)) = mismatch {
            self.record_error(GraphQLParseError::new(
                format!(
                    "expected `{keyword}`, found `{found}`"
                ),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec![keyword.to_string()],
                    found,
                },
            ));
            return Err(());
        }
        Ok(self.consume_token().unwrap())
    }

    /// Checks if the current token is a specific keyword without consuming.
    ///
    /// This is used for GraphQL structural keywords like `query`, `mutation`,
    /// `type`, `interface`, etc.
    ///
    /// # Note on `true`, `false`, `null`
    ///
    /// This function returns `false` for `True`, `False`, and `Null` tokens,
    /// even if you call `peek_is_keyword("true")`. Those are lexed as distinct
    /// token kinds, not as `Name` tokens. This is intentional:
    /// `peek_is_keyword()` is for structural keywords, not for boolean/null
    /// literals.
    // TODO: Ensure test coverage verifies peek_is_keyword("true") returns
    // false when looking at a True token.
    fn peek_is_keyword(&mut self, keyword: &str) -> bool {
        match self.token_stream.peek() {
            Some(token) => {
                if let GraphQLTokenKind::Name(name) = &token.kind {
                    name.as_ref() == keyword
                } else {
                    false
                }
            }
            None => false,
        }
    }

    /// Checks if the current token matches the given kind without consuming.
    fn peek_is(&mut self, kind: &GraphQLTokenKind) -> bool {
        match self.token_stream.peek() {
            Some(token) => Self::token_kinds_match(&token.kind, kind),
            None => false,
        }
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    /// Consumes the next token from the stream and tracks its end
    /// position for EOF error reporting.
    fn consume_token(
        &mut self,
    ) -> Option<GraphQLToken<'src>> {
        let token = self.token_stream.consume();
        if let Some(ref t) = token {
            self.last_end_position =
                Some(t.span.end_exclusive);
        }
        token
    }

    /// Returns a span for EOF errors, anchored to the end of the
    /// last consumed token if available.
    fn eof_span(&self) -> GraphQLSourceSpan {
        if let Some(pos) = self.last_end_position {
            GraphQLSourceSpan::new(pos, pos)
        } else {
            let zero = SourcePosition::new(0, 0, Some(0), 0);
            GraphQLSourceSpan::new(zero, zero)
        }
    }

    /// Builds a `GraphQLSourceSpan` from a start span (moved)
    /// and the parser's last-consumed end position.
    ///
    /// `start` is taken by value to move (not clone) its
    /// `start_inclusive`. The caller should pass the span from
    /// a consumed token directly — never clone a span just to
    /// pass it here.
    fn make_span(&self, start: GraphQLSourceSpan) -> GraphQLSourceSpan {
        let end = self.last_end_position.unwrap_or(start.start_inclusive);
        GraphQLSourceSpan::new(start.start_inclusive, end)
    }

    /// Like [`make_span`](Self::make_span), but borrows `start`
    /// instead of moving it.
    ///
    /// Used on the `retain_syntax == true` path where the token
    /// (and its span) must survive to be moved into a syntax
    /// struct. Copies `start_inclusive` from the reference.
    fn make_span_ref(&self, start: &GraphQLSourceSpan) -> GraphQLSourceSpan {
        let end = self.last_end_position.unwrap_or(start.start_inclusive);
        GraphQLSourceSpan::new(start.start_inclusive, end)
    }

    /// Returns a human-readable display string for a token kind.
    fn token_kind_display(kind: &GraphQLTokenKind) -> String {
        match kind {
            GraphQLTokenKind::Ampersand => "&".to_string(),
            GraphQLTokenKind::At => "@".to_string(),
            GraphQLTokenKind::Bang => "!".to_string(),
            GraphQLTokenKind::Colon => ":".to_string(),
            GraphQLTokenKind::CurlyBraceClose => "}".to_string(),
            GraphQLTokenKind::CurlyBraceOpen => "{".to_string(),
            GraphQLTokenKind::Dollar => "$".to_string(),
            GraphQLTokenKind::Ellipsis => "...".to_string(),
            GraphQLTokenKind::Equals => "=".to_string(),
            GraphQLTokenKind::ParenClose => ")".to_string(),
            GraphQLTokenKind::ParenOpen => "(".to_string(),
            GraphQLTokenKind::Pipe => "|".to_string(),
            GraphQLTokenKind::SquareBracketClose => "]".to_string(),
            GraphQLTokenKind::SquareBracketOpen => "[".to_string(),
            GraphQLTokenKind::Name(s) => s.to_string(),
            GraphQLTokenKind::IntValue(s) => s.to_string(),
            GraphQLTokenKind::FloatValue(s) => s.to_string(),
            GraphQLTokenKind::StringValue(_) => "string".to_string(),
            GraphQLTokenKind::True => "true".to_string(),
            GraphQLTokenKind::False => "false".to_string(),
            GraphQLTokenKind::Null => "null".to_string(),
            GraphQLTokenKind::Eof => "end of input".to_string(),
            GraphQLTokenKind::Error(err) => {
                format!("tokenization error: {}", err.message)
            }
        }
    }

    /// Compares token kinds for equality, ignoring payload for variant
    /// matching.
    ///
    /// # Structure Note
    ///
    /// This function intentionally uses an exhaustive match on `actual` rather
    /// than a wildcard. This ensures that if a new `GraphQLTokenKind` variant
    /// is added, the compiler will produce an exhaustive-matching error,
    /// forcing us to explicitly handle the new variant. Do not refactor this
    /// to use catch-all match cases.
    fn token_kinds_match(
        actual: &GraphQLTokenKind,
        expected: &GraphQLTokenKind,
    ) -> bool {
        match actual {
            // For payload-carrying variants, we just check the variant matches
            // (not the payload) since we're checking "is this a Name?" not "is
            // this the specific name 'foo'?"
            GraphQLTokenKind::Name(_) => matches!(expected, GraphQLTokenKind::Name(_)),
            GraphQLTokenKind::IntValue(_) => {
                matches!(expected, GraphQLTokenKind::IntValue(_))
            }
            GraphQLTokenKind::FloatValue(_) => {
                matches!(expected, GraphQLTokenKind::FloatValue(_))
            }
            GraphQLTokenKind::StringValue(_) => {
                matches!(expected, GraphQLTokenKind::StringValue(_))
            }
            GraphQLTokenKind::Error(_) => {
                matches!(expected, GraphQLTokenKind::Error(_))
            }
            // Unit variants - exhaustive to catch new variants at compile time
            GraphQLTokenKind::Ampersand => actual == expected,
            GraphQLTokenKind::At => actual == expected,
            GraphQLTokenKind::Bang => actual == expected,
            GraphQLTokenKind::Colon => actual == expected,
            GraphQLTokenKind::CurlyBraceClose => actual == expected,
            GraphQLTokenKind::CurlyBraceOpen => actual == expected,
            GraphQLTokenKind::Dollar => actual == expected,
            GraphQLTokenKind::Ellipsis => actual == expected,
            GraphQLTokenKind::Equals => actual == expected,
            GraphQLTokenKind::ParenClose => actual == expected,
            GraphQLTokenKind::ParenOpen => actual == expected,
            GraphQLTokenKind::Pipe => actual == expected,
            GraphQLTokenKind::SquareBracketClose => actual == expected,
            GraphQLTokenKind::SquareBracketOpen => actual == expected,
            GraphQLTokenKind::True => actual == expected,
            GraphQLTokenKind::False => actual == expected,
            GraphQLTokenKind::Null => actual == expected,
            GraphQLTokenKind::Eof => actual == expected,
        }
    }

    /// Handles a lexer error token by converting it to a parse error.
    fn handle_lexer_error(&mut self, token: &GraphQLToken<'src>) {
        if let GraphQLTokenKind::Error(err) = &token.kind {
            self.record_error(GraphQLParseError::from_lexer_error(
                err.message.clone(),
                token.span.clone(),
                err.error_notes.clone(),
            ));
        }
    }

    // =========================================================================
    // Value parsing
    // =========================================================================

    /// Checks recursion depth and returns an error if the limit is
    /// exceeded. On success, increments the depth counter; the caller
    /// must call `exit_recursion()` when done (use the wrapper pattern
    /// to guarantee this).
    fn enter_recursion(&mut self) -> Result<(), ()> {
        self.recursion_depth += 1;
        if self.recursion_depth > Self::MAX_RECURSION_DEPTH {
            let span = self
                .token_stream.peek()
                .map(|t| t.span.clone())
                .unwrap_or_else(|| self.eof_span());
            self.consume_token();
            self.record_error(GraphQLParseError::new(
                "maximum nesting depth exceeded",
                span,
                GraphQLParseErrorKind::InvalidSyntax,
            ));
            self.recursion_depth -= 1;
            return Err(());
        }
        Ok(())
    }

    /// Decrements the recursion depth counter.
    fn exit_recursion(&mut self) {
        self.recursion_depth -= 1;
    }

    /// Parses a value (literal or variable reference).
    ///
    /// The `context` parameter specifies whether variables are allowed and
    /// provides context for error messages when they're not.
    fn parse_value(
        &mut self,
        context: ConstContext,
    ) -> Result<ast::Value<'src>, ()> {
        self.enter_recursion()?;
        let result = self.parse_value_impl(context);
        self.exit_recursion();
        result
    }

    /// Inner implementation of value parsing.
    fn parse_value_impl(
        &mut self, context: ConstContext,
    ) -> Result<ast::Value<'src>, ()> {
        match self.token_stream.peek() {
            None => {
                let span = self.eof_span();
                self.record_error(GraphQLParseError::new(
                    "expected value",
                    span,
                    GraphQLParseErrorKind::UnexpectedEof {
                        expected: vec![
                            "value".to_string(),
                        ],
                    },
                ));
                Err(())
            },
            Some(token) => {
                let span = token.span.clone();
                match &token.kind {
                    // Variable reference: $name
                    GraphQLTokenKind::Dollar => {
                        if !matches!(context, ConstContext::AllowVariables) {
                            self.consume_token();
                            self.record_error(GraphQLParseError::new(
                                format!("variables are not allowed in {}", context.description()),
                                span,
                                GraphQLParseErrorKind::InvalidSyntax,
                            ));
                            return Err(());
                        }
                        let dollar = self.consume_token().unwrap();
                        let name = self.expect_ast_name()?;
                        if self.config.retain_syntax {
                            let var_span = self.make_span_ref(&dollar.span);
                            Ok(ast::Value::Variable(ast::VariableValue {
                                name, span: var_span,
                                syntax: Some(Box::new(ast::VariableValueSyntax { dollar })),
                            }))
                        } else {
                            let var_span = self.make_span(dollar.span);
                            Ok(ast::Value::Variable(ast::VariableValue {
                                name, span: var_span, syntax: None,
                            }))
                        }
                    },

                    // Integer literal
                    GraphQLTokenKind::IntValue(raw) => {
                        let parse_result =
                            token.kind.parse_int_value();
                        match parse_result {
                            Some(Ok(val)) => {
                                if val > i32::MAX as i64
                                    || val
                                        < i32::MIN
                                            as i64
                                {
                                    let raw_str = raw
                                        .clone()
                                        .into_owned();
                                    self.consume_token();
                                    self.record_error(
                                        GraphQLParseError::new(
                                            format!(
                                                "integer \
                                                `{raw_str}` \
                                                overflows \
                                                32-bit \
                                                integer",
                                            ),
                                            span,
                                            GraphQLParseErrorKind::InvalidValue(
                                                ValueParsingError::Int(
                                                    raw_str,
                                                ),
                                            ),
                                        ),
                                    );
                                    Err(())
                                } else {
                                    let token = self.consume_token().unwrap();
                                    if self.config.retain_syntax {
                                        let span = token.span.clone();
                                        Ok(ast::Value::Int(ast::IntValue {
                                            span,
                                            syntax: Some(Box::new(ast::IntValueSyntax { token })),
                                            value: val as i32,
                                        }))
                                    } else {
                                        Ok(ast::Value::Int(ast::IntValue {
                                            span: token.span,
                                            syntax: None,
                                            value: val as i32,
                                        }))
                                    }
                                }
                            },
                            Some(Err(_)) => {
                                let raw_str = raw
                                    .clone()
                                    .into_owned();
                                self.consume_token();
                                self.record_error(
                                    GraphQLParseError::new(
                                        format!(
                                            "invalid \
                                            integer \
                                            `{raw_str}`",
                                        ),
                                        span,
                                        GraphQLParseErrorKind::InvalidValue(
                                            ValueParsingError::Int(
                                                raw_str,
                                            ),
                                        ),
                                    ),
                                );
                                Err(())
                            },
                            None => unreachable!(
                                "parse_int_value on \
                                IntValue token",
                            ),
                        }
                    },

                    // Float literal
                    GraphQLTokenKind::FloatValue(
                        raw,
                    ) => {
                        let parse_result =
                            token.kind
                                .parse_float_value();
                        match parse_result {
                            Some(Ok(val)) => {
                                if val.is_infinite()
                                    || val.is_nan()
                                {
                                    let raw_str = raw
                                        .clone()
                                        .into_owned();
                                    self.consume_token();
                                    self.record_error(
                                        GraphQLParseError::new(
                                            format!(
                                                "float \
                                                `{raw_str}` \
                                                is not a \
                                                finite \
                                                number",
                                            ),
                                            span,
                                            GraphQLParseErrorKind::InvalidValue(
                                                ValueParsingError::Float(
                                                    raw_str,
                                                ),
                                            ),
                                        ),
                                    );
                                    Err(())
                                } else {
                                    let token = self.consume_token().unwrap();
                                    if self.config.retain_syntax {
                                        let span = token.span.clone();
                                        Ok(ast::Value::Float(ast::FloatValue {
                                            span,
                                            syntax: Some(Box::new(
                                                ast::FloatValueSyntax { token },
                                            )),
                                            value: val,
                                        }))
                                    } else {
                                        Ok(ast::Value::Float(ast::FloatValue {
                                            span: token.span,
                                            syntax: None,
                                            value: val,
                                        }))
                                    }
                                }
                            },
                            Some(Err(_)) => {
                                let raw_str = raw
                                    .clone()
                                    .into_owned();
                                self.consume_token();
                                self.record_error(
                                    GraphQLParseError::new(
                                        format!(
                                            "invalid \
                                            float \
                                            `{raw_str}`",
                                        ),
                                        span,
                                        GraphQLParseErrorKind::InvalidValue(
                                            ValueParsingError::Float(
                                                raw_str,
                                            ),
                                        ),
                                    ),
                                );
                                Err(())
                            },
                            None => unreachable!(
                                "parse_float_value \
                                on FloatValue token",
                            ),
                        }
                    },

                    // String literal
                    GraphQLTokenKind::StringValue(raw) => {
                        let is_block = raw.starts_with("\"\"\"");
                        let parse_result = token.kind.parse_string_value();
                        let consumed = self.consume_token().unwrap();
                        match parse_result {
                            Some(Ok(parsed)) => {
                                if self.config.retain_syntax {
                                    let s = consumed.span.clone();
                                    Ok(ast::Value::String(ast::StringValue {
                                        is_block, span: s,
                                        syntax: Some(Box::new(
                                            ast::StringValueSyntax { token: consumed },
                                        )),
                                        value: Cow::Owned(parsed),
                                    }))
                                } else {
                                    Ok(ast::Value::String(ast::StringValue {
                                        is_block, span: consumed.span,
                                        syntax: None, value: Cow::Owned(parsed),
                                    }))
                                }
                            },
                            Some(Err(e)) => {
                                self.record_error(GraphQLParseError::new(
                                    format!("invalid string: {e}"),
                                    span,
                                    GraphQLParseErrorKind::InvalidValue(
                                        ValueParsingError::String(e),
                                    ),
                                ));
                                Err(())
                            },
                            None => {
                                self.record_error(GraphQLParseError::new(
                                    "invalid string", span,
                                    GraphQLParseErrorKind::InvalidSyntax,
                                ));
                                Err(())
                            },
                        }
                    },

                    // Boolean literals
                    GraphQLTokenKind::True => {
                        let token = self.consume_token().unwrap();
                        if self.config.retain_syntax {
                            let span = token.span.clone();
                            Ok(ast::Value::Boolean(ast::BooleanValue {
                                span,
                                syntax: Some(Box::new(
                                    ast::BooleanValueSyntax { token },
                                )),
                                value: true,
                            }))
                        } else {
                            Ok(ast::Value::Boolean(ast::BooleanValue {
                                span: token.span, syntax: None, value: true,
                            }))
                        }
                    },
                    GraphQLTokenKind::False => {
                        let token = self.consume_token().unwrap();
                        if self.config.retain_syntax {
                            let span = token.span.clone();
                            Ok(ast::Value::Boolean(ast::BooleanValue {
                                span,
                                syntax: Some(Box::new(
                                    ast::BooleanValueSyntax { token },
                                )),
                                value: false,
                            }))
                        } else {
                            Ok(ast::Value::Boolean(ast::BooleanValue {
                                span: token.span, syntax: None, value: false,
                            }))
                        }
                    },

                    // Null literal
                    GraphQLTokenKind::Null => {
                        let token = self.consume_token().unwrap();
                        if self.config.retain_syntax {
                            let span = token.span.clone();
                            Ok(ast::Value::Null(ast::NullValue {
                                span,
                                syntax: Some(Box::new(ast::NullValueSyntax { token })),
                            }))
                        } else {
                            Ok(ast::Value::Null(ast::NullValue {
                                span: token.span, syntax: None,
                            }))
                        }
                    },

                    // List literal: [value, ...]
                    GraphQLTokenKind::SquareBracketOpen => {
                        self.parse_list_value(context)
                    },

                    // Object literal: {field: value, ...}
                    GraphQLTokenKind::CurlyBraceOpen => {
                        self.parse_object_value(context)
                    },

                    // Enum value (any other name)
                    GraphQLTokenKind::Name(_) => {
                        let token = self.consume_token().unwrap();
                        if self.config.retain_syntax {
                            let value = match &token.kind {
                                GraphQLTokenKind::Name(s) => s.clone(),
                                _ => unreachable!(),
                            };
                            let span = token.span.clone();
                            Ok(ast::Value::Enum(ast::EnumValue {
                                span,
                                syntax: Some(Box::new(ast::EnumValueSyntax { token })),
                                value,
                            }))
                        } else {
                            let value = match token.kind {
                                GraphQLTokenKind::Name(s) => s,
                                _ => unreachable!(),
                            };
                            Ok(ast::Value::Enum(ast::EnumValue {
                                span: token.span, syntax: None, value,
                            }))
                        }
                    },

                    // Lexer error
                    GraphQLTokenKind::Error(_) => {
                        let token = token.clone();
                        self.handle_lexer_error(
                            &token,
                        );
                        self.consume_token();
                        Err(())
                    },

                    // Unexpected token
                    _ => {
                        let found =
                            Self::token_kind_display(
                                &token.kind,
                            );
                        self.record_error(
                            GraphQLParseError::new(
                                format!(
                                    "expected value, \
                                    found `{found}`",
                                ),
                                span,
                                GraphQLParseErrorKind::UnexpectedToken {
                                    expected: vec![
                                        "value"
                                            .to_string(),
                                    ],
                                    found,
                                },
                            ),
                        );
                        Err(())
                    },
                }
            },
        }
    }

    /// Parses a list value: `[value, value, ...]`
    fn parse_list_value(
        &mut self, context: ConstContext,
    ) -> Result<ast::Value<'src>, ()> {
        let open_token = self.expect(&GraphQLTokenKind::SquareBracketOpen)?;
        self.push_delimiter(open_token.span.clone(), DelimiterContext::ListValue);
        let mut values = Vec::new();
        loop {
            if self.peek_is(&GraphQLTokenKind::SquareBracketClose) {
                break;
            }
            if self.token_stream.is_at_end() {
                let span = self.eof_span();
                let open_delim = self.pop_delimiter();
                let mut error = GraphQLParseError::new(
                    "unclosed `[`",
                    span,
                    GraphQLParseErrorKind::UnclosedDelimiter {
                        delimiter: "[".to_string(),
                    },
                );
                if let Some(delim) = open_delim {
                    error.add_note_with_span("opening `[` here", delim.span);
                }
                self.record_error(error);
                return Err(());
            }
            match self.parse_value(context) {
                Ok(value) => values.push(value),
                Err(()) => {
                    self.skip_to_list_recovery_point();
                    if self.peek_is(&GraphQLTokenKind::SquareBracketClose) {
                        break;
                    }
                },
            }
        }
        let close_token = self.expect(&GraphQLTokenKind::SquareBracketClose)?;
        self.pop_delimiter();
        if self.config.retain_syntax {
            let span = self.make_span_ref(&open_token.span);
            Ok(ast::Value::List(ast::ListValue {
                span,
                syntax: Some(Box::new(ast::ListValueSyntax {
                    brackets: ast::DelimiterPair { close: close_token, open: open_token },
                })),
                values,
            }))
        } else {
            let span = self.make_span(open_token.span);
            Ok(ast::Value::List(ast::ListValue { span, syntax: None, values }))
        }
    }

    /// Parses an object value: `{ field: value, ... }`
    fn parse_object_value(
        &mut self, context: ConstContext,
    ) -> Result<ast::Value<'src>, ()> {
        let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
        self.push_delimiter(open_token.span.clone(), DelimiterContext::ObjectValue);
        let mut fields = Vec::new();
        loop {
            if self.peek_is(&GraphQLTokenKind::CurlyBraceClose) {
                break;
            }
            if self.token_stream.is_at_end() {
                let span = self.eof_span();
                let open_delim = self.pop_delimiter();
                let mut error = GraphQLParseError::new(
                    "unclosed `{`",
                    span,
                    GraphQLParseErrorKind::UnclosedDelimiter {
                        delimiter: "{".to_string(),
                    },
                );
                if let Some(delim) = open_delim {
                    error.add_note_with_span(
                        format!("opening `{{` in {} here", delim.context.description()),
                        delim.span,
                    );
                }
                self.record_error(error);
                return Err(());
            }
            let field_name = self.expect_ast_name()?;
            let colon_token = self.expect(&GraphQLTokenKind::Colon)?;
            let value = self.parse_value(context)?;
            let field_span = GraphQLSourceSpan::new(
                field_name.span.start_inclusive,
                self.last_end_position
                    .unwrap_or(field_name.span.end_exclusive),
            );
            let field_syntax = if self.config.retain_syntax {
                Some(Box::new(ast::ObjectFieldSyntax { colon: colon_token }))
            } else {
                None
            };
            fields.push(ast::ObjectField {
                name: field_name, span: field_span, syntax: field_syntax, value,
            });
        }
        let close_token = self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
        self.pop_delimiter();
        if self.config.retain_syntax {
            let span = self.make_span_ref(&open_token.span);
            Ok(ast::Value::Object(ast::ObjectValue {
                fields, span,
                syntax: Some(Box::new(ast::ObjectValueSyntax {
                    braces: ast::DelimiterPair { close: close_token, open: open_token },
                })),
            }))
        } else {
            let span = self.make_span(open_token.span);
            Ok(ast::Value::Object(ast::ObjectValue { fields, span, syntax: None }))
        }
    }

    /// Skip tokens to find a recovery point within a list value.
    ///
    /// # Structure Note
    ///
    /// This function intentionally uses an exhaustive match rather than a
    /// wildcard. This ensures that if a new `GraphQLTokenKind` variant is
    /// added, the compiler will produce an exhaustive-matching error, forcing
    /// us to explicitly decide whether the new variant is a recovery point.
    /// Do not refactor this to use catch-all match cases.
    fn skip_to_list_recovery_point(&mut self) {
        loop {
            match self.token_stream.peek() {
                None => break,
                Some(token) => match &token.kind {
                    // End of list or input - stop
                    GraphQLTokenKind::SquareBracketClose | GraphQLTokenKind::Eof => break,
                    // Value starters - potential recovery point
                    GraphQLTokenKind::Dollar
                    | GraphQLTokenKind::IntValue(_)
                    | GraphQLTokenKind::FloatValue(_)
                    | GraphQLTokenKind::StringValue(_)
                    | GraphQLTokenKind::True
                    | GraphQLTokenKind::False
                    | GraphQLTokenKind::Null
                    | GraphQLTokenKind::SquareBracketOpen
                    | GraphQLTokenKind::CurlyBraceOpen
                    | GraphQLTokenKind::Name(_) => break,
                    // Skip these tokens (not valid value starters)
                    GraphQLTokenKind::Ampersand
                    | GraphQLTokenKind::At
                    | GraphQLTokenKind::Bang
                    | GraphQLTokenKind::Colon
                    | GraphQLTokenKind::CurlyBraceClose
                    | GraphQLTokenKind::Ellipsis
                    | GraphQLTokenKind::Equals
                    | GraphQLTokenKind::ParenClose
                    | GraphQLTokenKind::ParenOpen
                    | GraphQLTokenKind::Pipe
                    | GraphQLTokenKind::Error(_) => {
                        self.consume_token();
                    }
                },
            }
        }
    }

    // =========================================================================
    // Type annotation parsing
    // =========================================================================

    /// Parses a type annotation: `TypeName`, `[Type]`, `Type!`, `[Type]!`, etc.
    fn parse_type_annotation(&mut self) -> Result<ast::TypeAnnotation<'src>, ()> {
        self.enter_recursion()?;
        let result = self.parse_type_annotation_impl();
        self.exit_recursion();
        result
    }

    /// Inner implementation of type annotation parsing.
    fn parse_type_annotation_impl(&mut self) -> Result<ast::TypeAnnotation<'src>, ()> {
        if self.peek_is(&GraphQLTokenKind::SquareBracketOpen) {
            self.parse_list_type_annotation()
        } else {
            self.parse_named_type_annotation()
        }
    }

    /// Parses a named type annotation: `TypeName` or `TypeName!`
    fn parse_named_type_annotation(&mut self) -> Result<ast::TypeAnnotation<'src>, ()> {
        let name = self.expect_ast_name()?;
        let (nullability, span_end) = if self.peek_is(&GraphQLTokenKind::Bang) {
            let bang = self.consume_token().unwrap();
            let end = bang.span.end_exclusive;
            let syntax = if self.config.retain_syntax { Some(bang) } else { None };
            (ast::Nullability::NonNull { syntax }, end)
        } else {
            (ast::Nullability::Nullable, name.span.end_exclusive)
        };
        let span = GraphQLSourceSpan::new(name.span.start_inclusive, span_end);
        Ok(ast::TypeAnnotation::Named(
            ast::NamedTypeAnnotation { name, nullability, span },
        ))
    }

    /// Parses a list type annotation: `[InnerType]` or `[InnerType]!`
    fn parse_list_type_annotation(&mut self) -> Result<ast::TypeAnnotation<'src>, ()> {
        let open_token = self.expect(&GraphQLTokenKind::SquareBracketOpen)?;
        self.push_delimiter(open_token.span.clone(), DelimiterContext::ListType);
        let element_type = Box::new(self.parse_type_annotation()?);
        let close_token = self.expect(&GraphQLTokenKind::SquareBracketClose)?;
        self.pop_delimiter();
        let (nullability, span_end) = if self.peek_is(&GraphQLTokenKind::Bang) {
            let bang = self.consume_token().unwrap();
            let end = bang.span.end_exclusive;
            let syntax = if self.config.retain_syntax { Some(bang) } else { None };
            (ast::Nullability::NonNull { syntax }, end)
        } else {
            let end = self.last_end_position
                .unwrap_or(open_token.span.start_inclusive);
            (ast::Nullability::Nullable, end)
        };
        if self.config.retain_syntax {
            let span = GraphQLSourceSpan::new(
                open_token.span.start_inclusive, span_end,
            );
            Ok(ast::TypeAnnotation::List(ast::ListTypeAnnotation {
                element_type, nullability, span,
                syntax: Some(Box::new(ast::ListTypeAnnotationSyntax {
                    brackets: ast::DelimiterPair { close: close_token, open: open_token },
                })),
            }))
        } else {
            let span = GraphQLSourceSpan::new(open_token.span.start_inclusive, span_end);
            Ok(ast::TypeAnnotation::List(
                ast::ListTypeAnnotation { element_type, nullability, span, syntax: None },
            ))
        }
    }

    // =========================================================================
    // Directive annotation parsing
    // =========================================================================

    /// Parses zero or more directive annotations: `@directive(args)...`
    fn parse_directive_annotations(
        &mut self,
    ) -> Result<Vec<ast::DirectiveAnnotation<'src>>, ()> {
        let mut directives = Vec::new();
        while self.peek_is(&GraphQLTokenKind::At) {
            directives.push(self.parse_directive_annotation()?);
        }
        Ok(directives)
    }

    /// Parses a single directive annotation: `@name` or `@name(args)`
    fn parse_directive_annotation(&mut self) -> Result<ast::DirectiveAnnotation<'src>, ()> {
        let at_token = self.expect(&GraphQLTokenKind::At)?;
        let name = self.expect_ast_name()?;
        let (arguments, argument_delimiters) = if self.peek_is(&GraphQLTokenKind::ParenOpen) {
            self.parse_ast_arguments(
                DelimiterContext::DirectiveArguments,
                ConstContext::AllowVariables,
            )?
        } else {
            (Vec::new(), None)
        };
        if self.config.retain_syntax {
            let span = self.make_span_ref(&at_token.span);
            Ok(ast::DirectiveAnnotation {
                arguments, name, span,
                syntax: Some(Box::new(ast::DirectiveAnnotationSyntax {
                    argument_parens: argument_delimiters, at_sign: at_token,
                })),
            })
        } else {
            let span = self.make_span(at_token.span);
            Ok(ast::DirectiveAnnotation { arguments, name, span, syntax: None })
        }
    }

    /// Parses directive annotations in const contexts (arguments must be const values).
    fn parse_const_directive_annotations(
        &mut self,
    ) -> Result<Vec<ast::DirectiveAnnotation<'src>>, ()> {
        let mut directives = Vec::new();
        while self.peek_is(&GraphQLTokenKind::At) {
            directives.push(self.parse_const_directive_annotation()?);
        }
        Ok(directives)
    }

    /// Parses a directive annotation with const-only arguments.
    fn parse_const_directive_annotation(&mut self) -> Result<ast::DirectiveAnnotation<'src>, ()> {
        let at_token = self.expect(&GraphQLTokenKind::At)?;
        let name = self.expect_ast_name()?;
        let (arguments, argument_delimiters) = if self.peek_is(&GraphQLTokenKind::ParenOpen) {
            self.parse_ast_arguments(
                DelimiterContext::DirectiveArguments,
                ConstContext::DirectiveArgument,
            )?
        } else {
            (Vec::new(), None)
        };
        if self.config.retain_syntax {
            let span = self.make_span_ref(&at_token.span);
            Ok(ast::DirectiveAnnotation {
                arguments, name, span,
                syntax: Some(Box::new(ast::DirectiveAnnotationSyntax {
                    argument_parens: argument_delimiters, at_sign: at_token,
                })),
            })
        } else {
            let span = self.make_span(at_token.span);
            Ok(ast::DirectiveAnnotation { arguments, name, span, syntax: None })
        }
    }

    // =========================================================
    // Arguments parsing
    // =========================================================

    /// Parses arguments: `(name: value, ...)`
    fn parse_ast_arguments(
        &mut self, delim_context: DelimiterContext, const_context: ConstContext,
    ) -> Result<(Vec<ast::Argument<'src>>, Option<ast::DelimiterPair<'src>>), ()> {
        let open_token = self.expect(&GraphQLTokenKind::ParenOpen)?;
        self.push_delimiter(open_token.span.clone(), delim_context);
        let mut arguments = Vec::new();
        if self.peek_is(&GraphQLTokenKind::ParenClose) {
            let span = open_token.span.clone();
            self.record_error(GraphQLParseError::new(
                "argument list cannot be empty; omit the parentheses instead",
                span,
                GraphQLParseErrorKind::InvalidEmptyConstruct {
                    construct: "argument list".to_string(),
                },
            ));
        }
        loop {
            if self.peek_is(&GraphQLTokenKind::ParenClose) {
                break;
            }
            if self.token_stream.is_at_end() {
                self.handle_unclosed_paren();
                return Err(());
            }
            let arg_name = self.expect_ast_name()?;
            let colon_token = self.expect(&GraphQLTokenKind::Colon)?;
            let value = self.parse_value(const_context)?;
            let span = GraphQLSourceSpan::new(
                arg_name.span.start_inclusive,
                self.last_end_position
                    .unwrap_or(arg_name.span.end_exclusive),
            );
            let syntax = if self.config.retain_syntax {
                Some(Box::new(ast::ArgumentSyntax { colon: colon_token }))
            } else {
                None
            };
            arguments.push(ast::Argument {
                name: arg_name, span, syntax, value,
            });
        }
        let close_token = self.expect(&GraphQLTokenKind::ParenClose)?;
        self.pop_delimiter();
        let delimiters = if self.config.retain_syntax {
            Some(ast::DelimiterPair { close: close_token, open: open_token })
        } else {
            None
        };
        Ok((arguments, delimiters))
    }

    /// Helper for unclosed parenthesis errors.
    fn handle_unclosed_paren(&mut self) {
        let span = self.eof_span();
        let open_delim = self.pop_delimiter();
        let mut error = GraphQLParseError::new(
            "unclosed `(`",
            span,
            GraphQLParseErrorKind::UnclosedDelimiter {
                delimiter: "(".to_string(),
            },
        );
        if let Some(delim) = open_delim {
            error.add_note_with_span(
                format!("opening `(` in {} here", delim.context.description()),
                delim.span,
            );
        }
        self.record_error(error);
    }

    // =========================================================================
    // Selection set parsing
    // =========================================================================

    /// Parses a selection set: `{ selection... }`
    fn parse_selection_set(&mut self) -> Result<ast::SelectionSet<'src>, ()> {
        self.enter_recursion()?;
        let result = self.parse_selection_set_impl();
        self.exit_recursion();
        result
    }

    /// Inner implementation of selection set parsing.
    fn parse_selection_set_impl(&mut self) -> Result<ast::SelectionSet<'src>, ()> {
        let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
        self.push_delimiter(open_token.span.clone(), DelimiterContext::SelectionSet);
        let mut selections = Vec::new();
        if self.peek_is(&GraphQLTokenKind::CurlyBraceClose) {
            let span = open_token.span.clone();
            self.record_error(GraphQLParseError::new(
                "selection set cannot be empty",
                span,
                GraphQLParseErrorKind::InvalidEmptyConstruct {
                    construct: "selection set".to_string(),
                },
            ));
        }
        loop {
            if self.peek_is(&GraphQLTokenKind::CurlyBraceClose) {
                break;
            }
            if self.token_stream.is_at_end() {
                self.handle_unclosed_brace();
                return Err(());
            }
            match self.parse_selection() {
                Ok(sel) => selections.push(sel),
                Err(()) => {
                    self.skip_to_selection_recovery_point();
                },
            }
        }
        let close_token = self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
        self.pop_delimiter();
        if self.config.retain_syntax {
            let span = self.make_span_ref(&open_token.span);
            Ok(ast::SelectionSet {
                selections, span,
                syntax: Some(Box::new(ast::SelectionSetSyntax {
                    braces: ast::DelimiterPair { close: close_token, open: open_token },
                })),
            })
        } else {
            let span = self.make_span(open_token.span);
            Ok(ast::SelectionSet { selections, span, syntax: None })
        }
    }

    /// Parses a single selection (field, fragment spread, or inline fragment).
    fn parse_selection(&mut self) -> Result<ast::Selection<'src>, ()> {
        if self.peek_is(&GraphQLTokenKind::Ellipsis) {
            let ellipsis_token = self.expect(&GraphQLTokenKind::Ellipsis)?;
            if self.peek_is_keyword("on")
                || self.peek_is(&GraphQLTokenKind::At)
                || self.peek_is(&GraphQLTokenKind::CurlyBraceOpen)
            {
                self.parse_inline_fragment(ellipsis_token)
            } else {
                self.parse_fragment_spread(ellipsis_token)
            }
        } else {
            self.parse_field().map(ast::Selection::Field)
        }
    }

    /// Parses a field: `alias: name(args) @directives { selections }`
    fn parse_field(&mut self) -> Result<ast::Field<'src>, ()> {
        let first_name = self.expect_ast_name()?;
        let (alias, alias_colon, name) = if self.peek_is(&GraphQLTokenKind::Colon) {
            let colon_token = self.consume_token().unwrap();
            let field_name = self.expect_ast_name()?;
            (Some(first_name), Some(colon_token), field_name)
        } else {
            (None, None, first_name)
        };
        let (arguments, argument_delimiters) = if self.peek_is(&GraphQLTokenKind::ParenOpen) {
            self.parse_ast_arguments(
                DelimiterContext::FieldArguments,
                ConstContext::AllowVariables,
            )?
        } else {
            (Vec::new(), None)
        };
        let directives = self.parse_directive_annotations()?;
        let selection_set = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            Some(self.parse_selection_set()?)
        } else {
            None
        };
        let start = match &alias {
            Some(a) => a.span.start_inclusive,
            None => name.span.start_inclusive,
        };
        let end = self.last_end_position
            .unwrap_or(name.span.end_exclusive);
        let span = GraphQLSourceSpan::new(start, end);
        let syntax = if self.config.retain_syntax {
            Some(Box::new(ast::FieldSyntax {
                alias_colon,
                argument_parens: argument_delimiters,
            }))
        } else {
            None
        };
        Ok(ast::Field { alias, arguments, directives, name, selection_set, span, syntax })
    }

    /// Parses a fragment spread: `...FragmentName @directives` (called after consuming `...`)
    fn parse_fragment_spread(
        &mut self, ellipsis_token: GraphQLToken<'src>,
    ) -> Result<ast::Selection<'src>, ()> {
        let name = self.expect_ast_name()?;
        let directives = self.parse_directive_annotations()?;
        if self.config.retain_syntax {
            let span = self.make_span_ref(&ellipsis_token.span);
            Ok(ast::Selection::FragmentSpread(ast::FragmentSpread {
                directives, name, span,
                syntax: Some(Box::new(ast::FragmentSpreadSyntax {
                    ellipsis: ellipsis_token,
                })),
            }))
        } else {
            let span = self.make_span(ellipsis_token.span);
            Ok(ast::Selection::FragmentSpread(
                ast::FragmentSpread { directives, name, span, syntax: None },
            ))
        }
    }

    /// Parses an inline fragment: `... on Type @directives { sel }` or `... @directives { sel }`
    /// (called after consuming `...`)
    fn parse_inline_fragment(
        &mut self, ellipsis_token: GraphQLToken<'src>,
    ) -> Result<ast::Selection<'src>, ()> {
        let type_condition = if self.peek_is_keyword("on") {
            Some(self.parse_type_condition()?)
        } else {
            None
        };
        let directives = self.parse_directive_annotations()?;
        let selection_set = self.parse_selection_set()?;
        if self.config.retain_syntax {
            let span = self.make_span_ref(&ellipsis_token.span);
            Ok(ast::Selection::InlineFragment(ast::InlineFragment {
                directives, selection_set, span,
                syntax: Some(Box::new(ast::InlineFragmentSyntax {
                    ellipsis: ellipsis_token,
                })),
                type_condition,
            }))
        } else {
            let span = self.make_span(ellipsis_token.span);
            Ok(ast::Selection::InlineFragment(ast::InlineFragment {
                directives, selection_set, span, syntax: None, type_condition,
            }))
        }
    }

    /// Skip tokens to find a recovery point within a selection set.
    fn skip_to_selection_recovery_point(&mut self) {
        loop {
            match self.token_stream.peek() {
                None => break,
                Some(token) => match &token.kind {
                    GraphQLTokenKind::CurlyBraceClose | GraphQLTokenKind::Eof => break,
                    // Selection starters
                    GraphQLTokenKind::Ellipsis | GraphQLTokenKind::Name(_) => break,
                    // Also treat true/false/null as potential field names
                    GraphQLTokenKind::True
                    | GraphQLTokenKind::False
                    | GraphQLTokenKind::Null => break,
                    _ => {
                        self.consume_token();
                    }
                },
            }
        }
    }

    /// Helper for unclosed brace errors.
    fn handle_unclosed_brace(&mut self) {
        let span = self.eof_span();
        let open_delim = self.pop_delimiter();
        let mut error = GraphQLParseError::new(
            "unclosed `{`",
            span,
            GraphQLParseErrorKind::UnclosedDelimiter {
                delimiter: "{".to_string(),
            },
        );
        if let Some(delim) = open_delim {
            error.add_note_with_span(
                format!(
                    "opening `{{` in {} here",
                    delim.context.description()
                ),
                delim.span,
            );
        }
        self.record_error(error);
    }

    // =========================================================================
    // Operation parsing
    // =========================================================================

    /// Parses an operation definition.
    fn parse_operation_definition(&mut self) -> Result<ast::OperationDefinition<'src>, ()> {
        // Shorthand query: just a selection set with no keyword
        if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            let selection_set = self.parse_selection_set()?;
            let span = selection_set.span.clone();
            return Ok(ast::OperationDefinition {
                description: None, directives: Vec::new(), name: None,
                operation_kind: ast::OperationKind::Query, selection_set,
                shorthand: true, span, syntax: None,
                variable_definitions: Vec::new(),
            });
        }

        // Parse operation type keyword
        let (op_kind, keyword_token) = if self.peek_is_keyword("query") {
            (ast::OperationKind::Query, self.expect_keyword("query")?)
        } else if self.peek_is_keyword("mutation") {
            (ast::OperationKind::Mutation, self.expect_keyword("mutation")?)
        } else if self.peek_is_keyword("subscription") {
            (ast::OperationKind::Subscription, self.expect_keyword("subscription")?)
        } else {
            let span = self
                .token_stream.peek()
                .map(|t| t.span.clone())
                .unwrap_or_else(|| self.eof_span());
            let found = self
                .token_stream.peek()
                .map(|t| Self::token_kind_display(&t.kind))
                .unwrap_or_else(|| "end of input".to_string());
            self.record_error(GraphQLParseError::new(
                format!(
                    "expected operation type (`query`, `mutation`, or \
                    `subscription`), found `{found}`"
                ),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec![
                        "query".to_string(),
                        "mutation".to_string(),
                        "subscription".to_string(),
                    ],
                    found,
                },
            ));
            return Err(());
        };

        // Optional operation name
        let name = if !self.peek_is(&GraphQLTokenKind::ParenOpen)
            && !self.peek_is(&GraphQLTokenKind::At)
            && !self.peek_is(&GraphQLTokenKind::CurlyBraceOpen)
        {
            if let Some(token) = self.token_stream.peek() {
                match &token.kind {
                    GraphQLTokenKind::Name(_) | GraphQLTokenKind::True
                    | GraphQLTokenKind::False | GraphQLTokenKind::Null => {
                        Some(self.expect_ast_name()?)
                    },
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        // Optional variable definitions
        let (variable_definitions, variable_definition_delimiters) =
            if self.peek_is(&GraphQLTokenKind::ParenOpen) {
                self.parse_variable_definitions()?
            } else {
                (Vec::new(), None)
            };

        // Directives and selection set
        let directives = self.parse_directive_annotations()?;
        let selection_set = self.parse_selection_set()?;
        if self.config.retain_syntax {
            let span = self.make_span_ref(&keyword_token.span);
            Ok(ast::OperationDefinition {
                description: None, directives, name, operation_kind: op_kind,
                selection_set, shorthand: false, span,
                syntax: Some(Box::new(ast::OperationDefinitionSyntax {
                    operation_keyword: Some(keyword_token),
                    variable_definition_parens: variable_definition_delimiters,
                })),
                variable_definitions,
            })
        } else {
            let span = self.make_span(keyword_token.span);
            Ok(ast::OperationDefinition {
                description: None, directives, name, operation_kind: op_kind,
                selection_set, shorthand: false, span, syntax: None,
                variable_definitions,
            })
        }
    }

    /// Parses variable definitions: `($var: Type = default, ...)`
    fn parse_variable_definitions(
        &mut self,
    ) -> Result<(Vec<ast::VariableDefinition<'src>>, Option<ast::DelimiterPair<'src>>), ()> {
        let open_token = self.expect(&GraphQLTokenKind::ParenOpen)?;
        self.push_delimiter(
            open_token.span.clone(),
            DelimiterContext::VariableDefinitions,
        );

        let mut definitions = Vec::new();

        if self.peek_is(&GraphQLTokenKind::ParenClose) {
            let span = open_token.span.clone();
            self.record_error(GraphQLParseError::new(
                "variable definitions cannot be empty; omit the parentheses \
                instead",
                span,
                GraphQLParseErrorKind::InvalidEmptyConstruct {
                    construct: "variable definitions".to_string(),
                },
            ));
        }

        loop {
            if self.peek_is(&GraphQLTokenKind::ParenClose) {
                break;
            }
            if self.token_stream.is_at_end() {
                self.handle_unclosed_paren();
                return Err(());
            }

            definitions.push(self.parse_variable_definition()?);
        }

        let close_token = self.expect(&GraphQLTokenKind::ParenClose)?;
        self.pop_delimiter();
        let delimiters = if self.config.retain_syntax {
            Some(ast::DelimiterPair { close: close_token, open: open_token })
        } else {
            None
        };
        Ok((definitions, delimiters))
    }

    /// Parses a single variable definition: `$name: Type = default @directives`
    fn parse_variable_definition(&mut self) -> Result<ast::VariableDefinition<'src>, ()> {
        let dollar_token = self.expect(&GraphQLTokenKind::Dollar)?;
        let variable = self.expect_ast_name()?;
        let colon_token = self.expect(&GraphQLTokenKind::Colon)?;
        let var_type = self.parse_type_annotation()?;
        let (default_value, equals_token) = if self.peek_is(&GraphQLTokenKind::Equals) {
            let eq = self.consume_token().unwrap();
            (Some(self.parse_value(ConstContext::VariableDefaultValue)?), Some(eq))
        } else {
            (None, None)
        };
        let directives = self.parse_const_directive_annotations()?;
        if self.config.retain_syntax {
            let span = self.make_span_ref(&dollar_token.span);
            Ok(ast::VariableDefinition {
                default_value, description: None, directives, span,
                syntax: Some(Box::new(ast::VariableDefinitionSyntax {
                    colon: colon_token, dollar: dollar_token, equals: equals_token,
                })),
                var_type, variable,
            })
        } else {
            let span = self.make_span(dollar_token.span);
            Ok(ast::VariableDefinition {
                default_value, description: None, directives, span,
                syntax: None, var_type, variable,
            })
        }
    }

    // =========================================================================
    // Fragment parsing
    // =========================================================================

    /// Parses a fragment definition: `fragment Name on Type @directives { ... }`
    fn parse_fragment_definition(&mut self) -> Result<ast::FragmentDefinition<'src>, ()> {
        let keyword_token = self.expect_keyword("fragment")?;
        let name = self.expect_ast_name()?;
        if name.value == "on" {
            let mut error = GraphQLParseError::new(
                "fragment name cannot be `on`",
                name.span.clone(),
                GraphQLParseErrorKind::ReservedName {
                    name: "on".to_string(), context: ReservedNameContext::FragmentName,
                },
            );
            error.add_spec(
                "https://spec.graphql.org/October2021/#sec-Fragment-Name-Uniqueness",
            );
            self.record_error(error);
        }
        let type_condition = self.parse_type_condition()?;
        let directives = self.parse_directive_annotations()?;
        let selection_set = self.parse_selection_set()?;
        if self.config.retain_syntax {
            let span = self.make_span_ref(&keyword_token.span);
            Ok(ast::FragmentDefinition {
                description: None, directives, name, selection_set, span,
                syntax: Some(Box::new(ast::FragmentDefinitionSyntax {
                    fragment_keyword: keyword_token,
                })),
                type_condition,
            })
        } else {
            let span = self.make_span(keyword_token.span);
            Ok(ast::FragmentDefinition {
                description: None, directives, name, selection_set, span,
                syntax: None, type_condition,
            })
        }
    }

    /// Parses a type condition: `on TypeName`
    fn parse_type_condition(&mut self) -> Result<ast::TypeCondition<'src>, ()> {
        let on_token = self.expect_keyword("on")?;
        let named_type = self.expect_ast_name()?;
        if self.config.retain_syntax {
            let span = GraphQLSourceSpan::new(
                on_token.span.start_inclusive,
                named_type.span.end_exclusive,
            );
            Ok(ast::TypeCondition {
                named_type, span,
                syntax: Some(Box::new(ast::TypeConditionSyntax {
                    on_keyword: on_token,
                })),
            })
        } else {
            let span = GraphQLSourceSpan::new(
                on_token.span.start_inclusive, named_type.span.end_exclusive,
            );
            Ok(ast::TypeCondition { named_type, span, syntax: None })
        }
    }

    // =========================================================================
    // Type definition parsing
    // =========================================================================

    /// Parses an optional description, returning an
    /// `ast::StringValue` with the span moved from the
    /// consumed token.
    fn parse_ast_description(&mut self) -> Option<ast::StringValue<'src>> {
        if let Some(token) = self.token_stream.peek()
            && matches!(&token.kind, GraphQLTokenKind::StringValue(_)) {
            let is_block = match &token.kind {
                GraphQLTokenKind::StringValue(raw) => {
                    raw.starts_with("\"\"\"")
                },
                _ => false,
            };
            let token = self.consume_token().unwrap();
            match token.kind.parse_string_value() {
                Some(Ok(parsed)) => {
                    if self.config.retain_syntax {
                        let span = token.span.clone();
                        return Some(ast::StringValue {
                            is_block,
                            span,
                            syntax: Some(Box::new(
                                ast::StringValueSyntax { token },
                            )),
                            value: Cow::Owned(parsed),
                        });
                    } else {
                        return Some(ast::StringValue {
                            is_block,
                            span: token.span,
                            syntax: None,
                            value: Cow::Owned(parsed),
                        });
                    }
                },
                Some(Err(err)) => {
                    self.record_error(GraphQLParseError::new(
                        format!("invalid string in description: {err}"),
                        token.span,
                        GraphQLParseErrorKind::InvalidSyntax,
                    ));
                },
                None => unreachable!(),
            }
        }
        None
    }

    /// Parses a schema definition: `schema @directives { query: Query, ... }`
    fn parse_schema_definition(
        &mut self,
        description: Option<ast::StringValue<'src>>,
    ) -> Result<ast::SchemaDefinition<'src>, ()> {
        let keyword_token = self.expect_keyword("schema")?;
        let directives = self.parse_const_directive_annotations()?;
        let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
        self.push_delimiter(open_token.span.clone(), DelimiterContext::SchemaDefinition);
        let mut root_operations = Vec::new();
        loop {
            if self.peek_is(&GraphQLTokenKind::CurlyBraceClose) {
                break;
            }
            if self.token_stream.is_at_end() {
                self.handle_unclosed_brace();
                return Err(());
            }
            let op_name = self.expect_ast_name()?;
            let op_kind = match &*op_name.value {
                "query" => ast::OperationKind::Query,
                "mutation" => ast::OperationKind::Mutation,
                "subscription" => ast::OperationKind::Subscription,
                _ => {
                    self.record_error(GraphQLParseError::new(
                        format!(
                            "unknown operation type `{}`; expected `query`, `mutation`, \
                             or `subscription`",
                            op_name.value,
                        ),
                        op_name.span.clone(),
                        GraphQLParseErrorKind::InvalidSyntax,
                    ));
                    continue;
                },
            };
            let colon_token = self.expect(&GraphQLTokenKind::Colon)?;
            let named_type = self.expect_ast_name()?;
            let root_span = GraphQLSourceSpan::new(
                op_name.span.start_inclusive,
                named_type.span.end_exclusive,
            );
            let root_syntax = if self.config.retain_syntax {
                Some(Box::new(ast::RootOperationTypeDefinitionSyntax {
                    colon: colon_token,
                }))
            } else {
                None
            };
            root_operations.push(ast::RootOperationTypeDefinition {
                named_type, operation_kind: op_kind, span: root_span, syntax: root_syntax,
            });
        }
        let close_token = self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
        self.pop_delimiter();
        if self.config.retain_syntax {
            let span = self.make_span_ref(&keyword_token.span);
            Ok(ast::SchemaDefinition {
                description, directives, root_operations, span,
                syntax: Some(Box::new(ast::SchemaDefinitionSyntax {
                    braces: ast::DelimiterPair { close: close_token, open: open_token },
                    schema_keyword: keyword_token,
                })),
            })
        } else {
            let span = self.make_span(keyword_token.span);
            Ok(ast::SchemaDefinition {
                description, directives, root_operations, span, syntax: None,
            })
        }
    }

    /// Parses a scalar type definition: `scalar Name @directives`
    fn parse_scalar_type_definition(
        &mut self,
        description: Option<ast::StringValue<'src>>,
    ) -> Result<ast::TypeDefinition<'src>, ()> {
        let keyword_token = self.expect_keyword("scalar")?;
        let name = self.expect_ast_name()?;
        let directives = self.parse_const_directive_annotations()?;
        if self.config.retain_syntax {
            let span = self.make_span_ref(&keyword_token.span);
            Ok(ast::TypeDefinition::Scalar(ast::ScalarTypeDefinition {
                description, directives, name, span,
                syntax: Some(Box::new(ast::ScalarTypeDefinitionSyntax {
                    scalar_keyword: keyword_token,
                })),
            }))
        } else {
            let span = self.make_span(keyword_token.span);
            Ok(ast::TypeDefinition::Scalar(ast::ScalarTypeDefinition {
                description, directives, name, span, syntax: None,
            }))
        }
    }

    /// Parses an object type definition: `type Name implements I & J
    /// @directives { fields }`
    fn parse_object_type_definition(
        &mut self,
        description: Option<ast::StringValue<'src>>,
    ) -> Result<ast::TypeDefinition<'src>, ()> {
        let keyword_token = self.expect_keyword("type")?;
        let name = self.expect_ast_name()?;
        let (implements_tokens, implements) = if self.peek_is_keyword("implements") {
            let clause = self.parse_ast_implements_interfaces()?;
            (Some(ImplementsClauseTokens {
                ampersands: clause.ampersands,
                implements_keyword: clause.implements_keyword,
                leading_ampersand: clause.leading_ampersand,
            }), clause.interfaces)
        } else {
            (None, Vec::new())
        };
        let directives = self.parse_const_directive_annotations()?;
        let (fields, field_delimiters) = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_ast_fields_definition(DelimiterContext::ObjectTypeDefinition)?
        } else {
            (Vec::new(), None)
        };
        if self.config.retain_syntax {
            let span = self.make_span_ref(&keyword_token.span);
            let (impl_kw, leading_amp, amps) = match implements_tokens {
                Some(c) => (Some(c.implements_keyword), c.leading_ampersand, c.ampersands),
                None => (None, None, Vec::new()),
            };
            Ok(ast::TypeDefinition::Object(ast::ObjectTypeDefinition {
                description, directives, fields, implements, name, span,
                syntax: Some(Box::new(ast::ObjectTypeDefinitionSyntax {
                    ampersands: amps, braces: field_delimiters,
                    implements_keyword: impl_kw, leading_ampersand: leading_amp,
                    type_keyword: keyword_token,
                })),
            }))
        } else {
            let span = self.make_span(keyword_token.span);
            Ok(ast::TypeDefinition::Object(ast::ObjectTypeDefinition {
                description, directives, fields, implements, name, span, syntax: None,
            }))
        }
    }

    /// Parses an interface type definition.
    fn parse_interface_type_definition(
        &mut self,
        description: Option<ast::StringValue<'src>>,
    ) -> Result<ast::TypeDefinition<'src>, ()> {
        let keyword_token = self.expect_keyword("interface")?;
        let name = self.expect_ast_name()?;
        let (implements_tokens, implements) = if self.peek_is_keyword("implements") {
            let clause = self.parse_ast_implements_interfaces()?;
            (Some(ImplementsClauseTokens {
                ampersands: clause.ampersands,
                implements_keyword: clause.implements_keyword,
                leading_ampersand: clause.leading_ampersand,
            }), clause.interfaces)
        } else {
            (None, Vec::new())
        };
        let directives = self.parse_const_directive_annotations()?;
        let (fields, field_delimiters) = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_ast_fields_definition(DelimiterContext::InterfaceDefinition)?
        } else {
            (Vec::new(), None)
        };
        if self.config.retain_syntax {
            let span = self.make_span_ref(&keyword_token.span);
            let (impl_kw, leading_amp, amps) = match implements_tokens {
                Some(c) => (Some(c.implements_keyword), c.leading_ampersand, c.ampersands),
                None => (None, None, Vec::new()),
            };
            Ok(ast::TypeDefinition::Interface(ast::InterfaceTypeDefinition {
                description, directives, fields, implements, name, span,
                syntax: Some(Box::new(ast::InterfaceTypeDefinitionSyntax {
                    ampersands: amps, braces: field_delimiters,
                    implements_keyword: impl_kw, interface_keyword: keyword_token,
                    leading_ampersand: leading_amp,
                })),
            }))
        } else {
            let span = self.make_span(keyword_token.span);
            Ok(ast::TypeDefinition::Interface(ast::InterfaceTypeDefinition {
                description, directives, fields, implements, name, span, syntax: None,
            }))
        }
    }

    /// Parses a union type definition: `union Name @directives = A | B | C`
    fn parse_union_type_definition(
        &mut self,
        description: Option<ast::StringValue<'src>>,
    ) -> Result<ast::TypeDefinition<'src>, ()> {
        let keyword_token = self.expect_keyword("union")?;
        let name = self.expect_ast_name()?;
        let directives = self.parse_const_directive_annotations()?;
        let mut members = Vec::new();
        let mut equals_token = None;
        let mut leading_pipe = None;
        let mut pipes = Vec::new();
        if self.peek_is(&GraphQLTokenKind::Equals) {
            equals_token = Some(self.consume_token().unwrap());
            // Optional leading |
            if self.peek_is(&GraphQLTokenKind::Pipe) {
                leading_pipe = Some(self.consume_token().unwrap());
            }
            members.push(self.expect_ast_name()?);
            while self.peek_is(&GraphQLTokenKind::Pipe) {
                pipes.push(self.consume_token().unwrap());
                members.push(self.expect_ast_name()?);
            }
        }
        if self.config.retain_syntax {
            let span = self.make_span_ref(&keyword_token.span);
            Ok(ast::TypeDefinition::Union(ast::UnionTypeDefinition {
                description, directives, members, name, span,
                syntax: Some(Box::new(ast::UnionTypeDefinitionSyntax {
                    equals: equals_token, leading_pipe, pipes,
                    union_keyword: keyword_token,
                })),
            }))
        } else {
            let span = self.make_span(keyword_token.span);
            Ok(ast::TypeDefinition::Union(ast::UnionTypeDefinition {
                description, directives, members, name, span, syntax: None,
            }))
        }
    }

    /// Parses an enum type definition: `enum Name @directives { VALUES }`
    fn parse_enum_type_definition(
        &mut self,
        description: Option<ast::StringValue<'src>>,
    ) -> Result<ast::TypeDefinition<'src>, ()> {
        let keyword_token = self.expect_keyword("enum")?;
        let name = self.expect_ast_name()?;
        let directives = self.parse_const_directive_annotations()?;
        let (values, value_delimiters) = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_enum_values_definition()?
        } else {
            (Vec::new(), None)
        };
        if self.config.retain_syntax {
            let span = self.make_span_ref(&keyword_token.span);
            Ok(ast::TypeDefinition::Enum(ast::EnumTypeDefinition {
                description, directives, name, span,
                syntax: Some(Box::new(ast::EnumTypeDefinitionSyntax {
                    braces: value_delimiters, enum_keyword: keyword_token,
                })),
                values,
            }))
        } else {
            let span = self.make_span(keyword_token.span);
            Ok(ast::TypeDefinition::Enum(ast::EnumTypeDefinition {
                description, directives, name, span, syntax: None, values,
            }))
        }
    }

    /// Parses an input object type definition.
    fn parse_input_object_type_definition(
        &mut self,
        description: Option<ast::StringValue<'src>>,
    ) -> Result<ast::TypeDefinition<'src>, ()> {
        let keyword_token = self.expect_keyword("input")?;
        let name = self.expect_ast_name()?;
        let directives = self.parse_const_directive_annotations()?;
        let (fields, field_delimiters) = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_input_fields_definition()?
        } else {
            (Vec::new(), None)
        };
        if self.config.retain_syntax {
            let span = self.make_span_ref(&keyword_token.span);
            Ok(ast::TypeDefinition::InputObject(ast::InputObjectTypeDefinition {
                description, directives, fields, name, span,
                syntax: Some(Box::new(ast::InputObjectTypeDefinitionSyntax {
                    braces: field_delimiters, input_keyword: keyword_token,
                })),
            }))
        } else {
            let span = self.make_span(keyword_token.span);
            Ok(ast::TypeDefinition::InputObject(ast::InputObjectTypeDefinition {
                description, directives, fields, name, span, syntax: None,
            }))
        }
    }

    /// Parses a directive definition.
    fn parse_directive_definition(
        &mut self,
        description: Option<ast::StringValue<'src>>,
    ) -> Result<ast::DirectiveDefinition<'src>, ()> {
        let keyword_token = self.expect_keyword("directive")?;
        let at_token = self.expect(&GraphQLTokenKind::At)?;
        let name = self.expect_ast_name()?;
        let (arguments, argument_delimiters) =
            if self.peek_is(&GraphQLTokenKind::ParenOpen) {
                self.parse_arguments_definition()?
            } else {
                (Vec::new(), None)
            };
        let repeatable_token = if self.peek_is_keyword("repeatable") {
            Some(self.consume_token().unwrap())
        } else {
            None
        };
        let repeatable = repeatable_token.is_some();
        let on_token = self.expect_keyword("on")?;
        let locations = self.parse_directive_locations()?;
        if self.config.retain_syntax {
            let span = self.make_span_ref(&keyword_token.span);
            Ok(ast::DirectiveDefinition {
                arguments, description, locations, name, repeatable, span,
                syntax: Some(Box::new(ast::DirectiveDefinitionSyntax {
                    argument_parens: argument_delimiters, at_sign: at_token,
                    directive_keyword: keyword_token, on_keyword: on_token,
                    repeatable_keyword: repeatable_token,
                })),
            })
        } else {
            let span = self.make_span(keyword_token.span);
            Ok(ast::DirectiveDefinition {
                arguments, description, locations, name, repeatable, span, syntax: None,
            })
        }
    }

    /// Parses implements interfaces: `implements A & B & C`
    fn parse_ast_implements_interfaces(&mut self) -> Result<ImplementsClause<'src>, ()> {
        let implements_keyword = self.expect_keyword("implements")?;
        // Optional leading &
        let leading_ampersand = if self.peek_is(&GraphQLTokenKind::Ampersand) {
            Some(self.consume_token().unwrap())
        } else {
            None
        };
        let mut interfaces = Vec::new();
        let mut ampersands = Vec::new();
        interfaces.push(self.expect_ast_name()?);
        while self.peek_is(&GraphQLTokenKind::Ampersand) {
            ampersands.push(self.consume_token().unwrap());
            interfaces.push(self.expect_ast_name()?);
        }
        Ok(ImplementsClause {
            ampersands, implements_keyword, interfaces, leading_ampersand,
        })
    }

    /// Parses field definitions: `{ field: Type, ... }`
    fn parse_ast_fields_definition(
        &mut self,
        context: DelimiterContext,
    ) -> Result<(Vec<ast::FieldDefinition<'src>>, Option<ast::DelimiterPair<'src>>), ()> {
        let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
        self.push_delimiter(open_token.span.clone(), context);
        let mut fields = Vec::new();
        loop {
            if self.peek_is(&GraphQLTokenKind::CurlyBraceClose) {
                break;
            }
            if self.token_stream.is_at_end() {
                self.handle_unclosed_brace();
                return Err(());
            }
            fields.push(self.parse_field_definition()?);
        }
        let close_token = self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
        self.pop_delimiter();
        let delimiters = if self.config.retain_syntax {
            Some(ast::DelimiterPair { close: close_token, open: open_token })
        } else {
            None
        };
        Ok((fields, delimiters))
    }

    /// Parses a single field definition.
    fn parse_field_definition(&mut self) -> Result<ast::FieldDefinition<'src>, ()> {
        let description = self.parse_ast_description();
        let name = self.expect_ast_name()?;
        let (arguments, argument_delimiters) =
            if self.peek_is(&GraphQLTokenKind::ParenOpen) {
                self.parse_arguments_definition()?
            } else {
                (Vec::new(), None)
            };
        let colon_token = self.expect(&GraphQLTokenKind::Colon)?;
        let field_type = self.parse_type_annotation()?;
        let directives = self.parse_const_directive_annotations()?;
        let span = GraphQLSourceSpan::new(
            name.span.start_inclusive,
            self.last_end_position.unwrap_or(name.span.end_exclusive),
        );
        let syntax = if self.config.retain_syntax {
            Some(Box::new(ast::FieldDefinitionSyntax {
                argument_parens: argument_delimiters, colon: colon_token,
            }))
        } else {
            None
        };
        Ok(ast::FieldDefinition {
            arguments, description, directives, field_type, name, span, syntax,
        })
    }

    /// Parses argument definitions: `(arg: Type = default, ...)`
    fn parse_arguments_definition(
        &mut self,
    ) -> Result<(Vec<ast::InputValueDefinition<'src>>, Option<ast::DelimiterPair<'src>>), ()> {
        let open_token = self.expect(&GraphQLTokenKind::ParenOpen)?;
        self.push_delimiter(open_token.span.clone(), DelimiterContext::ArgumentDefinitions);
        let mut arguments = Vec::new();
        loop {
            if self.peek_is(&GraphQLTokenKind::ParenClose) {
                break;
            }
            if self.token_stream.is_at_end() {
                self.handle_unclosed_paren();
                return Err(());
            }
            arguments.push(self.parse_input_value_definition()?);
        }
        let close_token = self.expect(&GraphQLTokenKind::ParenClose)?;
        self.pop_delimiter();
        let delimiters = if self.config.retain_syntax {
            Some(ast::DelimiterPair { close: close_token, open: open_token })
        } else {
            None
        };
        Ok((arguments, delimiters))
    }

    /// Parses input fields definition (for input objects).
    fn parse_input_fields_definition(
        &mut self,
    ) -> Result<(Vec<ast::InputValueDefinition<'src>>, Option<ast::DelimiterPair<'src>>), ()> {
        let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
        self.push_delimiter(open_token.span.clone(), DelimiterContext::InputObjectDefinition);
        let mut fields = Vec::new();
        loop {
            if self.peek_is(&GraphQLTokenKind::CurlyBraceClose) {
                break;
            }
            if self.token_stream.is_at_end() {
                self.handle_unclosed_brace();
                return Err(());
            }
            fields.push(self.parse_input_value_definition()?);
        }
        let close_token = self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
        self.pop_delimiter();
        let delimiters = if self.config.retain_syntax {
            Some(ast::DelimiterPair { close: close_token, open: open_token })
        } else {
            None
        };
        Ok((fields, delimiters))
    }

    /// Parses an input value definition (used for arguments and input fields).
    fn parse_input_value_definition(&mut self) -> Result<ast::InputValueDefinition<'src>, ()> {
        let description = self.parse_ast_description();
        let name = self.expect_ast_name()?;
        let colon_token = self.expect(&GraphQLTokenKind::Colon)?;
        let value_type = self.parse_type_annotation()?;
        let (default_value, equals_token) = if self.peek_is(&GraphQLTokenKind::Equals) {
            let eq = self.consume_token().unwrap();
            (Some(self.parse_value(ConstContext::InputDefaultValue)?), Some(eq))
        } else {
            (None, None)
        };
        let directives = self.parse_const_directive_annotations()?;
        let span = GraphQLSourceSpan::new(
            name.span.start_inclusive,
            self.last_end_position.unwrap_or(name.span.end_exclusive),
        );
        let syntax = if self.config.retain_syntax {
            Some(Box::new(ast::InputValueDefinitionSyntax {
                colon: colon_token, equals: equals_token,
            }))
        } else {
            None
        };
        Ok(ast::InputValueDefinition {
            default_value, description, directives, name, span, syntax, value_type,
        })
    }

    /// Parses enum value definitions.
    fn parse_enum_values_definition(
        &mut self,
    ) -> Result<(Vec<ast::EnumValueDefinition<'src>>, Option<ast::DelimiterPair<'src>>), ()> {
        let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
        self.push_delimiter(open_token.span.clone(), DelimiterContext::EnumDefinition);
        let mut values = Vec::new();
        loop {
            if self.peek_is(&GraphQLTokenKind::CurlyBraceClose) {
                break;
            }
            if self.token_stream.is_at_end() {
                self.handle_unclosed_brace();
                return Err(());
            }
            values.push(self.parse_enum_value_definition()?);
        }
        let close_token = self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
        self.pop_delimiter();
        let delimiters = if self.config.retain_syntax {
            Some(ast::DelimiterPair { close: close_token, open: open_token })
        } else {
            None
        };
        Ok((values, delimiters))
    }

    /// Parses a single enum value definition.
    fn parse_enum_value_definition(&mut self) -> Result<ast::EnumValueDefinition<'src>, ()> {
        let description = self.parse_ast_description();
        let name = self.expect_ast_name()?;
        if matches!(&*name.value, "true" | "false" | "null") {
            let mut error = GraphQLParseError::new(
                format!("enum value cannot be `{}`", name.value),
                name.span.clone(),
                GraphQLParseErrorKind::ReservedName {
                    name: name.value.clone().into_owned(),
                    context: ReservedNameContext::EnumValue,
                },
            );
            error.add_spec(
                "https://spec.graphql.org/October2021/#sec-Enum-Value-Uniqueness",
            );
            self.record_error(error);
        }
        let directives = self.parse_const_directive_annotations()?;
        let span = GraphQLSourceSpan::new(
            name.span.start_inclusive,
            self.last_end_position.unwrap_or(name.span.end_exclusive),
        );
        Ok(ast::EnumValueDefinition { description, directives, name, span })
    }

    /// Parses directive locations: `FIELD | OBJECT | ...`
    fn parse_directive_locations(&mut self) -> Result<Vec<ast::DirectiveLocation<'src>>, ()> {
        // Optional leading |
        let leading_pipe = if self.peek_is(&GraphQLTokenKind::Pipe) {
            Some(self.consume_token().unwrap())
        } else {
            None
        };

        let mut locations = Vec::new();
        locations.push(self.parse_directive_location(leading_pipe)?);

        while self.peek_is(&GraphQLTokenKind::Pipe) {
            let pipe = self.consume_token().unwrap();
            locations.push(self.parse_directive_location(Some(pipe))?);
        }

        Ok(locations)
    }

    /// Parses a single directive location.
    fn parse_directive_location(
        &mut self, pipe: Option<GraphQLToken<'src>>,
    ) -> Result<ast::DirectiveLocation<'src>, ()> {
        let name = self.expect_ast_name()?;
        let kind = match &*name.value {
            // Executable locations
            "QUERY" => ast::DirectiveLocationKind::Query,
            "MUTATION" => ast::DirectiveLocationKind::Mutation,
            "SUBSCRIPTION" => ast::DirectiveLocationKind::Subscription,
            "FIELD" => ast::DirectiveLocationKind::Field,
            "FRAGMENT_DEFINITION" => ast::DirectiveLocationKind::FragmentDefinition,
            "FRAGMENT_SPREAD" => ast::DirectiveLocationKind::FragmentSpread,
            "INLINE_FRAGMENT" => ast::DirectiveLocationKind::InlineFragment,
            "VARIABLE_DEFINITION" => ast::DirectiveLocationKind::VariableDefinition,
            // Type system locations
            "SCHEMA" => ast::DirectiveLocationKind::Schema,
            "SCALAR" => ast::DirectiveLocationKind::Scalar,
            "OBJECT" => ast::DirectiveLocationKind::Object,
            "FIELD_DEFINITION" => ast::DirectiveLocationKind::FieldDefinition,
            "ARGUMENT_DEFINITION" => ast::DirectiveLocationKind::ArgumentDefinition,
            "INTERFACE" => ast::DirectiveLocationKind::Interface,
            "UNION" => ast::DirectiveLocationKind::Union,
            "ENUM" => ast::DirectiveLocationKind::Enum,
            "ENUM_VALUE" => ast::DirectiveLocationKind::EnumValue,
            "INPUT_OBJECT" => ast::DirectiveLocationKind::InputObject,
            "INPUT_FIELD_DEFINITION" => ast::DirectiveLocationKind::InputFieldDefinition,
            _ => {
                let mut error = GraphQLParseError::new(
                    format!("unknown directive location `{}`", name.value),
                    name.span.clone(),
                    GraphQLParseErrorKind::InvalidSyntax,
                );
                if let Some(suggestion) = Self::suggest_directive_location(&name.value) {
                    error.add_help(format!("did you mean `{suggestion}`?"));
                }
                self.record_error(error);
                return Err(());
            },
        };
        let syntax = if self.config.retain_syntax {
            Some(Box::new(ast::DirectiveLocationSyntax {
                pipe,
                token: name.syntax.unwrap().token,
            }))
        } else {
            None
        };
        Ok(ast::DirectiveLocation { kind, span: name.span, syntax })
    }

    /// Suggests the closest directive location for a typo.
    fn suggest_directive_location(input: &str) -> Option<&'static str> {
        const LOCATIONS: &[&str] = &[
            "QUERY",
            "MUTATION",
            "SUBSCRIPTION",
            "FIELD",
            "FRAGMENT_DEFINITION",
            "FRAGMENT_SPREAD",
            "INLINE_FRAGMENT",
            "VARIABLE_DEFINITION",
            "SCHEMA",
            "SCALAR",
            "OBJECT",
            "FIELD_DEFINITION",
            "ARGUMENT_DEFINITION",
            "INTERFACE",
            "UNION",
            "ENUM",
            "ENUM_VALUE",
            "INPUT_OBJECT",
            "INPUT_FIELD_DEFINITION",
        ];

        // Simple edit distance for suggestions
        let input_upper = input.to_uppercase();
        let mut best_match: Option<&'static str> = None;
        let mut best_distance = usize::MAX;

        for &location in LOCATIONS {
            let distance = Self::edit_distance(&input_upper, location);
            if distance < best_distance && distance <= 3 {
                best_distance = distance;
                best_match = Some(location);
            }
        }

        best_match
    }

    /// Simple Levenshtein edit distance.
    fn edit_distance(a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let m = a_chars.len();
        let n = b_chars.len();

        if m == 0 {
            return n;
        }
        if n == 0 {
            return m;
        }

        let mut prev: Vec<usize> = (0..=n).collect();
        let mut curr = vec![0; n + 1];

        for i in 1..=m {
            curr[0] = i;
            for j in 1..=n {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                curr[j] = (prev[j] + 1)
                    .min(curr[j - 1] + 1)
                    .min(prev[j - 1] + cost);
            }
            std::mem::swap(&mut prev, &mut curr);
        }

        prev[n]
    }

    // =========================================================================
    // Type extension parsing
    // =========================================================================

    /// Parses a type extension.
    ///
    /// Parses a type or schema extension: `extend <keyword> ...`
    fn parse_type_extension(&mut self) -> Result<ast::Definition<'src>, ()> {
        let extend_token = self.expect_keyword("extend")?;
        if self.peek_is_keyword("schema") {
            self.parse_schema_extension(extend_token)
        } else if self.peek_is_keyword("scalar") {
            self.parse_scalar_type_extension(extend_token)
        } else if self.peek_is_keyword("type") {
            self.parse_object_type_extension(extend_token)
        } else if self.peek_is_keyword("interface") {
            self.parse_interface_type_extension(extend_token)
        } else if self.peek_is_keyword("union") {
            self.parse_union_type_extension(extend_token)
        } else if self.peek_is_keyword("enum") {
            self.parse_enum_type_extension(extend_token)
        } else if self.peek_is_keyword("input") {
            self.parse_input_object_type_extension(extend_token)
        } else {
            let span = self.token_stream.peek()
                .map(|t| t.span.clone())
                .unwrap_or_else(|| self.eof_span());
            let found = self.token_stream.peek()
                .map(|t| Self::token_kind_display(&t.kind))
                .unwrap_or_else(|| "end of input".to_string());
            self.record_error(GraphQLParseError::new(
                format!(
                    "expected type extension keyword (`schema`, `scalar`, `type`, \
                     `interface`, `union`, `enum`, `input`), found `{found}`"
                ),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec![
                        "schema".to_string(),
                        "scalar".to_string(),
                        "type".to_string(),
                        "interface".to_string(),
                        "union".to_string(),
                        "enum".to_string(),
                        "input".to_string(),
                    ],
                    found,
                },
            ));
            Err(())
        }
    }

    /// Parses a schema extension: `extend schema @directives { query: Query }`
    fn parse_schema_extension(
        &mut self,
        extend_token: GraphQLToken<'src>,
    ) -> Result<ast::Definition<'src>, ()> {
        let schema_token = self.expect_keyword("schema")?;
        let directives = self.parse_const_directive_annotations()?;
        let mut root_operations = Vec::new();
        let mut braces = None;
        if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
            self.push_delimiter(open_token.span.clone(), DelimiterContext::SchemaDefinition);
            loop {
                if self.peek_is(&GraphQLTokenKind::CurlyBraceClose) {
                    break;
                }
                if self.token_stream.is_at_end() {
                    self.handle_unclosed_brace();
                    return Err(());
                }
                let op_name = self.expect_ast_name()?;
                let op_kind = match &*op_name.value {
                    "query" => ast::OperationKind::Query,
                    "mutation" => ast::OperationKind::Mutation,
                    "subscription" => ast::OperationKind::Subscription,
                    _ => {
                        self.record_error(GraphQLParseError::new(
                            format!(
                                "unknown operation type `{}`; expected `query`, `mutation`, \
                                 or `subscription`",
                                op_name.value,
                            ),
                            op_name.span.clone(),
                            GraphQLParseErrorKind::InvalidSyntax,
                        ));
                        continue;
                    },
                };
                let colon_token = self.expect(&GraphQLTokenKind::Colon)?;
                let named_type = self.expect_ast_name()?;
                let root_span = GraphQLSourceSpan::new(
                    op_name.span.start_inclusive,
                    named_type.span.end_exclusive,
                );
                let root_syntax = if self.config.retain_syntax {
                    Some(Box::new(ast::RootOperationTypeDefinitionSyntax {
                        colon: colon_token,
                    }))
                } else {
                    None
                };
                root_operations.push(ast::RootOperationTypeDefinition {
                    named_type, operation_kind: op_kind, span: root_span, syntax: root_syntax,
                });
            }
            let close_token = self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
            self.pop_delimiter();
            if self.config.retain_syntax {
                braces = Some(ast::DelimiterPair { close: close_token, open: open_token });
            }
        }
        if self.config.retain_syntax {
            let span = self.make_span_ref(&extend_token.span);
            Ok(ast::Definition::SchemaExtension(ast::SchemaExtension {
                directives, root_operations, span,
                syntax: Some(Box::new(ast::SchemaExtensionSyntax {
                    braces, extend_keyword: extend_token, schema_keyword: schema_token,
                })),
            }))
        } else {
            let span = self.make_span(extend_token.span);
            Ok(ast::Definition::SchemaExtension(ast::SchemaExtension {
                directives, root_operations, span, syntax: None,
            }))
        }
    }

    /// Parses a scalar type extension: `extend scalar Name @directives`
    fn parse_scalar_type_extension(
        &mut self,
        extend_token: GraphQLToken<'src>,
    ) -> Result<ast::Definition<'src>, ()> {
        let scalar_token = self.expect_keyword("scalar")?;
        let name = self.expect_ast_name()?;
        let directives = self.parse_const_directive_annotations()?;
        if self.config.retain_syntax {
            let span = self.make_span_ref(&extend_token.span);
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::Scalar(
                ast::ScalarTypeExtension {
                    directives, name, span,
                    syntax: Some(Box::new(ast::ScalarTypeExtensionSyntax {
                        extend_keyword: extend_token, scalar_keyword: scalar_token,
                    })),
                },
            )))
        } else {
            let span = self.make_span(extend_token.span);
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::Scalar(
                ast::ScalarTypeExtension { directives, name, span, syntax: None },
            )))
        }
    }

    /// Parses an object type extension: `extend type Name implements I & J
    /// @directives { fields }`
    fn parse_object_type_extension(
        &mut self,
        extend_token: GraphQLToken<'src>,
    ) -> Result<ast::Definition<'src>, ()> {
        let type_token = self.expect_keyword("type")?;
        let name = self.expect_ast_name()?;
        let (implements_tokens, implements) = if self.peek_is_keyword("implements") {
            let clause = self.parse_ast_implements_interfaces()?;
            (Some(ImplementsClauseTokens {
                ampersands: clause.ampersands,
                implements_keyword: clause.implements_keyword,
                leading_ampersand: clause.leading_ampersand,
            }), clause.interfaces)
        } else {
            (None, Vec::new())
        };
        let directives = self.parse_const_directive_annotations()?;
        let (fields, field_delimiters) = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_ast_fields_definition(DelimiterContext::ObjectTypeDefinition)?
        } else {
            (Vec::new(), None)
        };
        if self.config.retain_syntax {
            let span = self.make_span_ref(&extend_token.span);
            let (impl_kw, leading_amp, amps) = match implements_tokens {
                Some(c) => (Some(c.implements_keyword), c.leading_ampersand, c.ampersands),
                None => (None, None, Vec::new()),
            };
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::Object(
                ast::ObjectTypeExtension {
                    directives, fields, implements, name, span,
                    syntax: Some(Box::new(ast::ObjectTypeExtensionSyntax {
                        ampersands: amps, braces: field_delimiters,
                        extend_keyword: extend_token, implements_keyword: impl_kw,
                        leading_ampersand: leading_amp, type_keyword: type_token,
                    })),
                },
            )))
        } else {
            let span = self.make_span(extend_token.span);
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::Object(
                ast::ObjectTypeExtension {
                    directives, fields, implements, name, span, syntax: None,
                },
            )))
        }
    }

    /// Parses an interface type extension.
    fn parse_interface_type_extension(
        &mut self,
        extend_token: GraphQLToken<'src>,
    ) -> Result<ast::Definition<'src>, ()> {
        let interface_token = self.expect_keyword("interface")?;
        let name = self.expect_ast_name()?;
        let (implements_tokens, implements) = if self.peek_is_keyword("implements") {
            let clause = self.parse_ast_implements_interfaces()?;
            (Some(ImplementsClauseTokens {
                ampersands: clause.ampersands,
                implements_keyword: clause.implements_keyword,
                leading_ampersand: clause.leading_ampersand,
            }), clause.interfaces)
        } else {
            (None, Vec::new())
        };
        let directives = self.parse_const_directive_annotations()?;
        let (fields, field_delimiters) = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_ast_fields_definition(DelimiterContext::InterfaceDefinition)?
        } else {
            (Vec::new(), None)
        };
        if self.config.retain_syntax {
            let span = self.make_span_ref(&extend_token.span);
            let (impl_kw, leading_amp, amps) = match implements_tokens {
                Some(c) => (Some(c.implements_keyword), c.leading_ampersand, c.ampersands),
                None => (None, None, Vec::new()),
            };
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::Interface(
                ast::InterfaceTypeExtension {
                    directives, fields, implements, name, span,
                    syntax: Some(Box::new(ast::InterfaceTypeExtensionSyntax {
                        ampersands: amps, braces: field_delimiters,
                        extend_keyword: extend_token, implements_keyword: impl_kw,
                        interface_keyword: interface_token, leading_ampersand: leading_amp,
                    })),
                },
            )))
        } else {
            let span = self.make_span(extend_token.span);
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::Interface(
                ast::InterfaceTypeExtension {
                    directives, fields, implements, name, span, syntax: None,
                },
            )))
        }
    }

    /// Parses a union type extension: `extend union Name @directives = A | B`
    fn parse_union_type_extension(
        &mut self,
        extend_token: GraphQLToken<'src>,
    ) -> Result<ast::Definition<'src>, ()> {
        let union_token = self.expect_keyword("union")?;
        let name = self.expect_ast_name()?;
        let directives = self.parse_const_directive_annotations()?;
        let mut members = Vec::new();
        let mut equals_token = None;
        let mut leading_pipe = None;
        let mut pipes = Vec::new();
        if self.peek_is(&GraphQLTokenKind::Equals) {
            equals_token = Some(self.consume_token().unwrap());
            if self.peek_is(&GraphQLTokenKind::Pipe) {
                leading_pipe = Some(self.consume_token().unwrap());
            }
            members.push(self.expect_ast_name()?);
            while self.peek_is(&GraphQLTokenKind::Pipe) {
                pipes.push(self.consume_token().unwrap());
                members.push(self.expect_ast_name()?);
            }
        }
        if self.config.retain_syntax {
            let span = self.make_span_ref(&extend_token.span);
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::Union(
                ast::UnionTypeExtension {
                    directives, members, name, span,
                    syntax: Some(Box::new(ast::UnionTypeExtensionSyntax {
                        equals: equals_token, extend_keyword: extend_token,
                        leading_pipe, pipes, union_keyword: union_token,
                    })),
                },
            )))
        } else {
            let span = self.make_span(extend_token.span);
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::Union(
                ast::UnionTypeExtension { directives, members, name, span, syntax: None },
            )))
        }
    }

    /// Parses an enum type extension: `extend enum Name @directives { VALUES }`
    fn parse_enum_type_extension(
        &mut self,
        extend_token: GraphQLToken<'src>,
    ) -> Result<ast::Definition<'src>, ()> {
        let enum_token = self.expect_keyword("enum")?;
        let name = self.expect_ast_name()?;
        let directives = self.parse_const_directive_annotations()?;
        let (values, value_delimiters) = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_enum_values_definition()?
        } else {
            (Vec::new(), None)
        };
        if self.config.retain_syntax {
            let span = self.make_span_ref(&extend_token.span);
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::Enum(
                ast::EnumTypeExtension {
                    directives, name, span,
                    syntax: Some(Box::new(ast::EnumTypeExtensionSyntax {
                        braces: value_delimiters, enum_keyword: enum_token,
                        extend_keyword: extend_token,
                    })),
                    values,
                },
            )))
        } else {
            let span = self.make_span(extend_token.span);
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::Enum(
                ast::EnumTypeExtension { directives, name, span, syntax: None, values },
            )))
        }
    }

    /// Parses an input object type extension.
    fn parse_input_object_type_extension(
        &mut self,
        extend_token: GraphQLToken<'src>,
    ) -> Result<ast::Definition<'src>, ()> {
        let input_token = self.expect_keyword("input")?;
        let name = self.expect_ast_name()?;
        let directives = self.parse_const_directive_annotations()?;
        let (fields, field_delimiters) = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_input_fields_definition()?
        } else {
            (Vec::new(), None)
        };
        if self.config.retain_syntax {
            let span = self.make_span_ref(&extend_token.span);
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::InputObject(
                ast::InputObjectTypeExtension {
                    directives, fields, name, span,
                    syntax: Some(Box::new(ast::InputObjectTypeExtensionSyntax {
                        braces: field_delimiters, extend_keyword: extend_token,
                        input_keyword: input_token,
                    })),
                },
            )))
        } else {
            let span = self.make_span(extend_token.span);
            Ok(ast::Definition::TypeExtension(ast::TypeExtension::InputObject(
                ast::InputObjectTypeExtension { directives, fields, name, span, syntax: None },
            )))
        }
    }

    // =========================================================================
    // Document parsing (public API)
    // =========================================================================

    /// Returns a span covering the entire document (byte 0 to last token end).
    fn document_span(&self) -> GraphQLSourceSpan {
        let start = SourcePosition::new(0, 0, Some(0), 0);
        let end = self.last_end_position.unwrap_or(start);
        GraphQLSourceSpan::new(start, end)
    }

    /// Parses a schema document (type system definitions only).
    pub fn parse_schema_document(mut self) -> ParseResult<ast::Document<'src>> {
        let mut definitions = Vec::new();
        while !self.token_stream.is_at_end() {
            match self.parse_schema_definition_item() {
                Ok(def) => definitions.push(def),
                Err(()) => self.recover_to_next_definition(),
            }
        }
        let span = self.document_span();
        let syntax = if self.config.retain_syntax {
            let trailing_trivia = self.token_stream.peek()
                .map(|eof| eof.preceding_trivia.to_vec())
                .unwrap_or_default();
            Some(Box::new(ast::DocumentSyntax { trailing_trivia }))
        } else {
            None
        };
        let document = ast::Document { definitions, span, syntax };
        if self.errors.is_empty() {
            ParseResult::ok(document)
        } else {
            ParseResult::recovered(document, self.errors)
        }
    }

    /// Parses an executable document (operations and fragments only).
    pub fn parse_executable_document(mut self) -> ParseResult<ast::Document<'src>> {
        let mut definitions = Vec::new();
        while !self.token_stream.is_at_end() {
            match self.parse_executable_definition_item() {
                Ok(def) => definitions.push(def),
                Err(()) => self.recover_to_next_definition(),
            }
        }
        let span = self.document_span();
        let syntax = if self.config.retain_syntax {
            let trailing_trivia = self.token_stream.peek()
                .map(|eof| eof.preceding_trivia.to_vec())
                .unwrap_or_default();
            Some(Box::new(ast::DocumentSyntax { trailing_trivia }))
        } else {
            None
        };
        let document = ast::Document { definitions, span, syntax };
        if self.errors.is_empty() {
            ParseResult::ok(document)
        } else {
            ParseResult::recovered(document, self.errors)
        }
    }

    /// Parses a mixed document (both type system and executable definitions).
    pub fn parse_mixed_document(mut self) -> ParseResult<ast::Document<'src>> {
        let mut definitions = Vec::new();
        while !self.token_stream.is_at_end() {
            match self.parse_mixed_definition_item() {
                Ok(def) => definitions.push(def),
                Err(()) => self.recover_to_next_definition(),
            }
        }
        let span = self.document_span();
        let syntax = if self.config.retain_syntax {
            let trailing_trivia = self.token_stream.peek()
                .map(|eof| eof.preceding_trivia.to_vec())
                .unwrap_or_default();
            Some(Box::new(ast::DocumentSyntax { trailing_trivia }))
        } else {
            None
        };
        let document = ast::Document { definitions, span, syntax };
        if self.errors.is_empty() {
            ParseResult::ok(document)
        } else {
            ParseResult::recovered(document, self.errors)
        }
    }

    /// Parses a single schema definition item.
    fn parse_schema_definition_item(&mut self) -> Result<ast::Definition<'src>, ()> {
        // Handle lexer errors
        if let Some(token) = self.token_stream.peek()
            && let GraphQLTokenKind::Error(_) = &token.kind {
                let token = token.clone();
                self.handle_lexer_error(&token);
                self.consume_token();
                return Err(());
            }

        let description = self.parse_ast_description();

        if self.peek_is_keyword("schema") {
            Ok(ast::Definition::SchemaDefinition(self.parse_schema_definition(description)?))
        } else if self.peek_is_keyword("scalar") {
            Ok(ast::Definition::TypeDefinition(self.parse_scalar_type_definition(description)?))
        } else if self.peek_is_keyword("type") {
            Ok(ast::Definition::TypeDefinition(self.parse_object_type_definition(description)?))
        } else if self.peek_is_keyword("interface") {
            Ok(ast::Definition::TypeDefinition(
                self.parse_interface_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("union") {
            Ok(ast::Definition::TypeDefinition(self.parse_union_type_definition(description)?))
        } else if self.peek_is_keyword("enum") {
            Ok(ast::Definition::TypeDefinition(self.parse_enum_type_definition(description)?))
        } else if self.peek_is_keyword("input") {
            Ok(ast::Definition::TypeDefinition(
                self.parse_input_object_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("directive") {
            Ok(ast::Definition::DirectiveDefinition(
                self.parse_directive_definition(description)?,
            ))
        } else if self.peek_is_keyword("extend") {
            self.parse_type_extension()
        } else if self.peek_is_keyword("query")
            || self.peek_is_keyword("mutation")
            || self.peek_is_keyword("subscription")
            || self.peek_is_keyword("fragment")
            || self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            // Executable definition in schema document - record error
            let span = self
                .token_stream.peek()
                .map(|t| t.span.clone())
                .unwrap_or_else(|| self.eof_span());
            let kind = if self.peek_is_keyword("fragment") {
                DefinitionKind::Fragment
            } else {
                DefinitionKind::Operation
            };
            self.record_error(GraphQLParseError::new(
                format!(
                    "{} not allowed in schema document",
                    match kind {
                        DefinitionKind::Fragment => "fragment definition",
                        DefinitionKind::Operation => "operation definition",
                        _ => "definition",
                    }
                ),
                span,
                GraphQLParseErrorKind::WrongDocumentKind {
                    found: kind,
                    document_kind: DocumentKind::Schema,
                },
            ));
            // Consume the token to ensure forward progress during error
            // recovery. Without this, recovery sees `fragment`/`query`/etc.
            // as a definition start and breaks without consuming, causing
            // an infinite loop.
            self.consume_token();
            Err(())
        } else {
            let span = self
                .token_stream.peek()
                .map(|t| t.span.clone())
                .unwrap_or_else(|| self.eof_span());
            let found = self
                .token_stream.peek()
                .map(|t| Self::token_kind_display(&t.kind))
                .unwrap_or_else(|| "end of input".to_string());
            // Consume the token to ensure forward progress during error
            // recovery. Without this, recovery sees the unconsumed token
            // as a potential definition start and stops immediately,
            // causing an infinite loop.
            self.consume_token();
            self.record_error(GraphQLParseError::new(
                format!("expected schema definition, found `{found}`"),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec![
                        "type".to_string(),
                        "interface".to_string(),
                        "union".to_string(),
                        "enum".to_string(),
                        "scalar".to_string(),
                        "input".to_string(),
                        "directive".to_string(),
                        "schema".to_string(),
                        "extend".to_string(),
                    ],
                    found,
                },
            ));
            Err(())
        }
    }

    /// Parses a single executable definition item.
    fn parse_executable_definition_item(&mut self) -> Result<ast::Definition<'src>, ()> {
        // Handle lexer errors
        if let Some(token) = self.token_stream.peek()
            && let GraphQLTokenKind::Error(_) = &token.kind {
                let token = token.clone();
                self.handle_lexer_error(&token);
                self.consume_token();
                return Err(());
            }

        if self.peek_is_keyword("query")
            || self.peek_is_keyword("mutation")
            || self.peek_is_keyword("subscription")
            || self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            Ok(ast::Definition::OperationDefinition(self.parse_operation_definition()?))
        } else if self.peek_is_keyword("fragment") {
            Ok(ast::Definition::FragmentDefinition(self.parse_fragment_definition()?))
        } else if self.peek_is_keyword("type")
            || self.peek_is_keyword("interface")
            || self.peek_is_keyword("union")
            || self.peek_is_keyword("enum")
            || self.peek_is_keyword("scalar")
            || self.peek_is_keyword("input")
            || self.peek_is_keyword("directive")
            || self.peek_is_keyword("schema")
            || self.peek_is_keyword("extend") {
            // Schema definition in executable document - record error
            let span = self
                .token_stream.peek()
                .map(|t| t.span.clone())
                .unwrap_or_else(|| self.eof_span());
            let kind = if self.peek_is_keyword("directive") {
                DefinitionKind::DirectiveDefinition
            } else if self.peek_is_keyword("schema") || self.peek_is_keyword("extend") {
                DefinitionKind::Schema
            } else {
                DefinitionKind::TypeDefinition
            };
            self.consume_token();
            self.record_error(GraphQLParseError::new(
                format!(
                    "{} not allowed in executable document",
                    match kind {
                        DefinitionKind::TypeDefinition => "type definition",
                        DefinitionKind::DirectiveDefinition => "directive definition",
                        DefinitionKind::Schema => "schema definition",
                        _ => "definition",
                    }
                ),
                span,
                GraphQLParseErrorKind::WrongDocumentKind {
                    found: kind,
                    document_kind: DocumentKind::Executable,
                },
            ));
            Err(())
        } else {
            // Check for description followed by type definition (common mistake)
            // Extract info from first peek before taking second peek to avoid
            // double borrow.
            let first_is_string = self
                .token_stream.peek()
                .map(|t| matches!(&t.kind, GraphQLTokenKind::StringValue(_)))
                .unwrap_or(false);

            if first_is_string {
                // Might be a description - peek ahead to check for type keyword
                let is_type_def = self.token_stream.peek_nth(1).is_some_and(|next| {
                    if let GraphQLTokenKind::Name(name) = &next.kind {
                        matches!(
                            name.as_ref(),
                            "type"
                                | "interface"
                                | "union"
                                | "enum"
                                | "scalar"
                                | "input"
                                | "directive"
                                | "schema"
                                | "extend"
                        )
                    } else {
                        false
                    }
                });

                if is_type_def {
                    let span = self
                        .token_stream.peek()
                        .map(|t| t.span.clone())
                        .unwrap_or_else(|| self.eof_span());
                    self.consume_token();
                    self.record_error(GraphQLParseError::new(
                        "type definition not allowed in executable document",
                        span,
                        GraphQLParseErrorKind::WrongDocumentKind {
                            found: DefinitionKind::TypeDefinition,
                            document_kind: DocumentKind::Executable,
                        },
                    ));
                    return Err(());
                }
            }

            let span = self
                .token_stream.peek()
                .map(|t| t.span.clone())
                .unwrap_or_else(|| self.eof_span());
            let found = self
                .token_stream.peek()
                .map(|t| Self::token_kind_display(&t.kind))
                .unwrap_or_else(|| "end of input".to_string());
            // Consume the token to ensure forward progress during error
            // recovery. Without this, recovery sees the unconsumed token
            // as a potential definition start and stops immediately,
            // causing an infinite loop.
            self.consume_token();
            self.record_error(GraphQLParseError::new(
                format!(
                    "expected operation or fragment definition, found `{found}`"
                ),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec![
                        "query".to_string(),
                        "mutation".to_string(),
                        "subscription".to_string(),
                        "fragment".to_string(),
                        "{".to_string(),
                    ],
                    found,
                },
            ));
            Err(())
        }
    }

    /// Parses a definition for mixed documents.
    fn parse_mixed_definition_item(
        &mut self,
    ) -> Result<ast::Definition<'src>, ()> {
        // Handle lexer errors
        if let Some(token) = self.token_stream.peek()
            && let GraphQLTokenKind::Error(_) = &token.kind {
                let token = token.clone();
                self.handle_lexer_error(&token);
                self.consume_token();
                return Err(());
            }

        let description = self.parse_ast_description();

        if self.peek_is_keyword("schema") {
            Ok(ast::Definition::SchemaDefinition(
                self.parse_schema_definition(description)?,
            ))
        } else if self.peek_is_keyword("scalar") {
            Ok(ast::Definition::TypeDefinition(
                self.parse_scalar_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("type") {
            Ok(ast::Definition::TypeDefinition(
                self.parse_object_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("interface") {
            Ok(ast::Definition::TypeDefinition(
                self.parse_interface_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("union") {
            Ok(ast::Definition::TypeDefinition(
                self.parse_union_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("enum") {
            Ok(ast::Definition::TypeDefinition(
                self.parse_enum_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("input") {
            Ok(ast::Definition::TypeDefinition(
                self.parse_input_object_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("directive") {
            Ok(ast::Definition::DirectiveDefinition(
                self.parse_directive_definition(description)?,
            ))
        } else if self.peek_is_keyword("extend") {
            self.parse_type_extension()
        } else if self.peek_is_keyword("query")
            || self.peek_is_keyword("mutation")
            || self.peek_is_keyword("subscription")
            || self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            Ok(ast::Definition::OperationDefinition(
                self.parse_operation_definition()?,
            ))
        } else if self.peek_is_keyword("fragment") {
            Ok(ast::Definition::FragmentDefinition(
                self.parse_fragment_definition()?,
            ))
        } else {
            let span = self
                .token_stream.peek()
                .map(|t| t.span.clone())
                .unwrap_or_else(|| self.eof_span());
            let found = self
                .token_stream.peek()
                .map(|t| Self::token_kind_display(&t.kind))
                .unwrap_or_else(|| "end of input".to_string());
            // Consume the token to ensure forward progress during
            // error recovery. Without this, recovery sees the
            // unconsumed token as a potential definition start and
            // stops immediately, causing an infinite loop.
            self.consume_token();
            self.record_error(GraphQLParseError::new(
                format!("expected definition, found `{found}`"),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec![
                        "type".to_string(),
                        "query".to_string(),
                        "fragment".to_string(),
                    ],
                    found,
                },
            ));
            Err(())
        }
    }
}
