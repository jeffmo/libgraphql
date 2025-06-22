mod schema;
pub(crate) mod schema_builder;

pub use schema::Schema;
pub use schema_builder::SchemaBuilder;
pub use schema_builder::SchemaBuildError;

#[cfg(test)]
mod tests;
