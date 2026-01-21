//! This module provides the core token types used by GraphQL lexers and the
//! parser.

mod graphql_token;
mod graphql_token_kind;
mod graphql_trivia_token;

pub use graphql_token::GraphQLToken;
pub use graphql_token::GraphQLTriviaTokenVec;
pub use graphql_token_kind::GraphQLTokenKind;
pub use graphql_trivia_token::GraphQLTriviaToken;

#[cfg(test)]
mod tests;
