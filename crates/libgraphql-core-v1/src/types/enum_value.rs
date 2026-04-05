use crate::directive_annotation::DirectiveAnnotation;
use crate::names::EnumValueName;
use crate::names::TypeName;
use crate::span::Span;

/// A single value within an [`EnumType`](crate::types::EnumType)
/// definition.
///
/// See
/// [Enum Values](https://spec.graphql.org/September2025/#EnumValuesDefinition).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct EnumValue {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: EnumValueName,
    pub(crate) parent_type_name: TypeName,
    pub(crate) span: Span,
}

impl EnumValue {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn name(&self) -> &EnumValueName { &self.name }
    /// The name of the [`EnumType`](crate::types::EnumType) that
    /// defines this value.
    pub fn parent_type_name(&self) -> &TypeName {
        &self.parent_type_name
    }
    pub fn span(&self) -> Span { self.span }
}
