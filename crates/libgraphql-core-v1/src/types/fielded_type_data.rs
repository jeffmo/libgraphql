use crate::directive_annotation::DirectiveAnnotation;
use crate::located::Located;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::field_definition::FieldDefinition;
use indexmap::IndexMap;

/// Shared data for [`ObjectType`](crate::types::ObjectType) and
/// [`InterfaceType`](crate::types::InterfaceType).
///
/// Not part of the public API. Both types wrap this struct and
/// delegate via [`HasFieldsAndInterfaces`](crate::types::HasFieldsAndInterfaces).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct FieldedTypeData {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) fields: IndexMap<FieldName, FieldDefinition>,
    pub(crate) interfaces: Vec<Located<TypeName>>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

impl FieldedTypeData {
    pub(crate) fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub(crate) fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub(crate) fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields.get(name)
    }
    pub(crate) fn fields(&self) -> &IndexMap<FieldName, FieldDefinition> {
        &self.fields
    }
    pub(crate) fn interfaces(&self) -> &[Located<TypeName>] {
        &self.interfaces
    }
    pub(crate) fn name(&self) -> &TypeName { &self.name }
    pub(crate) fn span(&self) -> Span { self.span }
}
