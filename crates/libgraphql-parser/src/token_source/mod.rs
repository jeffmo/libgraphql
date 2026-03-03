//! Token source trait and implementations.

mod graphql_token_source;
mod str_graphql_token_source_config;
mod str_to_graphql_token_source;

pub use graphql_token_source::GraphQLTokenSource;
pub use str_graphql_token_source_config::StrGraphQLTokenSourceConfig;
pub use str_to_graphql_token_source::StrGraphQLTokenSource;

#[cfg(test)]
mod tests;
