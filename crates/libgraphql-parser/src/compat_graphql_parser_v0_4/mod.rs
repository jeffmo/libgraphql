//! Compatibility layer for converting between the
//! libgraphql AST (`crate::ast`) and `graphql_parser`
//! v0.4 types.
//!
//! See [Section 9.2 of the AST design plan](
//! ../../custom-ast-plan.md) for the full conversion
//! specification.

mod from_query;
mod from_schema;
mod helpers;
mod to_query;
mod to_schema;

pub use from_query::from_graphql_parser_query_ast;
pub use from_query::from_graphql_parser_query_ast_with_source;
pub use from_schema::from_graphql_parser_schema_ast;
pub use from_schema::from_graphql_parser_schema_ast_with_source;
pub use to_query::to_graphql_parser_query_ast;
pub use to_schema::to_graphql_parser_schema_ast;

#[cfg(test)]
pub(crate) use helpers::description_to_gp;
#[cfg(test)]
pub(crate) use helpers::directive_to_gp;
#[cfg(test)]
pub(crate) use helpers::enum_value_def_to_gp;
#[cfg(test)]
pub(crate) use helpers::field_def_to_gp;
#[cfg(test)]
pub(crate) use helpers::input_value_def_to_gp;
#[cfg(test)]
pub(crate) use helpers::type_annotation_to_gp;
#[cfg(test)]
pub(crate) use helpers::value_to_gp;

#[cfg(test)]
mod tests;
