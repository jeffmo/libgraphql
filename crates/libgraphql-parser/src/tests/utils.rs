//! Various test utils.
//!
//! Written by Claude Code, reviewed by a human.

use crate::legacy_ast;
use crate::token::GraphQLToken;
use crate::token::GraphQLTokenKind;
use crate::GraphQLParser;
use crate::GraphQLSourceSpan;
use crate::SourcePosition;
use smallvec::smallvec;

/// Creates a mock token with the given kind and minimal span/trivia.
///
/// Uses `'static` lifetime since test tokens use owned strings.
pub fn mock_token(kind: GraphQLTokenKind<'static>) -> GraphQLToken<'static> {
    let pos = SourcePosition::new(0, 0, Some(0), 0);
    GraphQLToken {
        kind,
        preceding_trivia: smallvec![],
        span: GraphQLSourceSpan {
            start_inclusive: pos.clone(),
            end_exclusive: pos,
            file_path: None,
        },
    }
}

/// Creates a mock Name token with the given name.
pub fn mock_name_token(name: &str) -> GraphQLToken<'static> {
    mock_token(GraphQLTokenKind::name_owned(name.to_string()))
}

/// Creates a mock Eof token.
pub fn mock_eof_token() -> GraphQLToken<'static> {
    mock_token(GraphQLTokenKind::Eof)
}

/// A mock token source that produces tokens from a Vec.
///
/// Uses `'static` lifetime since mock tokens use owned strings.
pub struct MockTokenSource {
    tokens: std::vec::IntoIter<GraphQLToken<'static>>,
}

impl MockTokenSource {
    pub fn new(tokens: Vec<GraphQLToken<'static>>) -> Self {
        Self {
            tokens: tokens.into_iter(),
        }
    }
}

impl Iterator for MockTokenSource {
    type Item = GraphQLToken<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next()
    }
}

/// Helper to parse a schema document and return errors if any.
///
/// Parses into `ast::Document`, then converts to
/// `graphql_parser` types via the compat layer.
pub(super) fn parse_schema(
    source: &str,
) -> crate::ParseResult<legacy_ast::schema::Document> {
    use crate::compat_graphql_parser_v0_4
        ::to_graphql_parser_schema_ast;
    let parser = GraphQLParser::new(source);
    let result = parser.parse_schema_document();
    let parse_errors = result.errors.clone();
    match result.into_ast() {
        Some(doc) => {
            let compat = to_graphql_parser_schema_ast(&doc);
            let mut errors = parse_errors;
            errors.extend(compat.errors.clone());
            match compat.into_ast() {
                Some(gp_doc) if errors.is_empty() => {
                    crate::ParseResult::ok(gp_doc)
                },
                Some(gp_doc) => {
                    crate::ParseResult::recovered(
                        gp_doc, errors,
                    )
                },
                None => crate::ParseResult::err(errors),
            }
        },
        None => crate::ParseResult::err(parse_errors),
    }
}

/// Helper to parse an executable document and return errors if
/// any.
///
/// Parses into `ast::Document`, then converts to
/// `graphql_parser` types via the compat layer.
pub(super) fn parse_executable(
    source: &str,
) -> crate::ParseResult<legacy_ast::operation::Document> {
    use crate::compat_graphql_parser_v0_4
        ::to_graphql_parser_query_ast;
    let parser = GraphQLParser::new(source);
    let result = parser.parse_executable_document();
    let parse_errors = result.errors.clone();
    match result.into_ast() {
        Some(doc) => {
            let compat = to_graphql_parser_query_ast(&doc);
            let mut errors = parse_errors;
            errors.extend(compat.errors.clone());
            match compat.into_ast() {
                Some(gp_doc) if errors.is_empty() => {
                    crate::ParseResult::ok(gp_doc)
                },
                Some(gp_doc) => {
                    crate::ParseResult::recovered(
                        gp_doc, errors,
                    )
                },
                None => crate::ParseResult::err(errors),
            }
        },
        None => crate::ParseResult::err(parse_errors),
    }
}

/// Helper to parse a mixed document and return errors if any.
///
/// Parses into `ast::Document`, then converts schema and
/// executable definitions separately via the compat layer,
/// interleaving them in original source order.
pub(super) fn parse_mixed(
    source: &str,
) -> crate::ParseResult<legacy_ast::MixedDocument> {
    use crate::ast;
    use crate::compat_graphql_parser_v0_4
        ::to_graphql_parser_query_ast;
    use crate::compat_graphql_parser_v0_4
        ::to_graphql_parser_schema_ast;
    let parser = GraphQLParser::new(source);
    let result = parser.parse_mixed_document();
    let parse_errors = result.errors.clone();
    match result.into_ast() {
        Some(doc) => {
            let schema_compat =
                to_graphql_parser_schema_ast(&doc);
            let exec_compat =
                to_graphql_parser_query_ast(&doc);
            let mut errors = parse_errors;
            errors.extend(schema_compat.errors.clone());
            errors.extend(exec_compat.errors.clone());

            let schema_defs = schema_compat
                .into_ast()
                .map(|d| d.definitions)
                .unwrap_or_default();
            let exec_defs = exec_compat
                .into_ast()
                .map(|d| d.definitions)
                .unwrap_or_default();
            let mut schema_iter = schema_defs.into_iter();
            let mut exec_iter = exec_defs.into_iter();

            let mut mixed_defs = Vec::new();
            for def in &doc.definitions {
                match def {
                    ast::Definition::DirectiveDefinition(_)
                    | ast::Definition::SchemaDefinition(_)
                    | ast::Definition::TypeDefinition(_)
                    | ast::Definition::TypeExtension(_) => {
                        if let Some(sd) = schema_iter.next()
                        {
                            mixed_defs.push(
                                legacy_ast
                                    ::MixedDefinition
                                    ::Schema(sd),
                            );
                        }
                    },
                    // Schema extensions have no
                    // `graphql_parser` representation;
                    // the compat layer records an error.
                    ast::Definition::SchemaExtension(_) => {
                    },
                    ast::Definition::OperationDefinition(_)
                    | ast::Definition
                        ::FragmentDefinition(_) => {
                        if let Some(ed) = exec_iter.next() {
                            mixed_defs.push(
                                legacy_ast
                                    ::MixedDefinition
                                    ::Executable(ed),
                            );
                        }
                    },
                }
            }

            let mixed = legacy_ast::MixedDocument {
                definitions: mixed_defs,
            };
            if errors.is_empty() {
                crate::ParseResult::ok(mixed)
            } else {
                crate::ParseResult::recovered(
                    mixed, errors,
                )
            }
        },
        None => {
            crate::ParseResult::err(parse_errors)
        },
    }
}
