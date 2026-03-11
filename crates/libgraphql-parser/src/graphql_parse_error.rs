use crate::ByteSpan;
use crate::GraphQLErrorNote;
use crate::GraphQLErrorNoteKind;
use crate::GraphQLErrorNotes;
use crate::GraphQLParseErrorKind;
use crate::SourceMap;

/// A parse error with location information and contextual notes.
///
/// This structure provides comprehensive error information for both
/// human-readable CLI output and programmatic handling by tools.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{}", self.message)]
pub struct GraphQLParseError {
    /// Human-readable primary error message.
    ///
    /// This is the main error description shown to users.
    /// Examples: "Expected `:` after field name", "Unclosed `{`"
    message: String,

    /// The primary span where the error was detected.
    ///
    /// This location is highlighted as the main error site in CLI output.
    /// - For "unexpected token" errors: the unexpected token's span
    /// - For "expected X" errors: where X should have appeared
    /// - For "unclosed delimiter" errors: the position where closing was expected
    span: ByteSpan,

    /// Categorized error kind for programmatic handling.
    ///
    /// Enables tools to pattern-match on error types without parsing messages.
    kind: GraphQLParseErrorKind,

    /// Additional notes providing context, suggestions, and related locations.
    ///
    /// Each note has a kind (General, Help, Spec), message, and optional span:
    /// - With span: Points to a related location (e.g., "opening `{` here")
    /// - Without span: General advice not tied to a specific location
    ///
    /// Uses `GraphQLErrorNotes` for consistency with lexer errors.
    notes: GraphQLErrorNotes,
}

impl GraphQLParseError {
    /// Creates a new parse error with no notes.
    pub fn new(
        message: impl Into<String>,
        span: ByteSpan,
        kind: GraphQLParseErrorKind,
    ) -> Self {
        Self {
            message: message.into(),
            span,
            kind,
            notes: GraphQLErrorNotes::new(),
        }
    }

    /// Creates a new parse error with notes.
    pub fn with_notes(
        message: impl Into<String>,
        span: ByteSpan,
        kind: GraphQLParseErrorKind,
        notes: GraphQLErrorNotes,
    ) -> Self {
        Self {
            message: message.into(),
            span,
            kind,
            notes,
        }
    }

    /// Creates a parse error from a lexer error token.
    ///
    /// When the parser encounters a `GraphQLTokenKind::Error` token, this
    /// method converts it to a `GraphQLParseError`, preserving the lexer's
    /// message and notes.
    pub fn from_lexer_error(
        message: impl Into<String>,
        span: ByteSpan,
        lexer_notes: GraphQLErrorNotes,
    ) -> Self {
        Self {
            message: message.into(),
            span,
            kind: GraphQLParseErrorKind::LexerError,
            notes: lexer_notes,
        }
    }

    /// Returns the human-readable error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the primary span where the error was detected.
    pub fn span(&self) -> &ByteSpan {
        &self.span
    }

    /// Returns the categorized error kind.
    pub fn kind(&self) -> &GraphQLParseErrorKind {
        &self.kind
    }

    /// Returns the additional notes for this error.
    pub fn notes(&self) -> &GraphQLErrorNotes {
        &self.notes
    }

    /// Adds a general note without a span.
    pub fn add_note(&mut self, message: impl Into<String>) {
        self.notes.push(GraphQLErrorNote::general(message));
    }

    /// Adds a general note with a span (pointing to a related location).
    pub fn add_note_with_span(&mut self, message: impl Into<String>, span: ByteSpan) {
        self.notes
            .push(GraphQLErrorNote::general_with_span(message, span));
    }

    /// Adds a help note without a span.
    pub fn add_help(&mut self, message: impl Into<String>) {
        self.notes.push(GraphQLErrorNote::help(message));
    }

    /// Adds a help note with a span.
    pub fn add_help_with_span(&mut self, message: impl Into<String>, span: ByteSpan) {
        self.notes
            .push(GraphQLErrorNote::help_with_span(message, span));
    }

    /// Adds a spec reference note.
    pub fn add_spec(&mut self, url: impl Into<String>) {
        self.notes.push(GraphQLErrorNote::spec(url));
    }

    /// Formats this error as a diagnostic string for CLI output.
    ///
    /// Produces output like:
    /// ```text
    /// error: Expected `:` after field name
    ///   --> schema.graphql:5:12
    ///    |
    ///  5 |     userName String
    ///    |              ^^^^^^ expected `:`
    ///    |
    ///    = note: Field definitions require `:` between name and type
    ///    = help: Did you mean: `userName: String`?
    /// ```
    ///
    /// The `SourceMap` resolves byte offsets to line/column
    /// positions and provides the source text (if available)
    /// for snippet extraction.
    pub fn format_detailed(
        &self,
        source_map: &SourceMap<'_>,
    ) -> String {
        let mut output = String::new();

        // Error header
        output.push_str("error: ");
        output.push_str(&self.message);
        output.push('\n');

        // Location line
        let file_name = source_map
            .file_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<input>".to_string());
        let (line, column) = source_map
            .resolve_offset(self.span.start)
            .map(|pos| (pos.line() + 1, pos.col_utf8() + 1))
            .unwrap_or((1, 1));
        output.push_str(&format!("  --> {file_name}:{line}:{column}\n"));

        // Source snippet
        if let Some(snippet) =
            self.format_source_snippet(source_map)
        {
            output.push_str(&snippet);
        }

        // Notes
        for note in &self.notes {
            let prefix = match note.kind {
                GraphQLErrorNoteKind::General => "note",
                GraphQLErrorNoteKind::Help => "help",
                GraphQLErrorNoteKind::Spec => "spec",
            };
            output.push_str(&format!("   = {prefix}: {}\n", note.message));

            if let Some(note_span) = &note.span
                && let Some(snippet) =
                    Self::format_note_snippet(source_map, note_span)
            {
                output.push_str(&snippet);
            }
        }

        output
    }

    /// Formats this error as a single-line summary.
    ///
    /// Produces output like:
    /// ```text
    /// schema.graphql:5:12: error: Expected `:` after field name
    /// ```
    pub fn format_oneline(
        &self,
        source_map: &SourceMap<'_>,
    ) -> String {
        let file_name = source_map
            .file_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<input>".to_string());
        let (line, column) = source_map
            .resolve_offset(self.span.start)
            .map(|pos| (pos.line() + 1, pos.col_utf8() + 1))
            .unwrap_or((1, 1));

        format!("{file_name}:{line}:{column}: error: {}", self.message)
    }

    /// Formats the source snippet for the primary error span.
    fn format_source_snippet(
        &self,
        source_map: &SourceMap<'_>,
    ) -> Option<String> {
        let start_pos = source_map.resolve_offset(self.span.start)?;
        let end_pos = source_map.resolve_offset(self.span.end)?;

        let line_num = start_pos.line();
        let line_content = source_map.get_line(line_num)?;
        let display_line_num = line_num + 1;
        let line_num_width = display_line_num.to_string().len().max(2);

        let mut output = String::new();

        // Separator line
        output.push_str(&format!("{:>width$} |\n", "", width = line_num_width));

        // Source line
        output.push_str(&format!(
            "{display_line_num:>line_num_width$} | {line_content}\n"
        ));

        // Underline (caret line)
        let col_start = start_pos.col_utf8();
        let col_end = end_pos.col_utf8();
        let underline_len = if col_end > col_start {
            col_end - col_start
        } else {
            1
        };

        output.push_str(&format!(
            "{:>width$} | {:>padding$}{}\n",
            "",
            "",
            "^".repeat(underline_len),
            width = line_num_width,
            padding = col_start
        ));

        Some(output)
    }

    /// Formats a source snippet for a note's span.
    fn format_note_snippet(
        source_map: &SourceMap<'_>,
        span: &ByteSpan,
    ) -> Option<String> {
        let start_pos = source_map.resolve_offset(span.start)?;

        let line_num = start_pos.line();
        let line_content = source_map.get_line(line_num)?;
        let display_line_num = line_num + 1;
        let line_num_width = display_line_num.to_string().len().max(2);

        let mut output = String::new();

        // Source line with line number
        output.push_str(&format!(
            "     {display_line_num:>line_num_width$} | {line_content}\n"
        ));

        // Underline
        let col_start = start_pos.col_utf8();
        output.push_str(&format!(
            "     {:>width$} | {:>padding$}-\n",
            "",
            "",
            width = line_num_width,
            padding = col_start
        ));

        Some(output)
    }
}
