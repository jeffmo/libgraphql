use crate::directive_annotation::DirectiveAnnotation;
use crate::located::Located;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::field_definition::FieldDefinition;
use crate::types::fielded_type_data::FieldedTypeData;
use crate::types::has_fields_and_interfaces::HasFieldsAndInterfaces;
use indexmap::IndexMap;

/// A GraphQL [interface type](https://spec.graphql.org/September2025/#sec-Interfaces).
///
/// Interface types define a set of fields that implementing types
/// must provide. An interface can itself implement other interfaces,
/// forming an interface hierarchy.
///
/// # Shared behavior
///
/// Interface types share their field and interface structure with
/// [`ObjectType`](crate::types::ObjectType) via the
/// [`HasFieldsAndInterfaces`] trait.
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct InterfaceType(pub(crate) FieldedTypeData);

impl HasFieldsAndInterfaces for InterfaceType {
    fn description(&self) -> Option<&str> { self.0.description() }
    fn directives(&self) -> &[DirectiveAnnotation] { self.0.directives() }
    fn field(&self, name: &str) -> Option<&FieldDefinition> { self.0.field(name) }
    fn fields(&self) -> &IndexMap<FieldName, FieldDefinition> { self.0.fields() }
    fn interfaces(&self) -> &[Located<TypeName>] { self.0.interfaces() }
    fn name(&self) -> &TypeName { self.0.name() }
    fn span(&self) -> Span { self.0.span() }
}
