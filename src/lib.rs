/// Things related to GraphQL syntax trees. Currently this is mostly re-exports
/// of/wrappers around [graphql_parser].
pub mod ast;
mod directive_annotation;
mod file_reader;
/// Things related to file- and definition-locations (e.g. [loc::FilePosition],
/// [loc::SchemaDefLocation], etc).
pub mod loc;
mod named_ref;
/// Things related to
/// [GraphQL operations](https://spec.graphql.org/October2021/#sec-Language.Operations)
/// (e.g. [operation::Query], [operation::QueryBuilder], etc...).
pub mod operation;
mod schema;
mod schema_builder;
/// Things related to
/// [GraphQL types](https://spec.graphql.org/October2021/#sec-Types) that have
/// been defined within some [Schema].
pub mod types;
mod value;

pub use directive_annotation::DirectiveAnnotation;
pub use named_ref::NamedRef;
pub use schema::Schema;
pub use schema_builder::SchemaBuilder;
pub use schema_builder::SchemaBuildError;
pub use value::Value;

#[cfg(test)]
pub(crate) use schema_builder::GraphQLOperationType;
#[cfg(test)]
pub(crate) use schema_builder::NamedTypeFilePosition;
#[cfg(test)]
mod tests;
