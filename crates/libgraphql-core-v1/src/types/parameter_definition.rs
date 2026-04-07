use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::span::Span;
use crate::types::type_annotation::TypeAnnotation;
use crate::value::Value;

/// A parameter definition on a
/// [`FieldDefinition`](crate::types::FieldDefinition) or
/// [`DirectiveDefinition`](crate::types::DirectiveDefinition).
///
/// Referred to as an "argument definition" in the GraphQL spec
/// ([`InputValueDefinition`](https://spec.graphql.org/September2025/#InputValueDefinition)
/// in the grammar).
///
/// See [Field Arguments](https://spec.graphql.org/September2025/#sec-Field-Arguments).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ParameterDefinition {
    pub(crate) default_value: Option<Value>,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: FieldName,
    pub(crate) span: Span,
    pub(crate) type_annotation: TypeAnnotation,
}

impl ParameterDefinition {
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
    pub fn span(&self) -> Span { self.span }
    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
