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
use crate::legacy_ast;
use crate::DefinitionKind;
use crate::DocumentKind;
use crate::GraphQLParseError;
use crate::GraphQLParseErrorKind;
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
/// use libgraphql_parser::legacy_ast;
/// use libgraphql_parser::GraphQLParser;
///
/// let source = "type Query { hello: String }";
/// let parser = GraphQLParser::new(source);
/// let result = parser.parse_schema_document();
///
/// assert!(result.is_ok());
/// if let Some(doc) = result.valid_ast() {
///     assert!(matches!(
///         doc.definitions[0],
///         legacy_ast::schema::Definition::TypeDefinition(_),
///     ));
/// }
/// ```
pub struct GraphQLParser<'src, TTokenSource: GraphQLTokenSource<'src>> {
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
    /// `parse_executable_type_annotation`, and `parse_schema_type_annotation`;
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
    /// assert!(result.is_ok());
    /// ```
    pub fn new<S: AsRef<str> + ?Sized>(
        source: &'src S,
    ) -> Self {
        let token_source =
            StrGraphQLTokenSource::new(source.as_ref());
        Self::from_token_source(token_source)
    }
}

impl<'src, TTokenSource: GraphQLTokenSource<'src>> GraphQLParser<'src, TTokenSource> {
    /// Maximum nesting depth for recursive parsing (values, selection
    /// sets, and type annotations).
    ///
    /// Prevents stack overflow from adversarial inputs like `[[[[[...`
    /// with hundreds of unclosed brackets. 64 levels is far beyond any
    /// realistic GraphQL document (most real-world documents nest
    /// fewer than 15 levels) while staying safe even in debug builds
    /// where un-optimized stack frames can be 4-8 KB each.
    const MAX_RECURSION_DEPTH: usize = 64;

    /// Creates a new parser from a token source.
    pub fn from_token_source(
        token_source: TTokenSource,
    ) -> Self {
        Self {
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

    /// Expects a name token and returns its value along with its source span.
    ///
    /// This is a thin wrapper around [`expect_name_only()`](Self::expect_name_only)
    /// for callers that need the source span. Use `expect_name_only()` when the
    /// span isn't needed to avoid an unnecessary clone.
    ///
    /// **Important**: Per the GraphQL spec, `true`, `false`, and `null` are
    /// valid names in most contexts (they match the Name regex). The lexer
    /// tokenizes them as distinct token kinds for type safety in value
    /// contexts, but this method accepts them as valid names.
    fn expect_name(&mut self) -> Result<(Cow<'src, str>, GraphQLSourceSpan), ()> {
        // Capture span before consuming - peek doesn't consume
        let span = self
            .token_stream
            .peek()
            .map(|t| t.span.clone())
            .unwrap_or_else(|| self.eof_span());
        let name = self.expect_name_only()?;
        Ok((name, span))
    }

    /// Expects a name token and returns its value without the span.
    ///
    /// Returns a `Cow<'src, str>` to avoid unnecessary allocations when the
    /// name is borrowed from the source. For `Name` tokens, returns the
    /// borrowed string; for `true`/`false`/`null` tokens, returns a static
    /// borrowed string.
    ///
    /// This is the core implementation that avoids cloning the span in the
    /// success case. Use [`expect_name()`](Self::expect_name) when you need
    /// the source span.
    ///
    /// **Important**: Per the GraphQL spec, `true`, `false`, and `null` are
    /// valid names in most contexts (they match the Name regex). The lexer
    /// tokenizes them as distinct token kinds for type safety in value
    /// contexts, but this method accepts them as valid names.
    fn expect_name_only(
        &mut self,
    ) -> Result<Cow<'src, str>, ()> {
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
                _ => Some((
                    token.span.clone(),
                    Self::token_kind_display(&token.kind),
                )),
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
        match token.kind {
            GraphQLTokenKind::Name(s) => Ok(s),
            GraphQLTokenKind::True => {
                Ok(Cow::Borrowed("true"))
            },
            GraphQLTokenKind::False => {
                Ok(Cow::Borrowed("false"))
            },
            GraphQLTokenKind::Null => {
                Ok(Cow::Borrowed("null"))
            },
            _ => unreachable!(),
        }
    }

    /// Expects a name token and returns an `ast::Name`.
    ///
    /// Moves the span from the consumed token (zero-cost).
    /// On error, does NOT consume the mismatched token (see
    /// error recovery convention in plan).
    fn expect_ast_name(
        &mut self,
    ) -> Result<ast::Name<'src>, ()> {
        let mismatch = match self.token_stream.peek() {
            None => {
                let span = self.eof_span();
                self.record_error(
                    GraphQLParseError::new(
                        "expected name",
                        span,
                        GraphQLParseErrorKind::UnexpectedEof {
                            expected: vec![
                                "name".to_string(),
                            ],
                        },
                    ),
                );
                return Err(());
            },
            Some(token) => match &token.kind {
                GraphQLTokenKind::Name(_)
                | GraphQLTokenKind::True
                | GraphQLTokenKind::False
                | GraphQLTokenKind::Null => None,
                _ => Some((
                    token.span.clone(),
                    Self::token_kind_display(
                        &token.kind,
                    ),
                )),
            },
        };
        if let Some((span, found)) = mismatch {
            self.record_error(
                GraphQLParseError::new(
                    format!(
                        "expected name, found \
                        `{found}`",
                    ),
                    span,
                    GraphQLParseErrorKind::UnexpectedToken {
                        expected: vec![
                            "name".to_string(),
                        ],
                        found,
                    },
                ),
            );
            return Err(());
        }
        let token = self.consume_token().unwrap();
        let value = match token.kind {
            GraphQLTokenKind::Name(s) => s,
            GraphQLTokenKind::True => {
                Cow::Borrowed("true")
            },
            GraphQLTokenKind::False => {
                Cow::Borrowed("false")
            },
            GraphQLTokenKind::Null => {
                Cow::Borrowed("null")
            },
            _ => unreachable!(),
        };
        Ok(ast::Name {
            span: token.span,
            syntax: None,
            value,
        })
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
    ) -> Result<GraphQLSourceSpan, ()> {
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
        Ok(self.consume_token().unwrap().span)
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
                Some(t.span.end_exclusive.clone());
        }
        token
    }

    /// Returns a span for EOF errors, anchored to the end of the
    /// last consumed token if available.
    fn eof_span(&self) -> GraphQLSourceSpan {
        if let Some(ref pos) = self.last_end_position {
            GraphQLSourceSpan::new(pos.clone(), pos.clone())
        } else {
            let zero = SourcePosition::new(0, 0, Some(0), 0);
            GraphQLSourceSpan::new(zero.clone(), zero)
        }
    }

    /// Builds a `GraphQLSourceSpan` from a start span (moved)
    /// and the parser's last-consumed end position.
    ///
    /// `start` is taken by value to move (not clone) its
    /// `start_inclusive`. The caller should pass the span from
    /// a consumed token directly — never clone a span just to
    /// pass it here.
    fn make_span(
        &self,
        start: GraphQLSourceSpan,
    ) -> GraphQLSourceSpan {
        let end = self
            .last_end_position
            .clone()
            .unwrap_or(start.start_inclusive.clone());
        GraphQLSourceSpan::new(
            start.start_inclusive,
            end,
        )
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
            GraphQLTokenKind::Error { message, .. } => {
                format!("tokenization error: {message}")
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
            GraphQLTokenKind::Error { .. } => {
                matches!(expected, GraphQLTokenKind::Error { .. })
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
        if let GraphQLTokenKind::Error {
            message,
            error_notes,
        } = &token.kind {
            self.record_error(GraphQLParseError::from_lexer_error(
                message.clone(),
                token.span.clone(),
                error_notes.clone(),
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
        &mut self,
        context: ConstContext,
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
                            self.record_error(
                                GraphQLParseError::new(
                                    format!(
                                        "variables are \
                                        not allowed \
                                        in {}",
                                        context
                                            .description(),
                                    ),
                                    span,
                                    GraphQLParseErrorKind::InvalidSyntax,
                                ),
                            );
                            return Err(());
                        }
                        let dollar =
                            self.consume_token().unwrap();
                        let name =
                            self.expect_ast_name()?;
                        let var_span =
                            self.make_span(dollar.span);
                        Ok(ast::Value::Variable(
                            ast::VariableValue {
                                name,
                                span: var_span,
                                syntax: None,
                            },
                        ))
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
                                    let token = self
                                        .consume_token()
                                        .unwrap();
                                    Ok(ast::Value::Int(
                                        ast::IntValue {
                                            span:
                                                token
                                                    .span,
                                            syntax:
                                                None,
                                            value:
                                                val
                                                    as i32,
                                        },
                                    ))
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
                                    let token = self
                                        .consume_token()
                                        .unwrap();
                                    Ok(
                                        ast::Value::Float(
                                            ast::FloatValue {
                                                span:
                                                    token.span,
                                                syntax:
                                                    None,
                                                value:
                                                    val,
                                            },
                                        ),
                                    )
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
                    GraphQLTokenKind::StringValue(
                        raw,
                    ) => {
                        let is_block =
                            raw.starts_with("\"\"\"");
                        let token_clone = token.clone();
                        self.consume_token();
                        let string_result =
                            token_clone
                                .kind
                                .parse_string_value();
                        match string_result {
                            Some(Ok(parsed)) => Ok(
                                ast::Value::String(
                                    ast::StringValue {
                                        is_block,
                                        span:
                                            token_clone
                                                .span,
                                        syntax: None,
                                        value:
                                            Cow::Owned(
                                                parsed,
                                            ),
                                    },
                                ),
                            ),
                            Some(Err(e)) => {
                                self.record_error(
                                    GraphQLParseError::new(
                                        format!(
                                            "invalid \
                                            string: {e}",
                                        ),
                                        span,
                                        GraphQLParseErrorKind::InvalidValue(
                                            ValueParsingError::String(
                                                e,
                                            ),
                                        ),
                                    ),
                                );
                                Err(())
                            },
                            None => {
                                self.record_error(
                                    GraphQLParseError::new(
                                        "invalid string",
                                        span,
                                        GraphQLParseErrorKind::InvalidSyntax,
                                    ),
                                );
                                Err(())
                            },
                        }
                    },

                    // Boolean literals
                    GraphQLTokenKind::True => {
                        let token =
                            self.consume_token().unwrap();
                        Ok(ast::Value::Boolean(
                            ast::BooleanValue {
                                span: token.span,
                                syntax: None,
                                value: true,
                            },
                        ))
                    },
                    GraphQLTokenKind::False => {
                        let token =
                            self.consume_token().unwrap();
                        Ok(ast::Value::Boolean(
                            ast::BooleanValue {
                                span: token.span,
                                syntax: None,
                                value: false,
                            },
                        ))
                    },

                    // Null literal
                    GraphQLTokenKind::Null => {
                        let token =
                            self.consume_token().unwrap();
                        Ok(ast::Value::Null(
                            ast::NullValue {
                                span: token.span,
                                syntax: None,
                            },
                        ))
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
                        let token =
                            self.consume_token().unwrap();
                        let token_span = token.span;
                        let value = match token.kind {
                            GraphQLTokenKind::Name(
                                s,
                            ) => s,
                            _ => unreachable!(),
                        };
                        Ok(ast::Value::Enum(
                            ast::EnumValue {
                                span: token_span,
                                syntax: None,
                                value,
                            },
                        ))
                    },

                    // Lexer error
                    GraphQLTokenKind::Error { .. } => {
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
        &mut self,
        context: ConstContext,
    ) -> Result<ast::Value<'src>, ()> {
        let open_token = self.expect(
            &GraphQLTokenKind::SquareBracketOpen,
        )?;
        self.push_delimiter(
            open_token.span.clone(),
            DelimiterContext::ListValue,
        );

        let mut values = Vec::new();

        loop {
            if self.peek_is(
                &GraphQLTokenKind::SquareBracketClose,
            ) {
                break;
            }
            if self.token_stream.is_at_end() {
                let span = self.eof_span();
                let open_delim =
                    self.pop_delimiter();
                let mut error =
                    GraphQLParseError::new(
                        "unclosed `[`",
                        span,
                        GraphQLParseErrorKind::UnclosedDelimiter {
                            delimiter: "["
                                .to_string(),
                        },
                    );
                if let Some(delim) = open_delim {
                    error.add_note_with_span(
                        "opening `[` here",
                        delim.span,
                    );
                }
                self.record_error(error);
                return Err(());
            }

            match self.parse_value(context) {
                Ok(value) => values.push(value),
                Err(()) => {
                    self.skip_to_list_recovery_point();
                    if self.peek_is(
                        &GraphQLTokenKind::SquareBracketClose,
                    ) {
                        break;
                    }
                },
            }
        }

        self.expect(
            &GraphQLTokenKind::SquareBracketClose,
        )?;
        self.pop_delimiter();

        let span = self.make_span(open_token.span);
        Ok(ast::Value::List(ast::ListValue {
            span,
            syntax: None,
            values,
        }))
    }

    /// Parses an object value: `{ field: value, ... }`
    fn parse_object_value(
        &mut self,
        context: ConstContext,
    ) -> Result<ast::Value<'src>, ()> {
        let open_token = self.expect(
            &GraphQLTokenKind::CurlyBraceOpen,
        )?;
        self.push_delimiter(
            open_token.span.clone(),
            DelimiterContext::ObjectValue,
        );

        let mut fields = Vec::new();

        loop {
            if self.peek_is(
                &GraphQLTokenKind::CurlyBraceClose,
            ) {
                break;
            }
            if self.token_stream.is_at_end() {
                let span = self.eof_span();
                let open_delim =
                    self.pop_delimiter();
                let mut error =
                    GraphQLParseError::new(
                        "unclosed `{`",
                        span,
                        GraphQLParseErrorKind::UnclosedDelimiter {
                            delimiter: "{"
                                .to_string(),
                        },
                    );
                if let Some(delim) = open_delim {
                    error.add_note_with_span(
                        format!(
                            "opening `{{` in {} here",
                            delim
                                .context
                                .description(),
                        ),
                        delim.span,
                    );
                }
                self.record_error(error);
                return Err(());
            }

            let field_name =
                self.expect_ast_name()?;
            self.expect(
                &GraphQLTokenKind::Colon,
            )?;
            let value =
                self.parse_value(context)?;
            let field_span =
                GraphQLSourceSpan::new(
                    field_name
                        .span
                        .start_inclusive
                        .clone(),
                    self.last_end_position
                        .clone()
                        .unwrap_or(
                            field_name
                                .span
                                .end_exclusive
                                .clone(),
                        ),
                );
            fields.push(ast::ObjectField {
                name: field_name,
                span: field_span,
                syntax: None,
                value,
            });
        }

        self.expect(
            &GraphQLTokenKind::CurlyBraceClose,
        )?;
        self.pop_delimiter();

        let span = self.make_span(open_token.span);
        Ok(ast::Value::Object(ast::ObjectValue {
            fields,
            span,
            syntax: None,
        }))
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
                    | GraphQLTokenKind::Error { .. } => {
                        self.consume_token();
                    }
                },
            }
        }
    }

    // =========================================================================
    // Type annotation parsing
    // =========================================================================

    /// Parses a type annotation: `TypeName`, `[Type]`, `Type!`, `[Type]!`,
    /// etc.
    fn parse_executable_type_annotation(
        &mut self,
    ) -> Result<legacy_ast::operation::Type, ()> {
        self.enter_recursion()?;
        let result = self.parse_executable_type_annotation_impl();
        self.exit_recursion();
        result
    }

    /// Inner implementation of type annotation parsing.
    fn parse_executable_type_annotation_impl(
        &mut self,
    ) -> Result<legacy_ast::operation::Type, ()> {
        let base_type = if self.peek_is(&GraphQLTokenKind::SquareBracketOpen) {
            // List type: [InnerType]
            self.parse_executable_list_type()?
        } else {
            // Named type
            self.parse_executable_named_type()?
        };

        // Check for non-null modifier
        if self.peek_is(&GraphQLTokenKind::Bang) {
            self.consume_token();
            Ok(legacy_ast::operation::Type::NonNullType(Box::new(base_type)))
        } else {
            Ok(base_type)
        }
    }

    /// Parses a named type annotation (just the type name).
    fn parse_executable_named_type(&mut self) -> Result<legacy_ast::operation::Type, ()> {
        let name = self.expect_name_only()?;
        Ok(legacy_ast::operation::Type::NamedType(name.into_owned()))
    }

    /// Parses a list type annotation: `[InnerType]`
    fn parse_executable_list_type(&mut self) -> Result<legacy_ast::operation::Type, ()> {
        let open_token = self.expect(&GraphQLTokenKind::SquareBracketOpen)?;
        self.push_delimiter(open_token.span.clone(), DelimiterContext::ListType);

        let inner = self.parse_executable_type_annotation()?;

        self.expect(&GraphQLTokenKind::SquareBracketClose)?;
        self.pop_delimiter();

        Ok(legacy_ast::operation::Type::ListType(Box::new(inner)))
    }

    // =========================================================================
    // Directive annotation parsing
    // =========================================================================

    /// Parses zero or more directive annotations: `@directive(args)...`
    fn parse_directive_annotations(
        &mut self,
    ) -> Result<Vec<legacy_ast::operation::Directive>, ()> {
        let mut directives = Vec::new();
        while self.peek_is(&GraphQLTokenKind::At) {
            directives.push(self.parse_directive_annotation()?);
        }
        Ok(directives)
    }

    /// Parses a single directive annotation: `@name` or `@name(args)`
    fn parse_directive_annotation(&mut self) -> Result<legacy_ast::operation::Directive, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self
            .expect(&GraphQLTokenKind::At)?
            .span
            .start_inclusive
            .to_ast_pos();
        let name = self.expect_name_only()?;

        let arguments = if self.peek_is(&GraphQLTokenKind::ParenOpen) {
            self.parse_arguments(DelimiterContext::DirectiveArguments)?
        } else {
            Vec::new()
        };

        Ok(legacy_ast::operation::Directive {
            position,
            name: name.into_owned(),
            arguments,
        })
    }

    /// Parses directive annotations that may appear in const contexts
    /// (directive arguments must be const values).
    fn parse_const_directive_annotations(
        &mut self,
    ) -> Result<Vec<legacy_ast::operation::Directive>, ()> {
        let mut directives = Vec::new();
        while self.peek_is(&GraphQLTokenKind::At) {
            directives.push(self.parse_const_directive_annotation()?);
        }
        Ok(directives)
    }

    /// Parses a directive annotation with const-only arguments.
    fn parse_const_directive_annotation(
        &mut self,
    ) -> Result<legacy_ast::operation::Directive, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self
            .expect(&GraphQLTokenKind::At)?
            .span
            .start_inclusive
            .to_ast_pos();
        let name = self.expect_name_only()?;

        let arguments = if self.peek_is(&GraphQLTokenKind::ParenOpen) {
            self.parse_const_arguments(DelimiterContext::DirectiveArguments)?
        } else {
            Vec::new()
        };

        Ok(legacy_ast::operation::Directive {
            position,
            name: name.into_owned(),
            arguments,
        })
    }

    // =========================================================================
    // Arguments parsing
    // =========================================================================

    /// Parses arguments: `(name: value, ...)`
    fn parse_arguments(
        &mut self,
        context: DelimiterContext,
    ) -> Result<Vec<(String, legacy_ast::Value)>, ()> {
        let open_token = self.expect(&GraphQLTokenKind::ParenOpen)?;
        self.push_delimiter(open_token.span.clone(), context);

        let mut arguments = Vec::new();

        // Check for empty arguments
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

            let arg_name = self.expect_name_only()?;
            self.expect(&GraphQLTokenKind::Colon)?;
            let value = self.parse_value(ConstContext::AllowVariables)?;

            arguments.push((arg_name.into_owned(), value));
        }

        self.expect(&GraphQLTokenKind::ParenClose)?;
        self.pop_delimiter();

        Ok(arguments)
    }

    /// Parses arguments with const-only values.
    fn parse_const_arguments(
        &mut self,
        context: DelimiterContext,
    ) -> Result<Vec<(String, legacy_ast::Value)>, ()> {
        let open_token = self.expect(&GraphQLTokenKind::ParenOpen)?;
        self.push_delimiter(open_token.span.clone(), context);

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

            let arg_name = self.expect_name_only()?;
            self.expect(&GraphQLTokenKind::Colon)?;
            let value = self.parse_value(ConstContext::DirectiveArgument)?;

            arguments.push((arg_name.into_owned(), value));
        }

        self.expect(&GraphQLTokenKind::ParenClose)?;
        self.pop_delimiter();

        Ok(arguments)
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
    fn parse_selection_set(
        &mut self,
    ) -> Result<legacy_ast::operation::SelectionSet, ()> {
        self.enter_recursion()?;
        let result = self.parse_selection_set_impl();
        self.exit_recursion();
        result
    }

    /// Inner implementation of selection set parsing.
    fn parse_selection_set_impl(
        &mut self,
    ) -> Result<legacy_ast::operation::SelectionSet, ()> {
        let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
        // Performance: Store only the AstPos (Copy) from the open brace, not
        // the full GraphQLToken or GraphQLSourceSpan. The close brace position
        // will be extracted similarly when we reach it.
        let open_pos = open_token.span.start_inclusive.to_ast_pos();
        self.push_delimiter(open_token.span.clone(), DelimiterContext::SelectionSet);

        let mut selections = Vec::new();

        // Check for empty selection set
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
                Ok(selection) => selections.push(selection),
                Err(()) => {
                    // Try to recover by skipping to next selection or }
                    self.skip_to_selection_recovery_point();
                }
            }
        }

        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // close brace span rather than storing the full GraphQLSourceSpan.
        let close_token = self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
        let close_pos = close_token.span.start_inclusive.to_ast_pos();
        self.pop_delimiter();

        Ok(legacy_ast::operation::SelectionSet {
            span: (open_pos, close_pos),
            items: selections,
        })
    }

    /// Parses a single selection (field, fragment spread, or inline fragment).
    fn parse_selection(&mut self) -> Result<legacy_ast::operation::Selection, ()> {
        if self.peek_is(&GraphQLTokenKind::Ellipsis) {
            // Fragment spread or inline fragment.
            // Performance: Extract AstPos (16 bytes, Copy) immediately from the
            // span rather than storing the full GraphQLSourceSpan (~104 bytes
            // with Option<PathBuf>). Pass AstPos by value (Copy) to helpers.
            let ellipsis_pos = self
                .expect(&GraphQLTokenKind::Ellipsis)?
                .span
                .start_inclusive
                .to_ast_pos();

            if self.peek_is_keyword("on")
                || self.peek_is(&GraphQLTokenKind::At)
                || self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
                // Inline fragment
                // Performance: Pass AstPos by value (Copy, 16 bytes) rather
                // than GraphQLSourceSpan by reference, as the callee only needs
                // the position.
                self.parse_inline_fragment(ellipsis_pos)
            } else {
                // Fragment spread
                // Performance: Pass AstPos by value (Copy, 16 bytes) rather
                // than GraphQLSourceSpan by reference, as the callee only needs
                // the position.
                self.parse_fragment_spread(ellipsis_pos)
            }
        } else {
            // Field
            self.parse_field().map(legacy_ast::operation::Selection::Field)
        }
    }

    /// Parses a field: `alias: name(args) @directives { selections }`
    fn parse_field(&mut self) -> Result<legacy_ast::operation::Field, ()> {
        // Parse name or alias. We use expect_name() (not expect_name_only()) to
        // capture the span for position tracking. The position is the start of
        // the field, which could be an alias or the field name itself.
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let (first_name, first_span) = self.expect_name()?;
        let position = first_span.start_inclusive.to_ast_pos();

        // Check for alias
        let (alias, name) = if self.peek_is(&GraphQLTokenKind::Colon) {
            self.consume_token();
            let field_name = self.expect_name_only()?;
            (Some(first_name), field_name)
        } else {
            (None, first_name)
        };

        // Parse arguments
        let arguments = if self.peek_is(&GraphQLTokenKind::ParenOpen) {
            self.parse_arguments(DelimiterContext::FieldArguments)?
        } else {
            Vec::new()
        };

        // Parse directives
        let directives = self.parse_directive_annotations()?;

        // Parse nested selection set
        let selection_set = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_selection_set()?
        } else {
            // Performance: For fields without a selection set, use the field's
            // position (already extracted as AstPos) for the empty span rather
            // than (0,0). This provides useful location context for tooling
            // while avoiding any additional span extraction.
            legacy_ast::operation::SelectionSet {
                span: (position, position),
                items: Vec::new(),
            }
        };

        Ok(legacy_ast::operation::Field {
            position,
            alias: alias.map(|a| a.into_owned()),
            name: name.into_owned(),
            arguments,
            directives,
            selection_set,
        })
    }

    /// Parses a fragment spread: `...FragmentName @directives`
    /// (called after consuming `...`)
    ///
    /// # Arguments
    /// * `position` - The position of the `...` token, passed as `AstPos`
    ///   (Copy, 16 bytes) rather than `GraphQLSourceSpan` (~104 bytes, contains
    ///   `Option<PathBuf>`) to avoid unnecessary allocation/copying of the full
    ///   span when only the start position is needed for the AST node.
    fn parse_fragment_spread(
        &mut self,
        position: legacy_ast::AstPos,
    ) -> Result<legacy_ast::operation::Selection, ()> {
        let fragment_name = self.expect_name_only()?;
        let directives = self.parse_directive_annotations()?;

        Ok(legacy_ast::operation::Selection::FragmentSpread(
            legacy_ast::operation::FragmentSpread {
                position,
                fragment_name: fragment_name.into_owned(),
                directives,
            },
        ))
    }

    /// Parses an inline fragment: `... on Type @directives { selections }`
    /// or `... @directives { selections }` (called after consuming `...`)
    ///
    /// # Arguments
    /// * `position` - The position of the `...` token, passed as `AstPos`
    ///   (Copy, 16 bytes) rather than `GraphQLSourceSpan` (~104 bytes, contains
    ///   `Option<PathBuf>`) to avoid unnecessary allocation/copying of the full
    ///   span when only the start position is needed for the AST node.
    fn parse_inline_fragment(
        &mut self,
        position: legacy_ast::AstPos,
    ) -> Result<legacy_ast::operation::Selection, ()> {
        // Optional type condition
        let type_condition = if self.peek_is_keyword("on") {
            self.consume_token(); // consume 'on'
            let type_name = self.expect_name_only()?;
            Some(legacy_ast::operation::TypeCondition::On(type_name.into_owned()))
        } else {
            None
        };

        let directives = self.parse_directive_annotations()?;
        let selection_set = self.parse_selection_set()?;

        Ok(legacy_ast::operation::Selection::InlineFragment(
            legacy_ast::operation::InlineFragment {
                position,
                type_condition,
                directives,
                selection_set,
            },
        ))
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
    fn parse_operation_definition(
        &mut self,
    ) -> Result<legacy_ast::operation::OperationDefinition, ()> {
        // Check for shorthand query (just a selection set)
        if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            let selection_set = self.parse_selection_set()?;
            return Ok(legacy_ast::operation::OperationDefinition::SelectionSet(
                selection_set,
            ));
        }

        // Parse operation type keyword and capture position.
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let (op_type, position) = if self.peek_is_keyword("query") {
            (
                "query",
                self.expect_keyword("query")?.start_inclusive.to_ast_pos(),
            )
        } else if self.peek_is_keyword("mutation") {
            (
                "mutation",
                self.expect_keyword("mutation")?
                    .start_inclusive
                    .to_ast_pos(),
            )
        } else if self.peek_is_keyword("subscription") {
            (
                "subscription",
                self.expect_keyword("subscription")?
                    .start_inclusive
                    .to_ast_pos(),
            )
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
            && !self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            if let Some(token) = self.token_stream.peek() {
                match &token.kind {
                    GraphQLTokenKind::Name(_)
                    | GraphQLTokenKind::True
                    | GraphQLTokenKind::False
                    | GraphQLTokenKind::Null => {
                        let n = self.expect_name_only()?;
                        Some(n)
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        // Optional variable definitions
        let variable_definitions = if self.peek_is(&GraphQLTokenKind::ParenOpen) {
            self.parse_variable_definitions()?
        } else {
            Vec::new()
        };

        // Optional directives
        let directives = self.parse_directive_annotations()?;

        // Required selection set
        let selection_set = self.parse_selection_set()?;

        // Build the appropriate operation type
        let name = name.map(|n| n.into_owned());
        match op_type {
            "query" => Ok(legacy_ast::operation::OperationDefinition::Query(
                legacy_ast::operation::Query {
                    position,
                    name,
                    variable_definitions,
                    directives,
                    selection_set,
                },
            )),
            "mutation" => Ok(legacy_ast::operation::OperationDefinition::Mutation(
                legacy_ast::operation::Mutation {
                    position,
                    name,
                    variable_definitions,
                    directives,
                    selection_set,
                },
            )),
            "subscription" => Ok(legacy_ast::operation::OperationDefinition::Subscription(
                legacy_ast::operation::Subscription {
                    position,
                    name,
                    variable_definitions,
                    directives,
                    selection_set,
                },
            )),
            _ => unreachable!(),
        }
    }

    /// Parses variable definitions: `($var: Type = default, ...)`
    fn parse_variable_definitions(
        &mut self,
    ) -> Result<Vec<legacy_ast::operation::VariableDefinition>, ()> {
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

        self.expect(&GraphQLTokenKind::ParenClose)?;
        self.pop_delimiter();

        Ok(definitions)
    }

    /// Parses a single variable definition: `$name: Type = default @directives`
    fn parse_variable_definition(
        &mut self,
    ) -> Result<legacy_ast::operation::VariableDefinition, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self
            .expect(&GraphQLTokenKind::Dollar)?
            .span
            .start_inclusive
            .to_ast_pos();
        let name = self.expect_name_only()?;
        self.expect(&GraphQLTokenKind::Colon)?;
        let var_type = self.parse_executable_type_annotation()?;

        // Optional default value
        let default_value = if self.peek_is(&GraphQLTokenKind::Equals) {
            self.consume_token();
            Some(self.parse_value(ConstContext::VariableDefaultValue)?)
        } else {
            None
        };

        // Note: Variable directives are supported in the GraphQL spec but not
        // in the graphql_parser crate's AST. We parse and discard them for now.
        // TODO: Track these when we have a custom AST.
        let _directives = self.parse_const_directive_annotations()?;

        Ok(legacy_ast::operation::VariableDefinition {
            position,
            name: name.into_owned(),
            var_type,
            default_value,
        })
    }

    // =========================================================================
    // Fragment parsing
    // =========================================================================

    /// Parses a fragment definition: `fragment Name on Type @directives {
    /// ... }`
    fn parse_fragment_definition(
        &mut self,
    ) -> Result<legacy_ast::operation::FragmentDefinition, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self
            .expect_keyword("fragment")?
            .start_inclusive
            .to_ast_pos();

        // Parse fragment name - must not be "on"
        let (name, name_span) = self.expect_name()?;
        if name == "on" {
            // Still produce AST but record error
            let mut error = GraphQLParseError::new(
                "fragment name cannot be `on`",
                name_span.clone(),
                GraphQLParseErrorKind::ReservedName {
                    name: "on".to_string(),
                    context: ReservedNameContext::FragmentName,
                },
            );
            error.add_spec(
                "https://spec.graphql.org/October2021/#sec-Fragment-Name-Uniqueness",
            );
            self.record_error(error);
        }

        // Type condition
        let type_condition = self.parse_type_condition()?;

        // Optional directives
        let directives = self.parse_directive_annotations()?;

        // Selection set
        let selection_set = self.parse_selection_set()?;

        Ok(legacy_ast::operation::FragmentDefinition {
            position,
            name: name.into_owned(),
            type_condition,
            directives,
            selection_set,
        })
    }

    /// Parses a type condition: `on TypeName`
    fn parse_type_condition(&mut self) -> Result<legacy_ast::operation::TypeCondition, ()> {
        self.expect_keyword("on")?;
        let type_name = self.expect_name_only()?;
        Ok(legacy_ast::operation::TypeCondition::On(type_name.into_owned()))
    }

    // =========================================================================
    // Type definition parsing
    // =========================================================================

    /// Parses an optional description (string before a definition).
    fn parse_description(&mut self) -> Option<String> {
        if let Some(token) = self.token_stream.peek()
            && matches!(&token.kind, GraphQLTokenKind::StringValue(_)) {
            let token = self.consume_token().unwrap();
            match token.kind.parse_string_value() {
                Some(Ok(parsed)) => return Some(parsed),
                Some(Err(err)) => {
                    self.record_error(GraphQLParseError::new(
                        format!(
                            "invalid string in description: {err}"
                        ),
                        token.span,
                        GraphQLParseErrorKind::InvalidSyntax,
                    ));
                },
                None => unreachable!(),
            }
        }
        None
    }

    /// Parses an optional description, returning an
    /// `ast::StringValue` with the span moved from the
    /// consumed token.
    fn parse_ast_description(
        &mut self,
    ) -> Option<ast::StringValue<'src>> {
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
                    return Some(ast::StringValue {
                        is_block,
                        span: token.span,
                        syntax: None,
                        value: Cow::Owned(parsed),
                    });
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
    fn parse_schema_definition(&mut self) -> Result<legacy_ast::schema::SchemaDefinition, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self
            .expect_keyword("schema")?
            .start_inclusive
            .to_ast_pos();

        let directives = self.parse_const_directive_annotations()?;

        let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
        self.push_delimiter(
            open_token.span.clone(),
            DelimiterContext::SchemaDefinition,
        );

        let mut query = None;
        let mut mutation = None;
        let mut subscription = None;

        loop {
            if self.peek_is(&GraphQLTokenKind::CurlyBraceClose) {
                break;
            }
            if self.token_stream.is_at_end() {
                self.handle_unclosed_brace();
                return Err(());
            }

            let (operation_type, operation_type_span) =
                self.expect_name()?;
            self.expect(&GraphQLTokenKind::Colon)?;
            let type_name = self.expect_name_only()?;

            match &*operation_type {
                "query" => query = Some(type_name.into_owned()),
                "mutation" => {
                    mutation = Some(type_name.into_owned())
                },
                "subscription" => {
                    subscription = Some(type_name.into_owned())
                },
                _ => {
                    self.record_error(GraphQLParseError::new(
                        format!(
                            "unknown operation type \
                            `{operation_type}`; expected \
                            `query`, `mutation`, or \
                            `subscription`"
                        ),
                        operation_type_span,
                        GraphQLParseErrorKind::InvalidSyntax,
                    ));
                }
            }
        }

        self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
        self.pop_delimiter();

        // Convert directives for schema type
        let schema_directives = self.convert_directives_to_schema(directives);

        Ok(legacy_ast::schema::SchemaDefinition {
            position,
            directives: schema_directives,
            query,
            mutation,
            subscription,
        })
    }

    /// Parses a scalar type definition: `scalar Name @directives`
    fn parse_scalar_type_definition(
        &mut self,
        description: Option<String>,
    ) -> Result<legacy_ast::schema::TypeDefinition, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self.expect_keyword("scalar")?.start_inclusive.to_ast_pos();
        let name = self.expect_name_only()?;
        let directives = self.parse_const_directive_annotations()?;

        let schema_directives = self.convert_directives_to_schema(directives);

        Ok(legacy_ast::schema::TypeDefinition::Scalar(legacy_ast::schema::ScalarType {
            position,
            description,
            name: name.into_owned(),
            directives: schema_directives,
        }))
    }

    /// Parses an object type definition: `type Name implements I & J
    /// @directives { fields }`
    fn parse_object_type_definition(
        &mut self,
        description: Option<String>,
    ) -> Result<legacy_ast::schema::TypeDefinition, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self.expect_keyword("type")?.start_inclusive.to_ast_pos();
        let name = self.expect_name_only()?;

        let implements_interfaces = if self.peek_is_keyword("implements") {
            self.parse_implements_interfaces()?
        } else {
            Vec::new()
        };

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        let fields = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_fields_definition(DelimiterContext::ObjectTypeDefinition)?
        } else {
            Vec::new()
        };

        Ok(legacy_ast::schema::TypeDefinition::Object(legacy_ast::schema::ObjectType {
            position,
            description,
            name: name.into_owned(),
            implements_interfaces,
            directives: schema_directives,
            fields,
        }))
    }

    /// Parses an interface type definition.
    fn parse_interface_type_definition(
        &mut self,
        description: Option<String>,
    ) -> Result<legacy_ast::schema::TypeDefinition, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self
            .expect_keyword("interface")?
            .start_inclusive
            .to_ast_pos();
        let name = self.expect_name_only()?;

        let implements_interfaces = if self.peek_is_keyword("implements") {
            self.parse_implements_interfaces()?
        } else {
            Vec::new()
        };

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        let fields = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_fields_definition(DelimiterContext::InterfaceDefinition)?
        } else {
            Vec::new()
        };

        Ok(legacy_ast::schema::TypeDefinition::Interface(
            legacy_ast::schema::InterfaceType {
                position,
                description,
                name: name.into_owned(),
                implements_interfaces,
                directives: schema_directives,
                fields,
            },
        ))
    }

    /// Parses a union type definition: `union Name @directives = A | B | C`
    fn parse_union_type_definition(
        &mut self,
        description: Option<String>,
    ) -> Result<legacy_ast::schema::TypeDefinition, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self.expect_keyword("union")?.start_inclusive.to_ast_pos();
        let name = self.expect_name_only()?;

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        let mut types = Vec::new();
        if self.peek_is(&GraphQLTokenKind::Equals) {
            self.consume_token();

            // Optional leading |
            if self.peek_is(&GraphQLTokenKind::Pipe) {
                self.consume_token();
            }

            let first_type = self.expect_name_only()?;
            types.push(first_type.into_owned());

            while self.peek_is(&GraphQLTokenKind::Pipe) {
                self.consume_token();
                let member_type = self.expect_name_only()?;
                types.push(member_type.into_owned());
            }
        }

        Ok(legacy_ast::schema::TypeDefinition::Union(legacy_ast::schema::UnionType {
            position,
            description,
            name: name.into_owned(),
            directives: schema_directives,
            types,
        }))
    }

    /// Parses an enum type definition: `enum Name @directives { VALUES }`
    fn parse_enum_type_definition(
        &mut self,
        description: Option<String>,
    ) -> Result<legacy_ast::schema::TypeDefinition, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self.expect_keyword("enum")?.start_inclusive.to_ast_pos();
        let name = self.expect_name_only()?;

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        let values = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_enum_values_definition()?
        } else {
            Vec::new()
        };

        Ok(legacy_ast::schema::TypeDefinition::Enum(legacy_ast::schema::EnumType {
            position,
            description,
            name: name.into_owned(),
            directives: schema_directives,
            values,
        }))
    }

    /// Parses an input object type definition.
    fn parse_input_object_type_definition(
        &mut self,
        description: Option<String>,
    ) -> Result<legacy_ast::schema::TypeDefinition, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self.expect_keyword("input")?.start_inclusive.to_ast_pos();
        let name = self.expect_name_only()?;

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        let fields = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_input_fields_definition()?
        } else {
            Vec::new()
        };

        Ok(legacy_ast::schema::TypeDefinition::InputObject(
            legacy_ast::schema::InputObjectType {
                position,
                description,
                name: name.into_owned(),
                directives: schema_directives,
                fields,
            },
        ))
    }

    /// Parses a directive definition.
    fn parse_directive_definition(
        &mut self,
        description: Option<String>,
    ) -> Result<legacy_ast::schema::DirectiveDefinition, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let position = self
            .expect_keyword("directive")?
            .start_inclusive
            .to_ast_pos();
        self.expect(&GraphQLTokenKind::At)?;
        let name = self.expect_name_only()?;

        let arguments = if self.peek_is(&GraphQLTokenKind::ParenOpen) {
            self.parse_arguments_definition()?
        } else {
            Vec::new()
        };

        let repeatable = if self.peek_is_keyword("repeatable") {
            self.consume_token();
            true
        } else {
            false
        };

        self.expect_keyword("on")?;

        // Parse directive locations
        let locations = self.parse_directive_locations()?;

        Ok(legacy_ast::schema::DirectiveDefinition {
            position,
            description,
            name: name.into_owned(),
            arguments,
            repeatable,
            locations,
        })
    }

    /// Parses implements interfaces: `implements A & B & C`
    fn parse_implements_interfaces(&mut self) -> Result<Vec<String>, ()> {
        self.expect_keyword("implements")?;

        // Optional leading &
        if self.peek_is(&GraphQLTokenKind::Ampersand) {
            self.consume_token();
        }

        let mut interfaces = Vec::new();
        let first = self.expect_name_only()?;
        interfaces.push(first.into_owned());

        while self.peek_is(&GraphQLTokenKind::Ampersand) {
            self.consume_token();
            let iface = self.expect_name_only()?;
            interfaces.push(iface.into_owned());
        }

        Ok(interfaces)
    }

    /// Parses field definitions: `{ field: Type, ... }`
    fn parse_fields_definition(
        &mut self,
        context: DelimiterContext,
    ) -> Result<Vec<legacy_ast::schema::Field>, ()> {
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

        self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
        self.pop_delimiter();

        Ok(fields)
    }

    /// Parses a single field definition.
    fn parse_field_definition(&mut self) -> Result<legacy_ast::schema::Field, ()> {
        let description = self.parse_description();
        // Use expect_name() (not expect_name_only()) to capture the span for
        // position tracking.
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let (name, name_span) = self.expect_name()?;
        let position = name_span.start_inclusive.to_ast_pos();

        let arguments = if self.peek_is(&GraphQLTokenKind::ParenOpen) {
            self.parse_arguments_definition()?
        } else {
            Vec::new()
        };

        self.expect(&GraphQLTokenKind::Colon)?;
        let field_type = self.parse_schema_type_annotation()?;

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        Ok(legacy_ast::schema::Field {
            position,
            description,
            name: name.into_owned(),
            arguments,
            field_type,
            directives: schema_directives,
        })
    }

    /// Parses argument definitions: `(arg: Type = default, ...)`
    fn parse_arguments_definition(&mut self) -> Result<Vec<legacy_ast::schema::InputValue>, ()> {
        let open_token = self.expect(&GraphQLTokenKind::ParenOpen)?;
        self.push_delimiter(
            open_token.span.clone(),
            DelimiterContext::ArgumentDefinitions,
        );

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

        self.expect(&GraphQLTokenKind::ParenClose)?;
        self.pop_delimiter();

        Ok(arguments)
    }

    /// Parses input fields definition (for input objects).
    fn parse_input_fields_definition(
        &mut self,
    ) -> Result<Vec<legacy_ast::schema::InputValue>, ()> {
        let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
        self.push_delimiter(
            open_token.span.clone(),
            DelimiterContext::InputObjectDefinition,
        );

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

        self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
        self.pop_delimiter();

        Ok(fields)
    }

    /// Parses an input value definition (used for arguments and input fields).
    fn parse_input_value_definition(&mut self) -> Result<legacy_ast::schema::InputValue, ()> {
        let description = self.parse_description();
        // Use expect_name() (not expect_name_only()) to capture the span for
        // position tracking.
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let (name, name_span) = self.expect_name()?;
        let position = name_span.start_inclusive.to_ast_pos();
        self.expect(&GraphQLTokenKind::Colon)?;
        let value_type = self.parse_schema_type_annotation()?;

        let default_value = if self.peek_is(&GraphQLTokenKind::Equals) {
            self.consume_token();
            Some(self.parse_value(ConstContext::InputDefaultValue)?)
        } else {
            None
        };

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        Ok(legacy_ast::schema::InputValue {
            position,
            description,
            name: name.into_owned(),
            value_type,
            default_value,
            directives: schema_directives,
        })
    }

    /// Parses enum value definitions.
    fn parse_enum_values_definition(
        &mut self,
    ) -> Result<Vec<legacy_ast::schema::EnumValue>, ()> {
        let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
        self.push_delimiter(
            open_token.span.clone(),
            DelimiterContext::EnumDefinition,
        );

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

        self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
        self.pop_delimiter();

        Ok(values)
    }

    /// Parses a single enum value definition.
    fn parse_enum_value_definition(&mut self) -> Result<legacy_ast::schema::EnumValue, ()> {
        let description = self.parse_description();

        // Check for reserved enum values (true, false, null)
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). The span is consumed and dropped here; only the
        // lightweight position is retained.
        let (name, name_span) = self.expect_name()?;
        let position = name_span.start_inclusive.to_ast_pos();
        if matches!(&*name, "true" | "false" | "null") {
            let mut error = GraphQLParseError::new(
                format!("enum value cannot be `{name}`"),
                name_span,
                GraphQLParseErrorKind::ReservedName {
                    name: name.clone().into_owned(),
                    context: ReservedNameContext::EnumValue,
                },
            );
            error.add_spec(
                "https://spec.graphql.org/October2021/#sec-Enum-Value-Uniqueness",
            );
            self.record_error(error);
            // Continue parsing to collect more errors
        }

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        Ok(legacy_ast::schema::EnumValue {
            position,
            description,
            name: name.into_owned(),
            directives: schema_directives,
        })
    }

    /// Parses directive locations: `FIELD | OBJECT | ...`
    fn parse_directive_locations(
        &mut self,
    ) -> Result<Vec<legacy_ast::schema::DirectiveLocation>, ()> {
        // Optional leading |
        if self.peek_is(&GraphQLTokenKind::Pipe) {
            self.consume_token();
        }

        let mut locations = Vec::new();
        locations.push(self.parse_directive_location()?);

        while self.peek_is(&GraphQLTokenKind::Pipe) {
            self.consume_token();
            locations.push(self.parse_directive_location()?);
        }

        Ok(locations)
    }

    /// Parses a single directive location.
    fn parse_directive_location(
        &mut self,
    ) -> Result<legacy_ast::schema::DirectiveLocation, ()> {
        let (name, span) = self.expect_name()?;

        // Match against known directive locations
        match &*name {
            // Executable locations
            "QUERY" => Ok(legacy_ast::schema::DirectiveLocation::Query),
            "MUTATION" => Ok(legacy_ast::schema::DirectiveLocation::Mutation),
            "SUBSCRIPTION" => Ok(legacy_ast::schema::DirectiveLocation::Subscription),
            "FIELD" => Ok(legacy_ast::schema::DirectiveLocation::Field),
            "FRAGMENT_DEFINITION" => {
                Ok(legacy_ast::schema::DirectiveLocation::FragmentDefinition)
            }
            "FRAGMENT_SPREAD" => Ok(legacy_ast::schema::DirectiveLocation::FragmentSpread),
            "INLINE_FRAGMENT" => Ok(legacy_ast::schema::DirectiveLocation::InlineFragment),
            "VARIABLE_DEFINITION" => {
                Ok(legacy_ast::schema::DirectiveLocation::VariableDefinition)
            }

            // Type system locations
            "SCHEMA" => Ok(legacy_ast::schema::DirectiveLocation::Schema),
            "SCALAR" => Ok(legacy_ast::schema::DirectiveLocation::Scalar),
            "OBJECT" => Ok(legacy_ast::schema::DirectiveLocation::Object),
            "FIELD_DEFINITION" => Ok(legacy_ast::schema::DirectiveLocation::FieldDefinition),
            "ARGUMENT_DEFINITION" => {
                Ok(legacy_ast::schema::DirectiveLocation::ArgumentDefinition)
            }
            "INTERFACE" => Ok(legacy_ast::schema::DirectiveLocation::Interface),
            "UNION" => Ok(legacy_ast::schema::DirectiveLocation::Union),
            "ENUM" => Ok(legacy_ast::schema::DirectiveLocation::Enum),
            "ENUM_VALUE" => Ok(legacy_ast::schema::DirectiveLocation::EnumValue),
            "INPUT_OBJECT" => Ok(legacy_ast::schema::DirectiveLocation::InputObject),
            "INPUT_FIELD_DEFINITION" => {
                Ok(legacy_ast::schema::DirectiveLocation::InputFieldDefinition)
            }

            _ => {
                // Unknown location - try to suggest closest match
                let mut error = GraphQLParseError::new(
                    format!("unknown directive location `{name}`"),
                    span,
                    GraphQLParseErrorKind::InvalidSyntax,
                );

                if let Some(suggestion) = Self::suggest_directive_location(&name) {
                    error.add_help(format!("did you mean `{suggestion}`?"));
                }

                self.record_error(error);
                Err(())
            }
        }
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

    /// Parses a type annotation for schema definitions.
    fn parse_schema_type_annotation(
        &mut self,
    ) -> Result<legacy_ast::schema::Type, ()> {
        self.enter_recursion()?;
        let result = self.parse_schema_type_annotation_impl();
        self.exit_recursion();
        result
    }

    /// Inner implementation of schema type annotation parsing.
    fn parse_schema_type_annotation_impl(
        &mut self,
    ) -> Result<legacy_ast::schema::Type, ()> {
        let base_type = if self.peek_is(&GraphQLTokenKind::SquareBracketOpen) {
            self.parse_schema_list_type()?
        } else {
            let name = self.expect_name_only()?;
            legacy_ast::schema::Type::NamedType(name.into_owned())
        };

        if self.peek_is(&GraphQLTokenKind::Bang) {
            self.consume_token();
            Ok(legacy_ast::schema::Type::NonNullType(Box::new(base_type)))
        } else {
            Ok(base_type)
        }
    }

    /// Parses a list type for schema definitions.
    fn parse_schema_list_type(&mut self) -> Result<legacy_ast::schema::Type, ()> {
        let open_token = self.expect(&GraphQLTokenKind::SquareBracketOpen)?;
        self.push_delimiter(open_token.span.clone(), DelimiterContext::ListType);

        let inner = self.parse_schema_type_annotation()?;

        self.expect(&GraphQLTokenKind::SquareBracketClose)?;
        self.pop_delimiter();

        Ok(legacy_ast::schema::Type::ListType(Box::new(inner)))
    }

    /// Convert operation directives to schema directives.
    fn convert_directives_to_schema(
        &self,
        directives: Vec<legacy_ast::operation::Directive>,
    ) -> Vec<legacy_ast::schema::Directive> {
        directives
            .into_iter()
            .map(|d| legacy_ast::schema::Directive {
                position: d.position,
                name: d.name,
                arguments: d.arguments,
            })
            .collect()
    }

    // =========================================================================
    // Type extension parsing
    // =========================================================================

    /// Parses a type extension.
    ///
    /// Note: Schema extensions (`extend schema`) are valid GraphQL but not
    /// supported by the underlying graphql_parser crate's AST.
    /// TODO: Support schema extensions when we have a custom AST.
    fn parse_type_extension(&mut self) -> Result<legacy_ast::schema::TypeExtension, ()> {
        // Performance: Extract AstPos (16 bytes, Copy) immediately from the
        // span rather than storing the full GraphQLSourceSpan (~104 bytes with
        // Option<PathBuf>). Pass AstPos by value (Copy) to helper methods.
        let extend_pos = self
            .expect_keyword("extend")?
            .start_inclusive
            .to_ast_pos();

        if self.peek_is_keyword("schema") {
            // Schema extensions are valid GraphQL but not supported by
            // graphql_parser crate's AST.
            // TODO: Support schema extensions when we have a custom AST.
            self.skip_schema_extension()?;
            Err(())
        } else if self.peek_is_keyword("scalar") {
            // Performance: Pass AstPos by value (Copy, 16 bytes) rather than
            // GraphQLSourceSpan by reference, as the callee only needs the
            // position.
            self.parse_scalar_type_extension(extend_pos)
        } else if self.peek_is_keyword("type") {
            self.parse_object_type_extension(extend_pos)
        } else if self.peek_is_keyword("interface") {
            self.parse_interface_type_extension(extend_pos)
        } else if self.peek_is_keyword("union") {
            self.parse_union_type_extension(extend_pos)
        } else if self.peek_is_keyword("enum") {
            self.parse_enum_type_extension(extend_pos)
        } else if self.peek_is_keyword("input") {
            self.parse_input_object_type_extension(extend_pos)
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

    /// Skips a schema extension, recording an error that it's unsupported.
    ///
    /// Schema extensions are valid GraphQL but the graphql_parser crate's AST
    /// doesn't have a representation for them.
    /// TODO: Support schema extensions when we have a custom AST.
    fn skip_schema_extension(&mut self) -> Result<(), ()> {
        let start_span = self
            .token_stream.peek()
            .map(|t| t.span.clone())
            .unwrap_or_else(|| self.eof_span());

        self.expect_keyword("schema")?;

        // Record error for unsupported feature
        self.record_error(GraphQLParseError::new(
            "schema extensions (`extend schema`) are not yet supported".to_string(),
            start_span,
            GraphQLParseErrorKind::InvalidSyntax,
        ));

        // Skip directives
        let _ = self.parse_const_directive_annotations();

        // Skip body if present
        if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            let open_token = self.expect(&GraphQLTokenKind::CurlyBraceOpen)?;
            self.push_delimiter(
                open_token.span.clone(),
                DelimiterContext::SchemaDefinition,
            );

            loop {
                if self.peek_is(&GraphQLTokenKind::CurlyBraceClose) {
                    break;
                }
                if self.token_stream.is_at_end() {
                    self.handle_unclosed_brace();
                    return Err(());
                }

                // Skip: operation_type : type_name
                let _ = self.expect_name();
                let _ = self.expect(&GraphQLTokenKind::Colon);
                let _ = self.expect_name();
            }

            self.expect(&GraphQLTokenKind::CurlyBraceClose)?;
            self.pop_delimiter();
        }

        Ok(())
    }

    /// Parses a scalar type extension: `extend scalar Name @directives`
    ///
    /// # Arguments
    /// * `position` - The position of the `extend` keyword, passed as `AstPos`
    ///   (Copy, 16 bytes) rather than `GraphQLSourceSpan` (~104 bytes, contains
    ///   `Option<PathBuf>`) to avoid unnecessary allocation/copying of the full
    ///   span when only the start position is needed for the AST node.
    fn parse_scalar_type_extension(
        &mut self,
        position: legacy_ast::AstPos,
    ) -> Result<legacy_ast::schema::TypeExtension, ()> {
        self.expect_keyword("scalar")?;
        let name = self.expect_name_only()?;
        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        Ok(legacy_ast::schema::TypeExtension::Scalar(
            legacy_ast::schema::ScalarTypeExtension {
                position,
                name: name.into_owned(),
                directives: schema_directives,
            },
        ))
    }

    /// Parses an object type extension: `extend type Name implements I & J
    /// @directives { fields }`
    ///
    /// # Arguments
    /// * `position` - The position of the `extend` keyword, passed as `AstPos`
    ///   (Copy, 16 bytes) rather than `GraphQLSourceSpan` (~104 bytes, contains
    ///   `Option<PathBuf>`) to avoid unnecessary allocation/copying of the full
    ///   span when only the start position is needed for the AST node.
    fn parse_object_type_extension(
        &mut self,
        position: legacy_ast::AstPos,
    ) -> Result<legacy_ast::schema::TypeExtension, ()> {
        self.expect_keyword("type")?;
        let name = self.expect_name_only()?;

        let implements_interfaces = if self.peek_is_keyword("implements") {
            self.parse_implements_interfaces()?
        } else {
            Vec::new()
        };

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        let fields = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_fields_definition(DelimiterContext::ObjectTypeDefinition)?
        } else {
            Vec::new()
        };

        Ok(legacy_ast::schema::TypeExtension::Object(
            legacy_ast::schema::ObjectTypeExtension {
                position,
                name: name.into_owned(),
                implements_interfaces,
                directives: schema_directives,
                fields,
            },
        ))
    }

    /// Parses an interface type extension.
    ///
    /// # Arguments
    /// * `position` - The position of the `extend` keyword, passed as `AstPos`
    ///   (Copy, 16 bytes) rather than `GraphQLSourceSpan` (~104 bytes, contains
    ///   `Option<PathBuf>`) to avoid unnecessary allocation/copying of the full
    ///   span when only the start position is needed for the AST node.
    fn parse_interface_type_extension(
        &mut self,
        position: legacy_ast::AstPos,
    ) -> Result<legacy_ast::schema::TypeExtension, ()> {
        self.expect_keyword("interface")?;
        let name = self.expect_name_only()?;

        let implements_interfaces = if self.peek_is_keyword("implements") {
            self.parse_implements_interfaces()?
        } else {
            Vec::new()
        };

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        let fields = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_fields_definition(DelimiterContext::InterfaceDefinition)?
        } else {
            Vec::new()
        };

        Ok(legacy_ast::schema::TypeExtension::Interface(
            legacy_ast::schema::InterfaceTypeExtension {
                position,
                name: name.into_owned(),
                implements_interfaces,
                directives: schema_directives,
                fields,
            },
        ))
    }

    /// Parses a union type extension: `extend union Name @directives = A | B`
    ///
    /// # Arguments
    /// * `position` - The position of the `extend` keyword, passed as `AstPos`
    ///   (Copy, 16 bytes) rather than `GraphQLSourceSpan` (~104 bytes, contains
    ///   `Option<PathBuf>`) to avoid unnecessary allocation/copying of the full
    ///   span when only the start position is needed for the AST node.
    fn parse_union_type_extension(
        &mut self,
        position: legacy_ast::AstPos,
    ) -> Result<legacy_ast::schema::TypeExtension, ()> {
        self.expect_keyword("union")?;
        let name = self.expect_name_only()?;

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        let mut types = Vec::new();
        if self.peek_is(&GraphQLTokenKind::Equals) {
            self.consume_token();

            if self.peek_is(&GraphQLTokenKind::Pipe) {
                self.consume_token();
            }

            let first_type = self.expect_name_only()?;
            types.push(first_type.into_owned());

            while self.peek_is(&GraphQLTokenKind::Pipe) {
                self.consume_token();
                let member_type = self.expect_name_only()?;
                types.push(member_type.into_owned());
            }
        }

        Ok(legacy_ast::schema::TypeExtension::Union(
            legacy_ast::schema::UnionTypeExtension {
                position,
                name: name.into_owned(),
                directives: schema_directives,
                types,
            },
        ))
    }

    /// Parses an enum type extension: `extend enum Name @directives { VALUES }`
    ///
    /// # Arguments
    /// * `position` - The position of the `extend` keyword, passed as `AstPos`
    ///   (Copy, 16 bytes) rather than `GraphQLSourceSpan` (~104 bytes, contains
    ///   `Option<PathBuf>`) to avoid unnecessary allocation/copying of the full
    ///   span when only the start position is needed for the AST node.
    fn parse_enum_type_extension(
        &mut self,
        position: legacy_ast::AstPos,
    ) -> Result<legacy_ast::schema::TypeExtension, ()> {
        self.expect_keyword("enum")?;
        let name = self.expect_name_only()?;

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        let values = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_enum_values_definition()?
        } else {
            Vec::new()
        };

        Ok(legacy_ast::schema::TypeExtension::Enum(
            legacy_ast::schema::EnumTypeExtension {
                position,
                name: name.into_owned(),
                directives: schema_directives,
                values,
            },
        ))
    }

    /// Parses an input object type extension.
    ///
    /// # Arguments
    /// * `position` - The position of the `extend` keyword, passed as `AstPos`
    ///   (Copy, 16 bytes) rather than `GraphQLSourceSpan` (~104 bytes, contains
    ///   `Option<PathBuf>`) to avoid unnecessary allocation/copying of the full
    ///   span when only the start position is needed for the AST node.
    fn parse_input_object_type_extension(
        &mut self,
        position: legacy_ast::AstPos,
    ) -> Result<legacy_ast::schema::TypeExtension, ()> {
        self.expect_keyword("input")?;
        let name = self.expect_name_only()?;

        let directives = self.parse_const_directive_annotations()?;
        let schema_directives = self.convert_directives_to_schema(directives);

        let fields = if self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            self.parse_input_fields_definition()?
        } else {
            Vec::new()
        };

        Ok(legacy_ast::schema::TypeExtension::InputObject(
            legacy_ast::schema::InputObjectTypeExtension {
                position,
                name: name.into_owned(),
                directives: schema_directives,
                fields,
            },
        ))
    }

    // =========================================================================
    // Document parsing (public API)
    // =========================================================================

    /// Parses a schema document (type system definitions only).
    pub fn parse_schema_document(mut self) -> ParseResult<legacy_ast::schema::Document> {
        let mut definitions = Vec::new();

        while !self.token_stream.is_at_end() {
            match self.parse_schema_definition_item() {
                Ok(def) => definitions.push(def),
                Err(()) => {
                    self.recover_to_next_definition();
                }
            }
        }

        let document = legacy_ast::schema::Document { definitions };

        if self.errors.is_empty() {
            ParseResult::ok(document)
        } else {
            ParseResult::recovered(document, self.errors)
        }
    }

    /// Parses an executable document (operations and fragments only).
    pub fn parse_executable_document(
        mut self,
    ) -> ParseResult<legacy_ast::operation::Document> {
        let mut definitions = Vec::new();

        while !self.token_stream.is_at_end() {
            match self.parse_executable_definition_item() {
                Ok(def) => definitions.push(def),
                Err(()) => {
                    self.recover_to_next_definition();
                }
            }
        }

        let document = legacy_ast::operation::Document { definitions };

        if self.errors.is_empty() {
            ParseResult::ok(document)
        } else {
            ParseResult::recovered(document, self.errors)
        }
    }

    /// Parses a mixed document (both type system and executable definitions).
    pub fn parse_mixed_document(mut self) -> ParseResult<legacy_ast::MixedDocument> {
        let mut definitions = Vec::new();

        while !self.token_stream.is_at_end() {
            match self.parse_mixed_definition_item() {
                Ok(def) => definitions.push(def),
                Err(()) => {
                    self.recover_to_next_definition();
                }
            }
        }

        let document = legacy_ast::MixedDocument { definitions };

        if self.errors.is_empty() {
            ParseResult::ok(document)
        } else {
            ParseResult::recovered(document, self.errors)
        }
    }

    /// Parses a single schema definition item.
    fn parse_schema_definition_item(&mut self) -> Result<legacy_ast::schema::Definition, ()> {
        // Handle lexer errors
        if let Some(token) = self.token_stream.peek()
            && let GraphQLTokenKind::Error { .. } = &token.kind {
                let token = token.clone();
                self.handle_lexer_error(&token);
                self.consume_token();
                return Err(());
            }

        let description = self.parse_description();

        if self.peek_is_keyword("schema") {
            Ok(legacy_ast::schema::Definition::SchemaDefinition(
                self.parse_schema_definition()?,
            ))
        } else if self.peek_is_keyword("scalar") {
            Ok(legacy_ast::schema::Definition::TypeDefinition(
                self.parse_scalar_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("type") {
            Ok(legacy_ast::schema::Definition::TypeDefinition(
                self.parse_object_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("interface") {
            Ok(legacy_ast::schema::Definition::TypeDefinition(
                self.parse_interface_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("union") {
            Ok(legacy_ast::schema::Definition::TypeDefinition(
                self.parse_union_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("enum") {
            Ok(legacy_ast::schema::Definition::TypeDefinition(
                self.parse_enum_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("input") {
            Ok(legacy_ast::schema::Definition::TypeDefinition(
                self.parse_input_object_type_definition(description)?,
            ))
        } else if self.peek_is_keyword("directive") {
            Ok(legacy_ast::schema::Definition::DirectiveDefinition(
                self.parse_directive_definition(description)?,
            ))
        } else if self.peek_is_keyword("extend") {
            Ok(legacy_ast::schema::Definition::TypeExtension(
                self.parse_type_extension()?,
            ))
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
    fn parse_executable_definition_item(
        &mut self,
    ) -> Result<legacy_ast::operation::Definition, ()> {
        // Handle lexer errors
        if let Some(token) = self.token_stream.peek()
            && let GraphQLTokenKind::Error { .. } = &token.kind {
                let token = token.clone();
                self.handle_lexer_error(&token);
                self.consume_token();
                return Err(());
            }

        if self.peek_is_keyword("query")
            || self.peek_is_keyword("mutation")
            || self.peek_is_keyword("subscription")
            || self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            Ok(legacy_ast::operation::Definition::Operation(
                self.parse_operation_definition()?,
            ))
        } else if self.peek_is_keyword("fragment") {
            Ok(legacy_ast::operation::Definition::Fragment(
                self.parse_fragment_definition()?,
            ))
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
    fn parse_mixed_definition_item(&mut self) -> Result<legacy_ast::MixedDefinition, ()> {
        // Handle lexer errors
        if let Some(token) = self.token_stream.peek()
            && let GraphQLTokenKind::Error { .. } = &token.kind {
                let token = token.clone();
                self.handle_lexer_error(&token);
                self.consume_token();
                return Err(());
            }

        let description = self.parse_description();

        // Schema definitions
        if self.peek_is_keyword("schema") {
            return Ok(legacy_ast::MixedDefinition::Schema(
                legacy_ast::schema::Definition::SchemaDefinition(self.parse_schema_definition()?),
            ));
        }
        if self.peek_is_keyword("scalar") {
            return Ok(legacy_ast::MixedDefinition::Schema(
                legacy_ast::schema::Definition::TypeDefinition(
                    self.parse_scalar_type_definition(description)?,
                ),
            ));
        }
        if self.peek_is_keyword("type") {
            return Ok(legacy_ast::MixedDefinition::Schema(
                legacy_ast::schema::Definition::TypeDefinition(
                    self.parse_object_type_definition(description)?,
                ),
            ));
        }
        if self.peek_is_keyword("interface") {
            return Ok(legacy_ast::MixedDefinition::Schema(
                legacy_ast::schema::Definition::TypeDefinition(
                    self.parse_interface_type_definition(description)?,
                ),
            ));
        }
        if self.peek_is_keyword("union") {
            return Ok(legacy_ast::MixedDefinition::Schema(
                legacy_ast::schema::Definition::TypeDefinition(
                    self.parse_union_type_definition(description)?,
                ),
            ));
        }
        if self.peek_is_keyword("enum") {
            return Ok(legacy_ast::MixedDefinition::Schema(
                legacy_ast::schema::Definition::TypeDefinition(
                    self.parse_enum_type_definition(description)?,
                ),
            ));
        }
        if self.peek_is_keyword("input") {
            return Ok(legacy_ast::MixedDefinition::Schema(
                legacy_ast::schema::Definition::TypeDefinition(
                    self.parse_input_object_type_definition(description)?,
                ),
            ));
        }
        if self.peek_is_keyword("directive") {
            return Ok(legacy_ast::MixedDefinition::Schema(
                legacy_ast::schema::Definition::DirectiveDefinition(
                    self.parse_directive_definition(description)?,
                ),
            ));
        }
        if self.peek_is_keyword("extend") {
            return Ok(legacy_ast::MixedDefinition::Schema(
                legacy_ast::schema::Definition::TypeExtension(self.parse_type_extension()?),
            ));
        }

        // Executable definitions
        if self.peek_is_keyword("query")
            || self.peek_is_keyword("mutation")
            || self.peek_is_keyword("subscription")
            || self.peek_is(&GraphQLTokenKind::CurlyBraceOpen) {
            return Ok(legacy_ast::MixedDefinition::Executable(
                legacy_ast::operation::Definition::Operation(self.parse_operation_definition()?),
            ));
        }
        if self.peek_is_keyword("fragment") {
            return Ok(legacy_ast::MixedDefinition::Executable(
                legacy_ast::operation::Definition::Fragment(self.parse_fragment_definition()?),
            ));
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
