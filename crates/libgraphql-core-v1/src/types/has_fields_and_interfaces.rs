use crate::directive_annotation::DirectiveAnnotation;
use crate::located::Located;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::field_definition::FieldDefinition;
use indexmap::IndexMap;

/// Shared behavior for types that define fields and implement
/// interfaces — [`ObjectType`](crate::types::ObjectType) and
/// [`InterfaceType`](crate::types::InterfaceType).
///
/// The GraphQL spec treats these as
/// "[composite output types](https://spec.graphql.org/September2025/#sec-Objects)"
/// with overlapping rules: both define fields, both can implement
/// interfaces, and both are validated by the same interface
/// implementation contract
/// ([IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation())).
///
/// This trait enables the validator and downstream consumers to
/// operate generically over both types without duplication.
pub trait HasFieldsAndInterfaces {
    fn description(&self) -> Option<&str>;
    fn directives(&self) -> &[DirectiveAnnotation];
    fn field(&self, name: &str) -> Option<&FieldDefinition>;
    fn fields(&self) -> &IndexMap<FieldName, FieldDefinition>;
    fn interfaces(&self) -> &[Located<TypeName>];
    fn name(&self) -> &TypeName;
    fn span(&self) -> Span;
}
