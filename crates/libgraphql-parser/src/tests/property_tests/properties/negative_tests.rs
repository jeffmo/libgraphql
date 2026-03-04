//! Property tests verifying that mutated/invalid documents
//! produce parse errors.
//!
//! These tests complement the positive spec conformance tests by
//! ensuring the parser correctly rejects invalid input. Each
//! mutation strategy breaks valid documents in a specific way.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;

use crate::tests::property_tests::generators::documents::arb_executable_document;
use crate::tests::property_tests::generators::documents::arb_schema_document;
use crate::tests::property_tests::generators::mutations::arb_reserved_fragment_name;
use crate::tests::property_tests::proptest_config;
use crate::GraphQLParser;

proptest! {
    #![proptest_config(proptest_config())]

    /// Verifies that truncating a schema document mid-token (inside
    /// a curly-brace block) produces parse errors.
    ///
    /// We find the first `{` in the source and truncate 2 characters
    /// after it, ensuring we cut inside a type body. This is more
    /// reliable than random truncation because it guarantees we're
    /// in the middle of a syntactic construct.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn truncated_schema_documents_produce_errors(
        source in arb_schema_document(3)
    ) {
        // Find the first `{` and truncate shortly after it
        let brace_pos = source.find('{');
        prop_assume!(brace_pos.is_some());
        let cut_point = brace_pos.unwrap() + 2;
        prop_assume!(cut_point < source.len());
        let mut boundary = cut_point;
        while boundary < source.len() && !source.is_char_boundary(boundary) {
            boundary += 1;
        }
        prop_assume!(boundary < source.len());
        let truncated = &source[..boundary];

        let result = GraphQLParser::new(truncated).parse_schema_document();
        prop_assert!(
            result.has_errors(),
            "Truncated schema document should produce errors.\n\
             Original:\n{}\n\nTruncated:\n{}",
            source,
            truncated,
        );
    }

    /// Verifies that truncating an executable document mid-token
    /// (inside a selection set) produces parse errors.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn truncated_executable_documents_produce_errors(
        source in arb_executable_document(3)
    ) {
        let brace_pos = source.find('{');
        prop_assume!(brace_pos.is_some());
        let cut_point = brace_pos.unwrap() + 2;
        prop_assume!(cut_point < source.len());
        let mut boundary = cut_point;
        while boundary < source.len() && !source.is_char_boundary(boundary) {
            boundary += 1;
        }
        prop_assume!(boundary < source.len());
        let truncated = &source[..boundary];

        let result = GraphQLParser::new(truncated).parse_executable_document();
        prop_assert!(
            result.has_errors(),
            "Truncated executable document should produce errors.\n\
             Original:\n{}\n\nTruncated:\n{}",
            source,
            truncated,
        );
    }

    /// Verifies that replacing `{` with `[` in schema documents
    /// that contain braces produces parse errors.
    ///
    /// GraphQL uses `{` for type bodies and `[` for list types —
    /// swapping them should always break the document.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn brace_to_bracket_swap_produces_errors(
        source in arb_schema_document(3)
    ) {
        // Only test documents that actually contain braces
        prop_assume!(source.contains('{'));
        let mutated = source.replace('{', "[");
        let result = GraphQLParser::new(&mutated).parse_schema_document();
        prop_assert!(
            result.has_errors(),
            "Brace-to-bracket swap should produce errors.\n\
             Original:\n{}\n\nMutated:\n{}",
            source,
            mutated,
        );
    }

    /// Verifies that fragments using the reserved name `on`
    /// produce parse errors.
    ///
    /// The name `on` is reserved for type conditions and cannot
    /// be used as a fragment name.
    /// See [FragmentName](https://spec.graphql.org/September2025/#FragmentName).
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn reserved_fragment_names_produce_errors(
        source in arb_reserved_fragment_name()
    ) {
        let result = GraphQLParser::new(&source).parse_executable_document();
        prop_assert!(
            result.has_errors(),
            "Fragment with reserved name 'on' should produce errors.\n\
             Source:\n{}",
            source,
        );
    }
}
