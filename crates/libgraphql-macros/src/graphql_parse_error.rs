use proc_macro2::Span;
use quote::quote;

/// Represents a single parsing error with span information.
///
/// This error type captures precise location information for parse errors,
/// allowing the parser to emit compiler errors that point to the
/// exact location of the problem in the user's code.
#[derive(Debug, Clone)]
pub struct GraphQLParseError {
    /// Human-readable error message
    pub message: String,
    /// Spans highlighting the problematic tokens (can be multiple for complex errors)
    pub spans: Vec<Span>,
    /// Categorized error kind for programmatic error handling
    pub kind: GraphQLParseErrorKind,
}

impl GraphQLParseError {
    /// Creates a new parse error with a single span
    pub fn new(message: String, span: Span, kind: GraphQLParseErrorKind) -> Self {
        Self {
            message,
            spans: vec![span],
            kind,
        }
    }

    /// Creates a new parse error with multiple spans
    pub fn with_spans(message: String, spans: Vec<Span>, kind: GraphQLParseErrorKind) -> Self {
        Self {
            message,
            spans,
            kind,
        }
    }

    /// Converts this error into a `compile_error!` token stream
    pub fn into_compile_error(self) -> proc_macro2::TokenStream {
        let message = self.message;

        // If we have spans, create a compile_error at each span
        if self.spans.is_empty() {
            quote! {
                compile_error!(#message);
            }
        } else {
            // Create compile_error at the first span (primary error location)
            let primary_span = self.spans[0];
            let error = quote::quote_spanned! {primary_span=>
                compile_error!(#message);
            };

            // TODO: In the future, we could emit additional notes at secondary spans
            // For now, we just use the primary span
            error
        }
    }
}

/// Categorizes different kinds of parse errors for better error reporting
/// and potential error recovery strategies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphQLParseErrorKind {
    /// Expected one or more specific tokens but found something else
    UnexpectedToken {
        expected: Vec<String>,
        found: String,
    },
    /// Unexpected end of input while expecting more tokens
    UnexpectedEof {
        expected: Vec<String>,
    },
    /// General syntax error that doesn't fit other categories
    InvalidSyntax,
    /// A definition with a duplicate name was encountered
    DuplicateDefinition {
        name: String,
    },
    /// A directive was used in an invalid location
    InvalidDirectiveLocation,
    /// An invalid value was encountered (e.g., malformed number, string)
    InvalidValue {
        details: String,
    },
    /// Unclosed delimiter (brace, bracket, or paren)
    UnclosedDelimiter {
        delimiter: String,
        opening_span_available: bool,
    },
    /// Mismatched delimiter (e.g., opened with '[' but closed with '}')
    MismatchedDelimiter {
        expected: String,
        found: String,
    },
}

/// Collection of errors accumulated during parsing with error recovery.
///
/// This type allows the parser to continue parsing after encountering errors,
/// collecting all errors to present to the user at once rather than failing
/// on the first error.
#[derive(Debug, Clone)]
pub struct GraphQLParseErrors {
    pub errors: Vec<GraphQLParseError>,
}

impl GraphQLParseErrors {
    /// Creates a new empty error collection
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// Adds an error to the collection
    pub fn add(&mut self, error: GraphQLParseError) {
        self.errors.push(error);
    }

    /// Returns true if any errors have been collected
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns the number of errors collected
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Returns true if no errors have been collected
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Converts all errors into a single TokenStream containing compile_error! invocations
    pub fn into_compile_errors(self) -> proc_macro2::TokenStream {
        let mut output = proc_macro2::TokenStream::new();

        for error in self.errors {
            output.extend(error.into_compile_error());
        }

        output
    }
}

impl Default for GraphQLParseErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for GraphQLParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Display for GraphQLParseErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, error) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{error}")?;
        }
        Ok(())
    }
}
