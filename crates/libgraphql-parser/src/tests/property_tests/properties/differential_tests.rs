//! Property tests comparing our parser against `graphql_parser` v0.4
//! as an oracle.
//!
//! Differential testing compares the accept/reject decisions of two
//! parsers on the same input. When both accept, we compare the
//! resulting ASTs via the compatibility layer.
//!
//! Where `graphql_parser` v0.4.1 lacks support for newer spec
//! features (e.g., `extend schema`, directives on variable
//! definitions), it will reject the input while our parser accepts.
//! These cases fall into the `(false, true)` match arm and are
//! silently skipped — no pre-filtering is required.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;

use crate::parser_compat::graphql_parser_v0_4::to_graphql_parser_query_ast;
use crate::parser_compat::graphql_parser_v0_4::to_graphql_parser_schema_ast;
use crate::tests::property_tests::generators::documents::arb_executable_document;
use crate::tests::property_tests::generators::documents::arb_schema_document;
use crate::tests::property_tests::proptest_config;
use crate::GraphQLParser;

proptest! {
    #![proptest_config(proptest_config())]

    /// Differential test for schema documents: compares accept/reject
    /// decisions and AST structure against `graphql_parser` v0.4.
    ///
    /// - Both accept: compare ASTs via compat layer
    /// - We reject, oracle accepts: possible parser bug (fail)
    /// - We accept, oracle rejects: likely spec version difference (skip)
    /// - Both reject: fine
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn schema_differential_vs_graphql_parser_v04(
        source in arb_schema_document(3)
    ) {
        let our_result = GraphQLParser::new(&source).parse_schema_document();
        let oracle_result =
            graphql_parser::schema::parse_schema::<String>(&source);

        match (our_result.has_errors(), oracle_result.is_err()) {
            // Both accept: compare ASTs
            (false, false) => {
                let our_doc = our_result.into_valid_ast().unwrap();
                let sm = crate::SourceMap::new_with_source(
                    &source, None,
                );
                let our_converted = to_graphql_parser_schema_ast(&our_doc, &sm);
                let oracle_doc = oracle_result.unwrap().into_static();

                let our_debug = format!("{:#?}", our_converted.into_ast());
                let oracle_debug = format!("{oracle_doc:#?}");

                // Compare debug representations for structural equality.
                // We use debug format comparison because it's simpler
                // than implementing deep structural equality across
                // different AST types.
                prop_assert_eq!(
                    our_debug,
                    oracle_debug,
                    "AST mismatch between our parser and graphql_parser \
                     v0.4.\nSource:\n{}",
                    source,
                );
            },
            // We reject but oracle accepts: possible bug in our parser
            (true, false) => {
                prop_assert!(
                    false,
                    "Our parser rejected a document that graphql_parser \
                     v0.4 accepted.\n\
                     Source:\n{}\n\n\
                     Our errors:\n{}",
                    source,
                    our_result.format_errors(),
                );
            },
            // We accept but oracle rejects: likely spec version
            // difference (not a failure).
            (false, true) => {
                // This is expected for newer spec features.
                // Log but don't fail.
            },
            // Both reject: fine
            (true, true) => {},
        }
    }

    /// Differential test for executable documents against
    /// `graphql_parser` v0.4.
    ///
    /// Written by Claude Code, reviewed by a human.
    #[test]
    fn executable_differential_vs_graphql_parser_v04(
        source in arb_executable_document(3)
    ) {
        let our_result = GraphQLParser::new(&source)
            .parse_executable_document();
        let oracle_result =
            graphql_parser::parse_query::<String>(&source);

        match (our_result.has_errors(), oracle_result.is_err()) {
            (false, false) => {
                let our_doc = our_result.into_valid_ast().unwrap();
                let sm = crate::SourceMap::new_with_source(
                    &source, None,
                );
                let our_converted = to_graphql_parser_query_ast(&our_doc, &sm);
                let oracle_doc = oracle_result.unwrap().into_static();

                let our_debug =
                    format!("{:#?}", our_converted.into_ast());
                let oracle_debug = format!("{oracle_doc:#?}");

                prop_assert_eq!(
                    our_debug,
                    oracle_debug,
                    "AST mismatch between our parser and graphql_parser \
                     v0.4.\nSource:\n{}",
                    source,
                );
            },
            (true, false) => {
                prop_assert!(
                    false,
                    "Our parser rejected a document that graphql_parser \
                     v0.4 accepted.\n\
                     Source:\n{}\n\n\
                     Our errors:\n{}",
                    source,
                    our_result.format_errors(),
                );
            },
            (false, true) => {},
            (true, true) => {},
        }
    }
}
