use crate::names::TypeName;
use crate::span::Span;
use crate::types::GraphQLType;

/// Trait for converting a type builder into a finalized
/// [`GraphQLType`](crate::types::GraphQLType).
///
/// Implemented by all type builders. Used by
/// [`SchemaBuilder::absorb_type()`](crate::schema::SchemaBuilder::absorb_type)
/// to accept any builder type.
pub trait IntoGraphQLType {
    /// Returns the type name declared by this builder.
    fn type_name(&self) -> &TypeName;

    /// Returns the source span of the type definition.
    fn type_span(&self) -> Span;

    /// Converts the builder into a finalized [`GraphQLType`].
    fn into_graphql_type(self) -> GraphQLType;
}
