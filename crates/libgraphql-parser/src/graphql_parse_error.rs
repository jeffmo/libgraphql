use crate::GraphQLErrorNote;
use crate::GraphQLErrorNoteKind;
use crate::GraphQLErrorNotes;
use crate::GraphQLParseErrorKind;
use crate::GraphQLSourceSpan;

/// A parse error with location information and contextual notes.
///
/// This structure provides comprehensive error information for both
/// human-readable CLI output and programmatic handling by tools.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{}", self.format_oneline())]
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
    span: GraphQLSourceSpan,

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
        span: GraphQLSourceSpan,
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
        span: GraphQLSourceSpan,
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
        span: GraphQLSourceSpan,
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
    pub fn span(&self) -> &GraphQLSourceSpan {
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
    pub fn add_note_with_span(&mut self, message: impl Into<String>, span: GraphQLSourceSpan) {
        self.notes
            .push(GraphQLErrorNote::general_with_span(message, span));
    }

    /// Adds a help note without a span.
    pub fn add_help(&mut self, message: impl Into<String>) {
        self.notes.push(GraphQLErrorNote::help(message));
    }

    /// Adds a help note with a span.
    pub fn add_help_with_span(&mut self, message: impl Into<String>, span: GraphQLSourceSpan) {
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
    /// # Arguments
    /// - `source`: Optional source text for snippet extraction. If `None`,
    ///   snippets are omitted but line/column info is still shown.
    pub fn format_detailed(&self, source: Option<&str>) -> String {
        let mut output = String::new();

        // Error header
        output.push_str("error: ");
        output.push_str(&self.message);
        output.push('\n');

        // Location line
        let file_name = self
            .span
            .file_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<input>".to_string());
        let line = self.span.start_inclusive.line() + 1;
        let column = self.span.start_inclusive.col_utf8() + 1;
        output.push_str(&format!("  --> {file_name}:{line}:{column}\n"));

        // Source snippet (if source is provided)
        if let Some(src) = source
            && let Some(snippet) = self.format_source_snippet(src)
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

            // If the note has a span and we have source, show that location too
            if let (Some(note_span), Some(src)) = (&note.span, source)
                && let Some(snippet) = self.format_note_snippet(src, note_span)
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
    pub fn format_oneline(&self) -> String {
        let file_name = self
            .span
            .file_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<input>".to_string());
        let line = self.span.start_inclusive.line() + 1;
        let column = self.span.start_inclusive.col_utf8() + 1;

        format!("{file_name}:{line}:{column}: error: {}", self.message)
    }

    /// Formats the source snippet for the primary error span.
    fn format_source_snippet(&self, source: &str) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();
        let line_num = self.span.start_inclusive.line();

        // Line numbers are 0-indexed internally
        if line_num >= lines.len() {
            return None;
        }

        let line_content = lines[line_num];
        let display_line_num = line_num + 1; // Display as 1-indexed for humans
        let line_num_width = display_line_num.to_string().len().max(2);

        let mut output = String::new();

        // Separator line
        output.push_str(&format!("{:>width$} |\n", "", width = line_num_width));

        // Source line
        output.push_str(&format!(
            "{display_line_num:>line_num_width$} | {line_content}\n"
        ));

        // Underline (caret line)
        let col_start = self.span.start_inclusive.col_utf8();
        let col_end = self.span.end_exclusive.col_utf8();
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
    fn format_note_snippet(&self, source: &str, span: &GraphQLSourceSpan) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();
        let line_num = span.start_inclusive.line();

        // Line numbers are 0-indexed internally
        if line_num >= lines.len() {
            return None;
        }

        let line_content = lines[line_num];
        let display_line_num = line_num + 1; // Display as 1-indexed for humans
        let line_num_width = display_line_num.to_string().len().max(2);

        let mut output = String::new();

        // Source line with line number
        output.push_str(&format!(
            "     {display_line_num:>line_num_width$} | {line_content}\n"
        ));

        // Underline
        let col_start = span.start_inclusive.col_utf8();
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
