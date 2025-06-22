/// Things related to GraphQL syntax trees. Currently this is mostly re-exports
/// of/wrappers around [`graphql_parser`](graphql_parser).
pub mod ast;
mod directive_annotation;
mod file_reader;
/// Things related to file- and definition-locations (e.g. [loc::FilePosition],
/// [loc::SchemaDefLocation], etc).
pub mod loc;
mod named_ref;
/// Things related to
/// [GraphQL operations](https://spec.graphql.org/October2021/#sec-Language.Operations)
/// (e.g. [`Query`](operation::Query),
/// [`QueryBuilder`](operation::QueryBuilder),
/// [`Mutation`](operation::Mutation), etc...).
pub mod operation;
/// Things related to
/// [GraphQL schemas](https://spec.graphql.org/October2021/#sec-Schema)
/// (e.g. [`Schema`](schema::Schema), [`SchemaBuilder`](schema::SchemaBuilder),
/// etc...)
pub mod schema;
/// Things related to
/// [GraphQL types](https://spec.graphql.org/October2021/#sec-Types) which have
/// been defined within some [`Schema`](schema::Schema).
pub mod types;
mod value;

pub use directive_annotation::DirectiveAnnotation;
pub use named_ref::NamedRef;
pub use value::Value;
