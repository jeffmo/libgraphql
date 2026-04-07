mod schema_build_error;
pub(crate) mod schema_builder;
mod schema_def;
mod schema_errors;
mod type_validation_error;

pub use crate::schema::schema_def::Schema;
pub use crate::schema::schema_build_error::SchemaBuildError;
pub use crate::schema::schema_build_error::SchemaBuildErrorKind;
pub use crate::schema::schema_builder::SchemaBuilder;
pub use crate::schema::schema_errors::SchemaErrors;
pub use crate::schema::type_validation_error::TypeValidationError;
pub use crate::schema::type_validation_error::TypeValidationErrorKind;

#[cfg(test)]
mod tests;
