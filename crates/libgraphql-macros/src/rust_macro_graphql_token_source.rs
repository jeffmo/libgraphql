//! A GraphQL token source that reads and translates from Rust proc-macro token
//! streams into a [`GraphQLTokenSource`](libgraphql_parser::GraphQLTokenSource).
//!
//! # Limitations
//!
//! Due to how Rust's tokenizer works, this token source has inherent
//! limitations:
//!
//! - **No Rust comment preservation**: Rust strips comments (`// ...`) before
//!   tokens reach proc macros, so `preceding_trivia` will only contain `Comma`
//!   tokens, never `Comment` tokens. Note that GraphQL uses `#` for comments,
//!   but since GraphQL is embedded in Rust macro syntax here, users might write
//!   Rust-style comments which are lost.
//! - **Limited Rust whitespace information**: Whitespace is not tokenized by
//!   Rust, so position information for [`GraphQLToken`]s is derived from
//!   `proc_macro2::Span` values rather than character-by-character scanning.
//!
//! For use cases requiring comment preservation (formatters, linters), use
//! `StrToGraphQLTokenSource` with the original source text instead.
//!
//! # Notes
//!
//! Rust macros only report `byte_offset`s properly when built with Rust nightly
//! toolchains. At the time of this writing stable rustc toolchains do not
//! provide accurate or meaningful output for `proc_macro::Span::byte_range()`.
//!
//! See: <https://github.com/rust-lang/rust/issues/54725>
//!
//! TODO: It would be good to add something that emits a warning with a clear
//! description of caveats when using `libgraphql-macros` with a non-nightly (or
//! otherwise incompatible) Rust toolchain.
//!
//! e.g. build_dependency on `rustc_version` -> build.rs file that uses
//! `rustc_version::version_meta()` to emit
//! `"cargo:rustc-cfg=libgraphql_rustc_nightly"` when on nightly.

use libgraphql_parser::smallvec;
use libgraphql_parser::token::GraphQLToken;
use libgraphql_parser::token::GraphQLTokenKind;
use libgraphql_parser::GraphQLSourceSpan;
use libgraphql_parser::token::GraphQLTriviaToken;
use libgraphql_parser::token::GraphQLTriviaTokenVec;
use libgraphql_parser::SourcePosition;
use proc_macro2::Delimiter;
use proc_macro2::Group;
use proc_macro2::Ident;
use proc_macro2::Literal;
use proc_macro2::Punct;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use std::iter::Peekable;

/// Sentinel error message for a single `.` token.
///
/// This message is used when emitting error tokens for isolated `.` punctuation
/// and when checking whether pending tokens should be combined into an
/// `Ellipsis` (`...`) token or a `..` error token.
const DOT_ERROR_MSG: &str = "Unexpected token: `.` (use `...` for spread operator)";

/// Sentinel error message for an adjacent `..` token sequence.
///
/// This message is used when two adjacent `.` tokens are detected. This is
/// still a "pending" state - if a third adjacent `.` follows, they will be
/// combined into an `Ellipsis` token.
const DOUBLE_DOT_ERROR_MSG: &str =
    "Unexpected token: `..` (use `...` for spread operator)";

/// Sentinel error message for a non-adjacent `. .` token sequence on same line.
///
/// This is a terminal error state - it will NOT be combined with subsequent
/// dots into an `Ellipsis` because the spacing indicates this wasn't intended
/// to be `...`. Includes an error note suggesting to remove spacing.
const SPACED_DOT_DOT_ERROR_MSG: &str =
    "Unexpected token: `. .` (use `...` for spread operator)";

/// Error message for a pending `-` that might be part of a negative number.
///
/// In GraphQL, negative numbers like `-17` are valid IntValue tokens. However,
/// Rust tokenizes `-17` as two separate tokens: `Punct('-')` and `Literal(17)`.
/// We store the `-` as an error with this message, then check if the next token
/// is a number and combine them in `try_combine_negative_number()`.
const PENDING_MINUS_ERROR_MSG: &str = "Unexpected token: `-`";

/// A GraphQL token source that reads and translates from Rust proc-macro token
/// streams into a [`GraphQLTokenSource`].
///
/// This implements `Iterator<Item = GraphQLToken>`, making it compatible with
/// [`libgraphql_parser::GraphQLTokenStream`].
///
/// See module documentation for limitations.
pub(crate) struct RustMacroGraphQLTokenSource {
    tokens: Peekable<proc_macro2::token_stream::IntoIter>,
    /// Buffer for tokens generated from processing a single Rust token tree.
    /// For example, a `Group` generates open bracket, contents, close bracket.
    pending: Vec<PendingToken>,
    /// Trivia (commas) accumulated before the next non-trivia token.
    pending_trivia: GraphQLTriviaTokenVec,
    /// Whether we've emitted the Eof token.
    finished: bool,
    /// The span of the last token we saw (used for Eof span).
    last_span: Option<Span>,
}

/// Internal representation of a pending token with its trivia already attached.
struct PendingToken {
    kind: GraphQLTokenKind,
    /// Trivia (commas) that precede this token.
    preceding_trivia: GraphQLTriviaTokenVec,
    /// The primary span for this token.
    span: Span,
    /// Optional ending span for multi-token sequences (e.g., `..`).
    /// When present, this span's end position is used instead of `span`'s end.
    ending_span: Option<Span>,
}

impl From<PendingToken> for GraphQLToken {
    fn from(pending: PendingToken) -> Self {
        let span = RustMacroGraphQLTokenSource::make_pending_source_span(&pending);
        GraphQLToken {
            kind: pending.kind,
            preceding_trivia: pending.preceding_trivia,
            span,
        }
    }
}

impl RustMacroGraphQLTokenSource {
    /// Creates a new token source from a proc-macro token stream.
    pub fn new(input: TokenStream) -> Self {
        Self {
            tokens: input.into_iter().peekable(),
            pending: Vec::new(),
            pending_trivia: smallvec![],
            finished: false,
            last_span: None,
        }
    }

    /// Convert a `proc_macro2::Span` start position to a `SourcePosition`.
    ///
    /// We use `span.start()` and `span.end()` instead of `span.byte_range()`
    /// because:
    /// 1. `byte_range()` only works reliably on nightly Rust (see module docs)
    /// 2. `byte_range()` returns raw byte offsets which don't account for
    ///    character encoding, making them less useful for user-facing positions
    /// 3. `span.start()`/`span.end()` were stabilized in Rust 1.88.0 and
    ///    provide line/column information that's more useful for error messages
    ///
    /// **Important:** `proc_macro2::Span` only provides UTF-8 char-based column
    /// positions, not UTF-16 code unit offsets. We pass the char offset as
    /// `col_utf8` since it is the UTF-8 char offset, but we don't have the
    /// UTF-16 col offset so we pass `None`.
    fn source_position_from_span_start(span: &Span) -> SourcePosition {
        let start = span.start();
        // proc_macro2 uses 1-based lines, we use 0-based
        // proc_macro2 column is already 0-based and is a UTF-8 char offset
        SourcePosition::new(
            start.line.saturating_sub(1),
            start.column,
            None, // UTF-16 column not available from proc_macro2
            span.byte_range().start,
        )
    }

    fn source_position_from_span_end(span: &Span) -> SourcePosition {
        let end = span.end();
        SourcePosition::new(
            end.line.saturating_sub(1),
            end.column,
            None,
            span.byte_range().end,
        )
    }

    fn make_source_span(span: &Span) -> GraphQLSourceSpan {
        GraphQLSourceSpan {
            start_inclusive: Self::source_position_from_span_start(span),
            end_exclusive: Self::source_position_from_span_end(span),
        }
    }

    /// Creates a `GraphQLSourceSpan` from a `PendingToken`, using `ending_span`
    /// if present for the end position.
    fn make_pending_source_span(pending: &PendingToken) -> GraphQLSourceSpan {
        let end_span = pending.ending_span.as_ref().unwrap_or(&pending.span);
        GraphQLSourceSpan {
            start_inclusive: Self::source_position_from_span_start(&pending.span),
            end_exclusive: Self::source_position_from_span_end(end_span),
        }
    }

    /// Creates a `PendingToken` with the current accumulated trivia.
    ///
    /// This takes ownership of `pending_trivia` and attaches it to the token,
    /// ensuring trivia is correctly associated with the token that follows it.
    fn make_pending_token(&mut self, kind: GraphQLTokenKind, span: Span) -> PendingToken {
        PendingToken {
            kind,
            preceding_trivia: std::mem::take(&mut self.pending_trivia),
            span,
            ending_span: None,
        }
    }

    /// Checks if a string literal is a Rust raw string (e.g., `r"..."`,
    /// `r#"..."#`).
    ///
    /// Raw strings are Rust-specific syntax with no GraphQL equivalent, so they
    /// should be rejected with a helpful error message.
    fn is_raw_string(s: &str) -> bool {
        s.starts_with("r\"") || s.starts_with("r#")
    }

    /// Checks if two spans are adjacent (end of first == start of second).
    ///
    /// This is used for detecting GraphQL block strings (`"""..."""`) which
    /// Rust tokenizes as three separate string literals. We need to verify
    /// they're actually adjacent with no whitespace between them.
    ///
    /// We use line/column comparison instead of byte offsets because:
    /// 1. `byte_range()` is unreliable on stable Rust
    /// 2. Line/column provides meaningful position info regardless of encoding
    fn spans_are_adjacent(first: &Span, second: &Span) -> bool {
        let first_end = first.end();
        let second_start = second.start();
        first_end.line == second_start.line
            && first_end.column == second_start.column
    }

    /// Checks if two spans are on the same line.
    fn spans_on_same_line(first: &Span, second: &Span) -> bool {
        first.start().line == second.start().line
    }

    /// Processes a token tree, delegating to token-specific handlers.
    fn process_token_tree(&mut self, tree: TokenTree) {
        match tree {
            TokenTree::Group(group) => self.process_group_token(group),
            TokenTree::Ident(ident) => self.process_ident_token(ident),
            TokenTree::Punct(punct) => self.process_punct_token(punct),
            TokenTree::Literal(lit) => self.process_literal_token(lit),
        }
    }

    /// Processes a `Group` token (delimited by `{}`, `[]`, `()`, or invisible).
    fn process_group_token(&mut self, group: Group) {
        let span = group.span();
        match group.delimiter() {
            Delimiter::Brace => {
                let open = self.make_pending_token(GraphQLTokenKind::CurlyBraceOpen, span);
                self.pending.push(open);
                for inner in group.stream() {
                    self.process_token_tree(inner);
                }
                let close =
                    self.make_pending_token(GraphQLTokenKind::CurlyBraceClose, span);
                self.pending.push(close);
            }
            Delimiter::Bracket => {
                let open =
                    self.make_pending_token(GraphQLTokenKind::SquareBracketOpen, span);
                self.pending.push(open);
                for inner in group.stream() {
                    self.process_token_tree(inner);
                }
                let close =
                    self.make_pending_token(GraphQLTokenKind::SquareBracketClose, span);
                self.pending.push(close);
            }
            Delimiter::Parenthesis => {
                let open = self.make_pending_token(GraphQLTokenKind::ParenOpen, span);
                self.pending.push(open);
                for inner in group.stream() {
                    self.process_token_tree(inner);
                }
                let close = self.make_pending_token(GraphQLTokenKind::ParenClose, span);
                self.pending.push(close);
            }
            Delimiter::None => {
                for inner in group.stream() {
                    self.process_token_tree(inner);
                }
            }
        }
    }

    /// Processes an `Ident` token (identifier or keyword).
    ///
    /// Note: We emit `GraphQLTokenKind::Name` for identifiers, but we do NOT
    /// validate that the name conforms to the GraphQL "Name" specification
    /// (https://spec.graphql.org/September2025/#Name). That validation is the
    /// responsibility of the `GraphQLToken` consumer (e.g., `GraphQLTokenStream`
    /// or `GraphQLParser`).
    fn process_ident_token(&mut self, ident: Ident) {
        let span = ident.span();
        let name = ident.to_string();

        // Check for special keywords that get distinct token kinds
        let kind = match name.as_str() {
            "true" => GraphQLTokenKind::True,
            "false" => GraphQLTokenKind::False,
            "null" => GraphQLTokenKind::Null,
            _ => GraphQLTokenKind::Name(name),
        };

        let token = self.make_pending_token(kind, span);
        self.pending.push(token);
    }

    /// Processes a `Punct` token (single punctuation character).
    fn process_punct_token(&mut self, punct: Punct) {
        let span = punct.span();
        let ch = punct.as_char();

        match ch {
            '.' => self.process_dot_punct(span),
            '-' => {
                // Minus sign might be part of a negative number (e.g., `-17`).
                // Store it as an error token - if followed by a number, we'll
                // combine them in try_combine_negative_number(). If not followed
                // by a number, it remains as an error token.
                let kind = GraphQLTokenKind::Error {
                    message: PENDING_MINUS_ERROR_MSG.to_string(),
                    error_notes: smallvec![],
                };
                let token = self.make_pending_token(kind, span);
                self.pending.push(token);
            }
            '!' => {
                let token = self.make_pending_token(GraphQLTokenKind::Bang, span);
                self.pending.push(token);
            }
            '$' => {
                let token = self.make_pending_token(GraphQLTokenKind::Dollar, span);
                self.pending.push(token);
            }
            '&' => {
                let token = self.make_pending_token(GraphQLTokenKind::Ampersand, span);
                self.pending.push(token);
            }
            ':' => {
                let token = self.make_pending_token(GraphQLTokenKind::Colon, span);
                self.pending.push(token);
            }
            '=' => {
                let token = self.make_pending_token(GraphQLTokenKind::Equals, span);
                self.pending.push(token);
            }
            '@' => {
                let token = self.make_pending_token(GraphQLTokenKind::At, span);
                self.pending.push(token);
            }
            '|' => {
                let token = self.make_pending_token(GraphQLTokenKind::Pipe, span);
                self.pending.push(token);
            }
            ',' => {
                // Comma is trivia - don't add to pending, track separately
                self.pending_trivia.push(GraphQLTriviaToken::Comma {
                    span: Self::make_source_span(&span),
                });
            }
            _ => {
                // Other punctuation - emit as error token
                let kind = GraphQLTokenKind::Error {
                    message: format!("Unexpected token: `{ch}`"),
                    error_notes: smallvec![],
                };
                let token = self.make_pending_token(kind, span);
                self.pending.push(token);
            }
        }
    }

    /// Processes a `.` punctuation character, potentially combining into `...`.
    ///
    /// This function handles these cases:
    /// 1. Third adjacent `.` after `..` → emit `Ellipsis`
    /// 2. Third non-adjacent `.` after `..` on same line → terminal error with
    ///    note about spacing
    /// 3. Third non-adjacent `.` after `..` on different line → keep `..` error,
    ///    emit new `.` error
    /// 4. Any `.` after terminal `. .` error → new `.` error (or merge if same
    ///    line with helpful note)
    /// 5. Second adjacent `.` after `.` → `..` error (can still become `...`)
    /// 6. Second non-adjacent `.` after `.` on same line → terminal `. .` error
    ///    with note about spacing
    /// 7. Second non-adjacent `.` after `.` on different line → leave first `.`
    ///    as error, emit new `.` error
    /// 8. First `.` → single `.` error
    fn process_dot_punct(&mut self, span: Span) {
        // Helper to check if a pending token is a single-dot error
        let is_single_dot_error = |pt: &PendingToken| {
            matches!(
                &pt.kind,
                GraphQLTokenKind::Error { message, .. } if message == DOT_ERROR_MSG
            )
        };

        // Helper to check if a pending token is an adjacent double-dot error
        // (can still become `...` if followed by adjacent `.`)
        let is_double_dot_error = |pt: &PendingToken| {
            matches!(
                &pt.kind,
                GraphQLTokenKind::Error { message, .. } if message == DOUBLE_DOT_ERROR_MSG
            )
        };

        // Helper to check if a pending token is a spaced double-dot error
        // (terminal - cannot become `...`)
        let is_spaced_double_dot_error = |pt: &PendingToken| {
            matches!(
                &pt.kind,
                GraphQLTokenKind::Error { message, .. }
                    if message == SPACED_DOT_DOT_ERROR_MSG
            )
        };

        if let Some(last) = self.pending.last() {
            // Check if previous token is an adjacent double-dot error (`..`)
            if is_double_dot_error(last) {
                let last_end = last.ending_span.as_ref().unwrap_or(&last.span);
                if Self::spans_are_adjacent(last_end, &span) {
                    // Third adjacent dot - complete the ellipsis!
                    let prev = self.pending.pop().unwrap();
                    self.pending.push(PendingToken {
                        kind: GraphQLTokenKind::Ellipsis,
                        preceding_trivia: prev.preceding_trivia,
                        span: prev.span,
                        ending_span: Some(span),
                    });
                    return;
                } else if Self::spans_on_same_line(last_end, &span) {
                    // Third dot on same line but not adjacent to `..`
                    // This looks like `.. .` - provide helpful error about spacing
                    let prev = self.pending.pop().unwrap();
                    self.pending.push(PendingToken {
                        kind: GraphQLTokenKind::Error {
                            message: "Unexpected token: `.. .`".to_string(),
                            error_notes: smallvec![(
                                "This `.` may have been intended to complete a `...` spread \
                                 operator. Try removing the extra spacing between the dots."
                                    .to_string(),
                                Some(Self::make_source_span(&span))
                            )],
                        },
                        preceding_trivia: prev.preceding_trivia,
                        span: prev.span,
                        ending_span: Some(span),
                    });
                    return;
                } else {
                    // Third dot on different line - leave `..` as separate error
                    // and emit new single dot error
                    let kind = GraphQLTokenKind::Error {
                        message: DOT_ERROR_MSG.to_string(),
                        error_notes: smallvec![],
                    };
                    let token = self.make_pending_token(kind, span);
                    self.pending.push(token);
                    return;
                }
            }

            // Check if previous token is a spaced double-dot error (`. .`)
            // This is terminal - we won't combine it into `...`
            if is_spaced_double_dot_error(last) {
                let last_end = last.ending_span.as_ref().unwrap_or(&last.span);
                if Self::spans_on_same_line(last_end, &span) {
                    // Third dot on same line after `. .` - this is `. . .`
                    // Merge into single error with helpful note
                    let prev = self.pending.pop().unwrap();
                    self.pending.push(PendingToken {
                        kind: GraphQLTokenKind::Error {
                            message: "Unexpected token: `. . .`".to_string(),
                            error_notes: smallvec![(
                                "These dots may have been intended to form a `...` spread \
                                 operator. Try removing the extra spacing between the dots."
                                    .to_string(),
                                Some(Self::make_source_span(&span))
                            )],
                        },
                        preceding_trivia: prev.preceding_trivia,
                        span: prev.span,
                        ending_span: Some(span),
                    });
                    return;
                } else {
                    // Third dot on different line - emit new single dot error
                    let kind = GraphQLTokenKind::Error {
                        message: DOT_ERROR_MSG.to_string(),
                        error_notes: smallvec![],
                    };
                    let token = self.make_pending_token(kind, span);
                    self.pending.push(token);
                    return;
                }
            }

            // Check if previous token is a single-dot error (`.`)
            if is_single_dot_error(last) {
                if Self::spans_are_adjacent(&last.span, &span) {
                    // Second adjacent dot - combine into `..` error
                    // This can still become `...` if followed by adjacent `.`
                    let prev = self.pending.pop().unwrap();
                    self.pending.push(PendingToken {
                        kind: GraphQLTokenKind::Error {
                            message: DOUBLE_DOT_ERROR_MSG.to_string(),
                            error_notes: smallvec![],
                        },
                        preceding_trivia: prev.preceding_trivia,
                        span: prev.span,
                        ending_span: Some(span),
                    });
                    return;
                } else if Self::spans_on_same_line(&last.span, &span) {
                    // Second dot on same line but not adjacent - terminal error
                    // This is `. .` with spacing - won't become `...`
                    let prev = self.pending.pop().unwrap();
                    self.pending.push(PendingToken {
                        kind: GraphQLTokenKind::Error {
                            message: SPACED_DOT_DOT_ERROR_MSG.to_string(),
                            error_notes: smallvec![(
                                "These dots may have been intended to form a `...` spread \
                                 operator. Try removing the extra spacing between the dots."
                                    .to_string(),
                                Some(Self::make_source_span(&span))
                            )],
                        },
                        preceding_trivia: prev.preceding_trivia,
                        span: prev.span,
                        ending_span: Some(span),
                    });
                    return;
                }
                // Second dot on different line - leave previous as single dot
                // error and fall through to emit a new single dot error
            }
        }

        // First dot, or non-adjacent to previous dot on different line
        let kind = GraphQLTokenKind::Error {
            message: DOT_ERROR_MSG.to_string(),
            error_notes: smallvec![],
        };
        let token = self.make_pending_token(kind, span);
        self.pending.push(token);
    }

    /// Processes a `Literal` token (string, number, etc.).
    fn process_literal_token(&mut self, lit: Literal) {
        let span = lit.span();
        let lit_str = lit.to_string();

        // Check for raw strings - these are Rust-specific and not valid
        // GraphQL syntax
        if Self::is_raw_string(&lit_str) {
            self.process_raw_string_error(&lit_str, span);
            return;
        }

        // Try to parse as integer (store raw string for later parsing)
        if lit_str.parse::<i64>().is_ok() {
            let kind = GraphQLTokenKind::IntValue(lit_str);
            let token = self.make_pending_token(kind, span);
            self.pending.push(token);
            return;
        }

        // Try to parse as float (store raw string for later parsing)
        if lit_str.parse::<f64>().is_ok() {
            let kind = GraphQLTokenKind::FloatValue(lit_str);
            let token = self.make_pending_token(kind, span);
            self.pending.push(token);
            return;
        }

        // Regular string literal - store the raw source text as-is
        // (including quotes and escape sequences). The `cook_string_value()`
        // method on `GraphQLTokenKind` will handle escape sequence
        // processing later per the GraphQL spec.
        //
        // Per GraphQL spec §2.4.1 (String Value):
        // https://spec.graphql.org/September2025/#sec-String-Value
        //
        // String values store the raw source text. Escape sequences like
        // `\n`, `\t`, `\"`, etc. are processed when the string is "cooked"
        // (interpreted). We don't unescape here because:
        // 1. GraphQL and Rust have different escape sequence syntaxes
        // 2. The raw text is needed for accurate error reporting
        // 3. `cook_string_value()` handles GraphQL-specific escapes
        if lit_str.starts_with('"') && lit_str.ends_with('"') {
            let kind = GraphQLTokenKind::StringValue(lit_str);
            let token = self.make_pending_token(kind, span);
            self.pending.push(token);
            return;
        }

        // Fallback: treat as name (e.g., for character literals or other
        // unexpected literal types)
        let kind = GraphQLTokenKind::Name(lit_str);
        let token = self.make_pending_token(kind, span);
        self.pending.push(token);
    }

    /// Emits an error token for a Rust raw string with a helpful suggestion.
    ///
    /// Analyzes the raw string content and suggests an equivalent GraphQL
    /// string (either inline or block string) based on the content.
    fn process_raw_string_error(&mut self, lit_str: &str, span: Span) {
        // Extract content from raw string (handles r"..." and r#"..."# forms)
        let content = Self::extract_raw_string_content(lit_str);

        // Decide whether to suggest inline string or block string
        let suggestion = Self::suggest_graphql_string(&content);

        let kind = GraphQLTokenKind::Error {
            message: "Rust raw strings (`r\"...\"` or `r#\"...\"#`) are not valid \
                     GraphQL syntax"
                .to_string(),
            error_notes: smallvec![(
                format!("Consider using: {suggestion}"),
                Some(Self::make_source_span(&span))
            )],
        };
        let token = self.make_pending_token(kind, span);
        self.pending.push(token);
    }

    /// Extracts the content from a Rust raw string literal.
    ///
    /// Handles both `r"..."` and `r#"..."#` (with any number of `#`s) forms.
    fn extract_raw_string_content(lit_str: &str) -> String {
        // Count leading # signs after 'r'
        let after_r = &lit_str[1..];
        let hash_count = after_r.chars().take_while(|&c| c == '#').count();

        // Extract content between the opening and closing delimiters
        // For r"...", hash_count is 0, so we have: r" ... "
        // For r#"..."#, hash_count is 1, so we have: r#" ... "#
        let start = 1 + hash_count + 1; // 'r' + '#'s + '"'
        let end = lit_str.len() - hash_count - 1; // remove trailing '"' + '#'s

        if start < end {
            lit_str[start..end].to_string()
        } else {
            String::new()
        }
    }

    /// Suggests a GraphQL string representation for the given content.
    ///
    /// Returns a block string (`"""..."""`) if the content contains more than
    /// 4 newlines or more than 4 double quotes, otherwise returns an inline
    /// string with proper escaping.
    fn suggest_graphql_string(content: &str) -> String {
        // Count problematic characters
        let newline_count = content.chars().filter(|&c| c == '\n').count()
            + content.chars().filter(|&c| c == '\r').count();
        let quote_count = content.chars().filter(|&c| c == '"').count();

        // Use block string only if there are more than 4 newlines or quotes
        let needs_block_string = newline_count > 4 || quote_count > 4;

        if needs_block_string {
            // For block strings, we need to escape `"""` sequences within
            let escaped = content.replace("\"\"\"", "\\\"\"\"");
            format!("\"\"\"{escaped}\"\"\"")
        } else {
            // For inline strings, escape backslashes and double quotes
            let escaped = content.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{escaped}\"")
        }
    }

    /// Attempts to detect and combine GraphQL block strings (`"""..."""`).
    ///
    /// Rust tokenizes `"""content"""` as three separate string literals:
    /// 1. `""` (empty string)
    /// 2. `"content"` (the actual content)
    /// 3. `""` (empty string)
    ///
    /// This method checks if the first three pending tokens form this pattern
    /// AND are positionally adjacent (no whitespace between them). If so, it
    /// combines them into a single block string token.
    ///
    /// Returns `Some(token)` if a block string was detected and combined,
    /// `None` otherwise.
    fn try_combine_block_string(&mut self) -> Option<PendingToken> {
        if self.pending.len() < 3 {
            return None;
        }

        // Check if first three tokens are string values
        let (s1, span1, s2, span2, s3, span3) = match (
            &self.pending[0],
            &self.pending[1],
            &self.pending[2],
        ) {
            (
                PendingToken {
                    kind: GraphQLTokenKind::StringValue(s1),
                    span: span1,
                    ..
                },
                PendingToken {
                    kind: GraphQLTokenKind::StringValue(s2),
                    span: span2,
                    ..
                },
                PendingToken {
                    kind: GraphQLTokenKind::StringValue(s3),
                    span: span3,
                    ..
                },
            ) => (s1.clone(), *span1, s2.clone(), *span2, s3.clone(), *span3),
            _ => return None,
        };

        // Check if first and third are empty strings (`""`)
        if s1 != "\"\"" || s3 != "\"\"" {
            return None;
        }

        // Check if spans are adjacent (no whitespace between the quotes)
        // This ensures we only accept `"""content"""` and not `"" "content" ""`
        if !Self::spans_are_adjacent(&span1, &span2)
            || !Self::spans_are_adjacent(&span2, &span3)
        {
            return None;
        }

        // Extract the content from the middle string (remove surrounding quotes)
        let content = if s2.len() >= 2 {
            &s2[1..s2.len() - 1]
        } else {
            ""
        };

        // Build the block string in GraphQL format: `"""content"""`
        let block_string = format!("\"\"\"{content}\"\"\"");

        // Preserve trivia from the first token before removing
        let trivia = std::mem::take(&mut self.pending[0].preceding_trivia);

        // Remove the three tokens we're combining
        self.pending.drain(0..3);

        // Create a combined span (from start of first to end of third)
        Some(PendingToken {
            kind: GraphQLTokenKind::StringValue(block_string),
            preceding_trivia: trivia,
            span: span1,
            ending_span: Some(span3),
        })
    }

    /// Attempts to combine a pending `-` with a following number.
    ///
    /// In GraphQL, negative numbers like `-17` are valid IntValue/FloatValue
    /// tokens. However, Rust tokenizes `-17` as two separate tokens:
    /// `Punct('-')` and `Literal(17)`. When we see `-`, we store it as an
    /// error token with `PENDING_MINUS_ERROR_MSG`. This method checks if
    /// that error is followed by an IntValue or FloatValue and combines them.
    ///
    /// Returns `Some(token)` if a negative number was detected and combined,
    /// `None` otherwise.
    fn try_combine_negative_number(&mut self) -> Option<PendingToken> {
        if self.pending.len() < 2 {
            return None;
        }

        // Check if first token is the pending minus error
        let is_minus_error = matches!(
            &self.pending[0].kind,
            GraphQLTokenKind::Error { message, .. }
                if message == PENDING_MINUS_ERROR_MSG
        );
        if !is_minus_error {
            return None;
        }

        // Check if second token is IntValue or FloatValue
        match &self.pending[1].kind {
            GraphQLTokenKind::IntValue(value) => {
                let negative_value = format!("-{value}");
                let minus_span = self.pending[0].span;
                let number_span = self.pending[1].span;

                // Preserve trivia from the minus token
                let trivia = std::mem::take(&mut self.pending[0].preceding_trivia);

                // Remove the two tokens we're combining
                self.pending.drain(0..2);

                Some(PendingToken {
                    kind: GraphQLTokenKind::IntValue(negative_value),
                    preceding_trivia: trivia,
                    span: minus_span,
                    ending_span: Some(number_span),
                })
            }
            GraphQLTokenKind::FloatValue(value) => {
                let negative_value = format!("-{value}");
                let minus_span = self.pending[0].span;
                let number_span = self.pending[1].span;

                // Preserve trivia from the minus token
                let trivia = std::mem::take(&mut self.pending[0].preceding_trivia);

                // Remove the two tokens we're combining
                self.pending.drain(0..2);

                Some(PendingToken {
                    kind: GraphQLTokenKind::FloatValue(negative_value),
                    preceding_trivia: trivia,
                    span: minus_span,
                    ending_span: Some(number_span),
                })
            }
            _ => None,
        }
    }

    /// Creates an Eof token with any remaining trivia.
    fn make_eof_token(&mut self) -> GraphQLToken {
        let span = self
            .last_span
            .map(|s| Self::make_source_span(&s))
            .unwrap_or_else(|| {
                let pos = SourcePosition::new(0, 0, None, 0);
                GraphQLSourceSpan {
                    start_inclusive: pos.clone(),
                    end_exclusive: pos,
                }
            });

        let trivia = std::mem::take(&mut self.pending_trivia);
        GraphQLToken {
            kind: GraphQLTokenKind::Eof,
            preceding_trivia: trivia,
            span,
        }
    }
}

impl Iterator for RustMacroGraphQLTokenSource {
    type Item = GraphQLToken;

    fn next(&mut self) -> Option<Self::Item> {
        // If we've already emitted Eof, we're done
        if self.finished {
            return None;
        }

        // Process tokens until we have at least 3 pending tokens (needed for
        // block string detection) or until input is exhausted
        while self.pending.len() < 3 && self.tokens.peek().is_some() {
            if let Some(tree) = self.tokens.next() {
                self.process_token_tree(tree);
            }
        }

        // Try to detect and combine block strings
        if let Some(block_string) = self.try_combine_block_string() {
            self.last_span = block_string.ending_span.or(Some(block_string.span));
            return Some(block_string.into());
        }

        // Try to detect and combine negative numbers
        if let Some(negative_number) = self.try_combine_negative_number() {
            self.last_span = negative_number.ending_span.or(Some(negative_number.span));
            return Some(negative_number.into());
        }

        // Return next pending token if available
        if !self.pending.is_empty() {
            let pending = self.pending.remove(0);
            self.last_span = pending.ending_span.or(Some(pending.span));
            return Some(pending.into());
        }

        // No more tokens - emit Eof
        self.finished = true;
        Some(self.make_eof_token())
    }
}
