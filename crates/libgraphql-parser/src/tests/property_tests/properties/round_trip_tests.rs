//! Property tests verifying round-trip fidelity of the parser.
//!
//! Round-trip testing ensures that:
//! 1. Source-slice round trip: `parse(src).to_source(Some(src))`
//!    reproduces the original source exactly (validates span tracking)
//! 2. Re-parse stability: the reconstructed source parses without
//!    errors (validates `AstNode::to_source` correctness)
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;

use crate::ast::AstNode;
use crate::tests::property_tests::generators::documents::arb_executable_document;
use crate::tests::property_tests::generators::documents::arb_schema_document;
use crate::tests::property_tests::proptest_config;
use crate::GraphQLParser;

proptest! {
    #![proptest_config(proptest_config())]

    /// Verifies source-slice round trip for schema documents:
    /// `parse(src).to_source(Some(src))` should reproduce the
    /// original source exactly.
    ///
    /// When `to_source` is given the original source text, it uses
    /// byte spans to slice out the original text — so the output
    /// should be character-for-character identical.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn schema_source_slice_round_trip(source in arb_schema_document(4)) {
        let result = GraphQLParser::new(&source).parse_schema_document();
        prop_assert!(
            !result.has_errors(),
            "Generated schema document should parse without errors.\n\
             Source:\n{}",
            source,
        );
        let doc = result.into_ast();
        let reconstructed = doc.to_source(Some(&source));
        prop_assert_eq!(
            &reconstructed,
            &source,
            "Source-slice round trip failed for schema document.",
        );
    }

    /// Verifies source-slice round trip for executable documents.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn executable_source_slice_round_trip(
        source in arb_executable_document(4)
    ) {
        let result = GraphQLParser::new(&source).parse_executable_document();
        prop_assert!(
            !result.has_errors(),
            "Generated executable document should parse without errors.\n\
             Source:\n{}",
            source,
        );
        let doc = result.into_ast();
        let reconstructed = doc.to_source(Some(&source));
        prop_assert_eq!(
            &reconstructed,
            &source,
            "Source-slice round trip failed for executable document.",
        );
    }

    /// Verifies re-parse stability for schema documents:
    /// `parse(src) -> to_source(Some(src)) -> re-parse -> no errors`.
    ///
    /// Even if `to_source` doesn't produce identical output (e.g., in
    /// synthetic mode without source), the reconstructed text should
    /// still be valid GraphQL.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn schema_reparse_stability(source in arb_schema_document(4)) {
        let result = GraphQLParser::new(&source).parse_schema_document();
        prop_assert!(
            !result.has_errors(),
            "Generated schema document should parse without errors.\n\
             Source:\n{}",
            source,
        );
        let doc = result.into_ast();
        let reconstructed = doc.to_source(Some(&source));
        let reparse_result = GraphQLParser::new(&reconstructed)
            .parse_schema_document();
        prop_assert!(
            !reparse_result.has_errors(),
            "Re-parse of reconstructed schema document failed.\n\
             Original:\n{}\n\n\
             Reconstructed:\n{}\n\n\
             Errors:\n{}",
            source,
            reconstructed,
            reparse_result.format_errors(),
        );
    }

    /// Verifies re-parse stability for executable documents.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn executable_reparse_stability(
        source in arb_executable_document(4)
    ) {
        let result = GraphQLParser::new(&source).parse_executable_document();
        prop_assert!(
            !result.has_errors(),
            "Generated executable document should parse without errors.\n\
             Source:\n{}",
            source,
        );
        let doc = result.into_ast();
        let reconstructed = doc.to_source(Some(&source));
        let reparse_result = GraphQLParser::new(&reconstructed)
            .parse_executable_document();
        prop_assert!(
            !reparse_result.has_errors(),
            "Re-parse of reconstructed executable document failed.\n\
             Original:\n{}\n\n\
             Reconstructed:\n{}\n\n\
             Errors:\n{}",
            source,
            reconstructed,
            reparse_result.format_errors(),
        );
    }

}
