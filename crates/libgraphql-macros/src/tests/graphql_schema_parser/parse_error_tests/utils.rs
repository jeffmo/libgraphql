use crate::rust_macro_graphql_token_source::RustMacroGraphQLTokenSource;
use libgraphql_parser::GraphQLParseError;
use libgraphql_parser::GraphQLParser;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

/// Helper to parse schema and expect errors.
///
/// Returns the `ParseResult` so callers can inspect both the
/// error list and (when error recovery succeeds) the partial AST.
///
/// Note: If the schema contains syntax that makes it invalid as
/// a Rust TokenStream (e.g., unclosed delimiters, unterminated
/// strings), this will return a single synthetic error since our
/// parser never gets invoked.
pub fn parse_expecting_error(
    schema: &str,
) -> Vec<GraphQLParseError> {
    let token_stream =
        match proc_macro2::TokenStream::from_str(schema) {
            Ok(ts) => ts,
            Err(_) => {
                // If TokenStream parsing fails, return
                // synthetic error
                return vec![
                    GraphQLParseError::new(
                        "Invalid Rust TokenStream",
                        libgraphql_parser::GraphQLSourceSpan::new(
                            libgraphql_parser::SourcePosition::new(
                                0, 0, None, 0,
                            ),
                            libgraphql_parser::SourcePosition::new(
                                0, 0, None, 0,
                            ),
                        ),
                        libgraphql_parser::GraphQLParseErrorKind::InvalidSyntax,
                    ),
                ];
            },
        };

    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let token_source =
        RustMacroGraphQLTokenSource::new(token_stream, span_map);
    let parser = GraphQLParser::new(token_source);
    let result = parser.parse_schema_document();
    assert!(
        result.has_errors(),
        "Expected parse to fail, but it succeeded",
    );
    result.errors
}

