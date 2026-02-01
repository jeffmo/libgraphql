//! Converts `libgraphql_parser::GraphQLParseError`s into
//! `compile_error!` token streams using a [`SpanMap`] to recover
//! `proc_macro2::Span` locations.

use crate::span_map::SpanMap;
use libgraphql_parser::GraphQLErrorNoteKind;
use libgraphql_parser::GraphQLParseError;
use proc_macro2::Span;
use quote::quote_spanned;

/// Converts a slice of [`GraphQLParseError`]s into a combined
/// `proc_macro2::TokenStream` of `compile_error!` invocations.
///
/// Each error produces at least one `compile_error!` at the
/// primary error span. Notes that carry their own span produce
/// an additional `compile_error!` at that location so the user
/// sees multiple pointers in their editor/terminal.
pub(crate) fn convert_parse_errors_to_tokenstream(
    errors: &[GraphQLParseError],
    span_map: &SpanMap,
) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();

    for error in errors {
        // Look up primary span
        let primary_span = span_map
            .lookup(&error.span().start_inclusive)
            .unwrap_or_else(|| {
                // Unexpected: every error position should have
                // been recorded. Fall back gracefully.
                Span::call_site()
            });

        // Build the formatted message with inline notes
        let formatted_msg =
            format_parse_error_message(error);
        output.extend(quote_spanned! { primary_span =>
            compile_error!(#formatted_msg);
        });

        // Emit additional compile_error! at note spans
        for note in error.notes() {
            if let Some(note_source_span) = &note.span
                && let Some(note_span) = span_map
                    .lookup(
                        &note_source_span.start_inclusive,
                    ) {
                let note_msg =
                    format_parse_error_note(note);
                output.extend(
                    quote_spanned! { note_span =>
                        compile_error!(#note_msg);
                    },
                );
            }
        }
    }

    output
}

/// Formats a single error into a human-readable message string
/// that includes all notes inline.
fn format_parse_error_message(
    error: &GraphQLParseError,
) -> String {
    let mut msg = error.message().to_string();

    for note in error.notes() {
        let prefix = match note.kind {
            GraphQLErrorNoteKind::General => "note",
            GraphQLErrorNoteKind::Help => "help",
            GraphQLErrorNoteKind::Spec => "spec",
        };
        msg.push_str(&format!(
            "\n  = {prefix}: {}",
            note.message,
        ));
    }

    msg
}

/// Formats a single note into a message suitable for a
/// secondary `compile_error!`.
fn format_parse_error_note(
    note: &libgraphql_parser::GraphQLErrorNote,
) -> String {
    let prefix = match note.kind {
        GraphQLErrorNoteKind::General => "note",
        GraphQLErrorNoteKind::Help => "help",
        GraphQLErrorNoteKind::Spec => "spec",
    };
    format!("{prefix}: {}", note.message)
}
