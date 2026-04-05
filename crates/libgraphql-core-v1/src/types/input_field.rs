use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::type_annotation::TypeAnnotation;
use crate::value::Value;

/// A field on an
/// [`InputObjectType`](crate::types::InputObjectType).
///
/// Input fields differ from output
/// [`FieldDefinition`](crate::types::FieldDefinition)s: they
/// can have default values but cannot have parameters or
/// selection sets.
///
/// See [Input Object Fields](https://spec.graphql.org/September2025/#sec-Input-Objects).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct InputField {
    pub(crate) default_value: Option<Value>,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: FieldName,
    pub(crate) parent_type_name: TypeName,
    pub(crate) span: Span,
    pub(crate) type_annotation: TypeAnnotation,
}

impl InputField {
    pub fn default_value(&self) -> Option<&Value> {
        self.default_value.as_ref()
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn name(&self) -> &FieldName { &self.name }
    pub fn parent_type_name(&self) -> &TypeName {
        &self.parent_type_name
    }
    pub fn span(&self) -> Span { self.span }
    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
