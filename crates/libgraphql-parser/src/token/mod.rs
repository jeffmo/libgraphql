//! This module provides the core token types used by GraphQL lexers and the
//! parser.

mod graphql_token;
mod graphql_token_source;
mod graphql_token_kind;
mod graphql_trivia_token;
mod str_graphql_token_source_config;
mod str_to_graphql_token_source;

pub use graphql_token::GraphQLToken;
pub use graphql_token::GraphQLTriviaTokenVec;
pub use graphql_token_kind::GraphQLTokenError;
pub use graphql_token_kind::GraphQLTokenKind;
pub use graphql_token_source::GraphQLTokenSource;
pub use graphql_trivia_token::GraphQLTriviaToken;
pub use str_graphql_token_source_config::StrGraphQLTokenSourceConfig;
pub use str_to_graphql_token_source::StrGraphQLTokenSource;

#[cfg(test)]
mod tests;
