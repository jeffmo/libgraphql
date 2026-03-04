//! Property tests verifying that generated valid GraphQL documents
//! parse without errors.
//!
//! These tests exercise the parser's happy path: every document
//! produced by our grammar-guided generators should parse
//! successfully, confirming that the parser accepts the full
//! breadth of valid GraphQL syntax.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;

use crate::tests::property_tests::generators::documents::arb_executable_document;
use crate::tests::property_tests::generators::documents::arb_mixed_document;
use crate::tests::property_tests::generators::documents::arb_schema_document;
use crate::tests::property_tests::proptest_config;
use crate::GraphQLParser;
use crate::GraphQLParserConfig;

proptest! {
    #![proptest_config(proptest_config())]

    /// Verifies that all generated schema documents parse without errors.
    ///
    /// This is the most fundamental spec conformance property: if a
    /// document is structurally valid GraphQL (as produced by our
    /// grammar-guided generators), the parser must accept it.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn schema_documents_parse_without_errors(
        source in arb_schema_document(4)
    ) {
        let result = GraphQLParser::new(&source).parse_schema_document();
        prop_assert!(
            !result.has_errors(),
            "Schema document should parse without errors.\n\
             Source:\n{}\n\nErrors:\n{}",
            source,
            result.format_errors(Some(&source)),
        );
    }

    /// Verifies that all generated executable documents parse without
    /// errors.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn executable_documents_parse_without_errors(
        source in arb_executable_document(4)
    ) {
        let result = GraphQLParser::new(&source).parse_executable_document();
        prop_assert!(
            !result.has_errors(),
            "Executable document should parse without errors.\n\
             Source:\n{}\n\nErrors:\n{}",
            source,
            result.format_errors(Some(&source)),
        );
    }

    /// Verifies that all generated mixed documents parse without errors.
    ///
    /// Mixed documents can contain both type-system and executable
    /// definitions.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn mixed_documents_parse_without_errors(
        source in arb_mixed_document(4)
    ) {
        let result = GraphQLParser::new(&source).parse_mixed_document();
        prop_assert!(
            !result.has_errors(),
            "Mixed document should parse without errors.\n\
             Source:\n{}\n\nErrors:\n{}",
            source,
            result.format_errors(Some(&source)),
        );
    }

    /// Verifies that schema documents also parse in lean mode
    /// (without syntax retention).
    ///
    /// Lean mode (`retain_syntax: false`) skips populating the
    /// `*Syntax` structs for performance. The parser should still
    /// accept all valid documents.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn schema_documents_parse_in_lean_mode(
        source in arb_schema_document(4)
    ) {
        let result = GraphQLParser::with_config(
            &source,
            GraphQLParserConfig::lean(),
        ).parse_schema_document();
        prop_assert!(
            !result.has_errors(),
            "Schema document should parse in lean mode.\n\
             Source:\n{}\n\nErrors:\n{}",
            source,
            result.format_errors(Some(&source)),
        );
    }

    /// Verifies that executable documents also parse in lean mode.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn executable_documents_parse_in_lean_mode(
        source in arb_executable_document(4)
    ) {
        let result = GraphQLParser::with_config(
            &source,
            GraphQLParserConfig::lean(),
        ).parse_executable_document();
        prop_assert!(
            !result.has_errors(),
            "Executable document should parse in lean mode.\n\
             Source:\n{}\n\nErrors:\n{}",
            source,
            result.format_errors(Some(&source)),
        );
    }
}
