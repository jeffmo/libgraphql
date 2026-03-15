use crate::GraphQLErrorNote;
use crate::GraphQLErrorNoteKind;
use crate::GraphQLErrorNotes;
use crate::GraphQLParseErrorKind;
use crate::SourceSpan;

/// A parse error with location information and contextual notes.
///
/// This structure provides comprehensive error information for both
/// human-readable CLI output and programmatic handling by tools.
#[derive(Debug, Clone)]
pub struct GraphQLParseError {
    /// Human-readable primary error message.
    ///
    /// This is the main error description shown to users.
    /// Examples: "Expected `:` after field name", "Unclosed `{`"
    message: String,

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

    /// Pre-resolved source span with line/column/byte-offset/file info.
    ///
    /// Eagerly resolved at construction time from the `SourceMap`, so
    /// all position information (including byte offsets) is always
    /// available without requiring a `SourceMap` at access time.
    ///
    /// The `SourcePosition::byte_offset()` values on
    /// `start_inclusive` / `end_exclusive` carry the original byte
    /// offsets from the `ByteSpan` that was resolved — these are the
    /// same synthetic offsets used for `SpanMap` lookup in the
    /// proc-macro path.
    source_span: SourceSpan,
}

impl GraphQLParseError {
    /// Creates a new parse error with no notes.
    pub fn new(
        message: impl Into<String>,
        kind: GraphQLParseErrorKind,
        source_span: SourceSpan,
    ) -> Self {
        Self {
            message: message.into(),
            kind,
            notes: GraphQLErrorNotes::new(),
            source_span,
        }
    }

    /// Creates a new parse error with notes.
    pub fn with_notes(
        message: impl Into<String>,
        kind: GraphQLParseErrorKind,
        notes: GraphQLErrorNotes,
        source_span: SourceSpan,
    ) -> Self {
        Self {
            message: message.into(),
            kind,
            notes,
            source_span,
        }
    }

    /// Creates a parse error from a lexer error token.
    ///
    /// When the parser encounters a `GraphQLTokenKind::Error` token,
    /// this method converts it to a `GraphQLParseError`, preserving
    /// the lexer's message and notes.
    pub fn from_lexer_error(
        message: impl Into<String>,
        lexer_notes: GraphQLErrorNotes,
        source_span: SourceSpan,
    ) -> Self {
        Self {
            message: message.into(),
            kind: GraphQLParseErrorKind::LexerError,
            notes: lexer_notes,
            source_span,
        }
    }

    /// Returns the human-readable error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the pre-resolved source span for this error.
    ///
    /// This span is resolved at construction time, so it is always
    /// available without a `SourceMap`. When the error was
    /// constructed without position info, this returns a
    /// zero-position span.
    ///
    /// The `SourcePosition::byte_offset()` values on
    /// `start_inclusive` / `end_exclusive` carry the original byte
    /// offsets that can be used for `SpanMap` lookup or
    /// `SourceMap::resolve_offset()` calls.
    pub fn source_span(&self) -> &SourceSpan {
        &self.source_span
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

    /// Adds a general note with a pre-resolved span (pointing to
    /// a related location).
    pub fn add_note_with_span(
        &mut self,
        message: impl Into<String>,
        span: SourceSpan,
    ) {
        self.notes.push(
            GraphQLErrorNote::general_with_span(message, span),
        );
    }

    /// Adds a help note without a span.
    pub fn add_help(&mut self, message: impl Into<String>) {
        self.notes.push(GraphQLErrorNote::help(message));
    }

    /// Adds a help note with a pre-resolved span.
    pub fn add_help_with_span(
        &mut self,
        message: impl Into<String>,
        span: SourceSpan,
    ) {
        self.notes.push(
            GraphQLErrorNote::help_with_span(message, span),
        );
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
    ///    |              ^^^^^^
    ///    |
    ///    = note: Field definitions require `:` between name and type
    ///    = help: Did you mean: `userName: String`?
    /// ```
    ///
    /// All position information (file path, line, column) comes
    /// from the pre-resolved `source_span` and note spans. The
    /// optional `source` parameter provides the original source
    /// text for snippet display — when `None`, the diagnostic
    /// omits source snippets but still shows the error header,
    /// location, and notes.
    pub fn format_detailed(
        &self,
        source: Option<&str>,
    ) -> String {
        let mut output = String::new();

        // Error header
        output.push_str("error: ");
        output.push_str(&self.message);
        output.push('\n');

        // Location line
        let file_name = self.source_span.file_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<input>".to_string());
        let line =
            self.source_span.start_inclusive.line() + 1;
        let column =
            self.source_span.start_inclusive.col_utf8() + 1;
        output.push_str(
            &format!("  --> {file_name}:{line}:{column}\n"),
        );

        // Source snippet
        if let Some(snippet) =
            self.format_source_snippet(source)
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
            output.push_str(
                &format!(
                    "   = {prefix}: {}\n",
                    note.message,
                ),
            );

            if let Some(note_span) = &note.span
                && let Some(snippet) =
                    Self::format_note_snippet(
                        source, note_span,
                    )
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
    ///
    /// This is equivalent to the `Display` impl. Prefer using
    /// `format!("{error}")` or `error.to_string()` directly.
    pub fn format_oneline(&self) -> String {
        self.to_string()
    }

    /// Formats the source snippet for the primary error span.
    fn format_source_snippet(
        &self,
        source: Option<&str>,
    ) -> Option<String> {
        let source = source?;
        let start_pos = &self.source_span.start_inclusive;
        let end_pos = &self.source_span.end_exclusive;

        let line_num = start_pos.line();
        let line_content = get_line(source, line_num)?;
        let display_line_num = line_num + 1;
        let line_num_width =
            display_line_num.to_string().len().max(2);

        let mut output = String::new();

        // Separator line
        output.push_str(
            &format!(
                "{:>width$} |\n",
                "",
                width = line_num_width,
            ),
        );

        // Source line
        output.push_str(&format!(
            "{display_line_num:>line_num_width$} | \
             {line_content}\n"
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

    /// Formats a source snippet for a note's pre-resolved span.
    fn format_note_snippet(
        source: Option<&str>,
        span: &SourceSpan,
    ) -> Option<String> {
        let source = source?;
        let start_pos = &span.start_inclusive;

        let line_num = start_pos.line();
        let line_content = get_line(source, line_num)?;
        let display_line_num = line_num + 1;
        let line_num_width =
            display_line_num.to_string().len().max(2);

        let mut output = String::new();

        // Source line with line number
        output.push_str(&format!(
            "     {display_line_num:>line_num_width$} | \
             {line_content}\n"
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

impl std::fmt::Display for GraphQLParseError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let file_name = self.source_span.file_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<input>".to_string());
        let line =
            self.source_span.start_inclusive.line() + 1;
        let col =
            self.source_span.start_inclusive.col_utf8() + 1;
        write!(
            f,
            "{file_name}:{line}:{col}: error: {}",
            self.message,
        )
    }
}

impl std::error::Error for GraphQLParseError {}

/// Extracts the content of the line at the given 0-based line
/// index from source text.
///
/// Recognizes `\n`, `\r\n`, and bare `\r` as line terminators per
/// the GraphQL spec (Section 2.2). The returned line content is
/// stripped of its trailing line terminator.
///
/// Returns `None` if `line_index` is out of bounds.
///
/// Note: `SourceMap::get_line()` provides similar functionality but
/// uses a pre-computed `line_starts` table for O(1) line-start
/// lookup. This version scans from the beginning each time (O(n)),
/// which is fine for error formatting but would be less suitable
/// for hot paths. Both must use the same line-terminator semantics.
fn get_line(source: &str, line_index: usize) -> Option<&str> {
    let bytes = source.as_bytes();
    let mut current_line = 0;
    let mut pos = 0;

    // Skip lines until we reach the target line index
    while current_line < line_index {
        match memchr::memchr2(b'\n', b'\r', &bytes[pos..]) {
            Some(offset) => {
                pos += offset;
                if bytes[pos] == b'\r'
                    && pos + 1 < bytes.len()
                    && bytes[pos + 1] == b'\n'
                {
                    pos += 2; // \r\n
                } else {
                    pos += 1; // \n or bare \r
                }
                current_line += 1;
            },
            None => return None, // line_index exceeds line count
        }
    }

    // Find the end of the target line
    let line_start = pos;
    let line_end = memchr::memchr2(b'\n', b'\r', &bytes[pos..])
        .map(|offset| pos + offset)
        .unwrap_or(bytes.len());

    Some(&source[line_start..line_end])
}
