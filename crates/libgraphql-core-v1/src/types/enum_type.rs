use crate::directive_annotation::DirectiveAnnotation;
use crate::names::EnumValueName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::enum_value::EnumValue;
use indexmap::IndexMap;

/// A GraphQL [enum type](https://spec.graphql.org/September2025/#sec-Enums).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct EnumType {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
    pub(crate) values: IndexMap<EnumValueName, EnumValue>,
}

impl EnumType {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn name(&self) -> &TypeName { &self.name }
    pub fn span(&self) -> Span { self.span }
    pub fn value(&self, name: &str) -> Option<&EnumValue> {
        self.values.get(name)
    }
    pub fn values(&self) -> &IndexMap<EnumValueName, EnumValue> {
        &self.values
    }
}
