use crate::directive_annotation::DirectiveAnnotation;
use crate::located::Located;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::field_definition::FieldDefinition;
use crate::types::fielded_type_data::FieldedTypeData;
use crate::types::has_fields_and_interfaces::HasFieldsAndInterfaces;
use indexmap::IndexMap;
use inherent::inherent;

/// A GraphQL [object type](https://spec.graphql.org/September2025/#sec-Objects).
///
/// Object types are the primary composite output type in GraphQL.
/// They define a set of named fields, each of which yields a value
/// of a specific type. Object types may implement one or more
/// interfaces, committing to provide the fields those interfaces
/// specify.
///
/// # Shared behavior
///
/// Object types share their field and interface structure with
/// [`InterfaceType`](crate::types::InterfaceType) via the
/// [`HasFieldsAndInterfaces`] trait.
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct ObjectType(pub(crate) FieldedTypeData);

#[inherent]
impl HasFieldsAndInterfaces for ObjectType {
    pub fn description(&self) -> Option<&str> { self.0.description() }
    pub fn directives(&self) -> &[DirectiveAnnotation] { self.0.directives() }
    pub fn field(&self, name: &str) -> Option<&FieldDefinition> { self.0.field(name) }
    pub fn fields(&self) -> &IndexMap<FieldName, FieldDefinition> { self.0.fields() }
    pub fn interfaces(&self) -> &[Located<TypeName>] { self.0.interfaces() }
    pub fn name(&self) -> &TypeName { self.0.name() }
    pub fn span(&self) -> Span { self.0.span() }
}
