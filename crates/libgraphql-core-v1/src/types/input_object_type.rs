use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::input_field::InputField;
use indexmap::IndexMap;

/// A GraphQL [input object type](https://spec.graphql.org/September2025/#sec-Input-Objects).
///
/// Input objects are the composite input type — they define a
/// set of named input fields, each with a type that must itself
/// be an input type (scalar, enum, or another input object).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct InputObjectType {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) fields: IndexMap<FieldName, InputField>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

impl InputObjectType {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn field(&self, name: &str) -> Option<&InputField> {
        self.fields.get(name)
    }
    pub fn fields(&self) -> &IndexMap<FieldName, InputField> {
        &self.fields
    }
    pub fn name(&self) -> &TypeName { &self.name }
    pub fn span(&self) -> Span { self.span }
}
