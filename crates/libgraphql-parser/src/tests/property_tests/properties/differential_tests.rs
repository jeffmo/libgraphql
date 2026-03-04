//! Property tests comparing our parser against `graphql_parser` v0.4
//! as an oracle.
//!
//! Differential testing compares the accept/reject decisions of two
//! parsers on the same input. When both accept, we compare the
//! resulting ASTs via the compatibility layer.
//!
//! Known divergences (due to spec version differences between
//! `graphql_parser` v0.4 and our parser) are filtered out.
//!
//! Written by Claude Code, reviewed by a human.

use proptest::prelude::*;

use crate::parser_compat::graphql_parser_v0_4::to_graphql_parser_query_ast;
use crate::parser_compat::graphql_parser_v0_4::to_graphql_parser_schema_ast;
use crate::tests::property_tests::generators::documents::arb_executable_document;
use crate::tests::property_tests::generators::documents::arb_schema_document;
use crate::tests::property_tests::proptest_config;
use crate::GraphQLParser;

/// Strips the contents of string literals from GraphQL source,
/// preserving only the delimiters. This prevents keyword detection
/// from false-matching on content inside string literals.
///
/// Handles both block strings (`"""..."""`) and regular strings
/// (`"..."`), including escape sequences.
fn strip_string_contents(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let b = source.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i < n {
        if i + 2 < n && b[i] == b'"' && b[i + 1] == b'"' && b[i + 2] == b'"' {
            // Block string: emit delimiters, skip body
            out.push_str("\"\"\"\"\"\"");
            i += 3;
            while i < n {
                if b[i] == b'\\' && i + 3 < n
                    && b[i + 1] == b'"' && b[i + 2] == b'"' && b[i + 3] == b'"'
                {
                    i += 4; // skip \""" escape
                } else if i + 2 < n
                    && b[i] == b'"' && b[i + 1] == b'"' && b[i + 2] == b'"'
                {
                    i += 3;
                    break;
                } else {
                    i += 1;
                }
            }
        } else if b[i] == b'"' {
            // Regular string: emit delimiters, skip body
            out.push_str("\"\"");
            i += 1;
            while i < n {
                if b[i] == b'\\' && i + 1 < n {
                    i += 2; // skip escape sequence
                } else if b[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
        } else {
            out.push(b[i] as char);
            i += 1;
        }
    }
    out
}

/// Checks if a source string uses features not supported by
/// `graphql_parser` v0.4 (which implements an older GraphQL spec).
///
/// Known gaps:
/// - Block string descriptions on more constructs
/// - `repeatable` keyword on directives
/// - Interface implementing interfaces
/// - `extend` with only `implements`
///
/// String literal contents are stripped before checking to avoid
/// false positives from keywords appearing inside string values.
fn uses_unsupported_v04_features(source: &str) -> bool {
    let stripped = strip_string_contents(source);
    // graphql_parser v0.4 doesn't handle `repeatable` directives
    stripped.contains("repeatable")
        // Interface implementing interfaces was added in a later spec
        || (stripped.contains("interface") && stripped.contains("implements"))
}

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
        // Skip sources with features unsupported by v0.4
        prop_assume!(!uses_unsupported_v04_features(&source));

        let our_result = GraphQLParser::new(&source).parse_schema_document();
        let oracle_result =
            graphql_parser::schema::parse_schema::<String>(&source);

        match (our_result.has_errors(), oracle_result.is_err()) {
            // Both accept: compare ASTs
            (false, false) => {
                let our_doc = our_result.into_valid_ast().unwrap();
                let our_converted = to_graphql_parser_schema_ast(&our_doc);
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
                    our_result.format_errors(Some(&source)),
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
        prop_assume!(!uses_unsupported_v04_features(&source));

        let our_result = GraphQLParser::new(&source)
            .parse_executable_document();
        let oracle_result =
            graphql_parser::parse_query::<String>(&source);

        match (our_result.has_errors(), oracle_result.is_err()) {
            (false, false) => {
                let our_doc = our_result.into_valid_ast().unwrap();
                let our_converted = to_graphql_parser_query_ast(&our_doc);
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
                    our_result.format_errors(Some(&source)),
                );
            },
            (false, true) => {},
            (true, true) => {},
        }
    }
}
