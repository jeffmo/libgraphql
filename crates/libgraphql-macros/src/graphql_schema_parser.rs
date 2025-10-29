use crate::rust_to_graphql_token_adapter::GraphQLToken;
use crate::rust_to_graphql_token_adapter::RustToGraphQLTokenAdapter;
use libgraphql_core::ast;
use proc_macro2::Span;
use thiserror::Error;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to parse GraphQL schema at {span:#?}: {message}")]
    GraphQLParseError {
        message: String,
        span: Span,
    },
}

pub struct GraphQLSchemaParser {
    adapter: RustToGraphQLTokenAdapter,
}

impl GraphQLSchemaParser {
    pub fn new(adapter: RustToGraphQLTokenAdapter) -> Self {
        Self { adapter }
    }

    pub fn parse_document(self) -> Result<ast::schema::Document> {
        // Convert tokens back to GraphQL SDL string
        let mut sdl = String::new();
        let mut first_span: Option<Span> = None;
        let mut last_span: Option<Span> = None;

        for (token, span) in self.adapter {
            if first_span.is_none() {
                first_span = Some(span);
            }
            last_span = Some(span);

            match token {
                GraphQLToken::Punctuator(ref p) => {
                    // Add spacing for readability
                    match p.as_str() {
                        "{" | "[" | "(" => {
                            sdl.push_str(p);
                            sdl.push(' ');
                        }
                        "}" | "]" | ")" => {
                            sdl.push(' ');
                            sdl.push_str(p);
                            sdl.push(' ');
                        }
                        ":" | "=" => {
                            sdl.push_str(p);
                            sdl.push(' ');
                        }
                        "!" => {
                            sdl.push_str(p);
                            sdl.push(' ');
                        }
                        "|" | "&" => {
                            sdl.push(' ');
                            sdl.push_str(p);
                            sdl.push(' ');
                        }
                        "@" => {
                            sdl.push_str(p);
                        }
                        _ => {
                            sdl.push_str(p);
                            sdl.push(' ');
                        }
                    }
                }
                GraphQLToken::Name(ref name) => {
                    sdl.push_str(name);
                    sdl.push(' ');
                }
                GraphQLToken::IntValue(val) => {
                    sdl.push_str(&val.to_string());
                    sdl.push(' ');
                }
                GraphQLToken::FloatValue(val) => {
                    sdl.push_str(&val.to_string());
                    sdl.push(' ');
                }
                GraphQLToken::StringValue(ref s) => {
                    sdl.push('"');
                    sdl.push_str(s);
                    sdl.push('"');
                    sdl.push(' ');
                }
            }
        }

        // Parse the SDL string using libgraphql_core's parser
        ast::schema::parse(&sdl)
            .map_err(|err| {
                // Create a span that encompasses the entire macro invocation content
                let error_span = match (first_span, last_span) {
                    (Some(first), Some(last)) => first.join(last).unwrap_or(first),
                    (Some(span), None) | (None, Some(span)) => span,
                    (None, None) => Span::call_site(),
                };

                ParseError::GraphQLParseError {
                    message: err.to_string(),
                    span: error_span,
                }
            })
    }
}
