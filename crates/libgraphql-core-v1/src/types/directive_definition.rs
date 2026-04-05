use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::span::Span;
use crate::types::directive_definition_kind::DirectiveDefinitionKind;
use crate::types::directive_location_kind::DirectiveLocationKind;
use crate::types::parameter_definition::ParameterDefinition;
use indexmap::IndexMap;

/// A directive definition in a GraphQL schema.
///
/// All directives — both built-in (`@skip`, `@include`,
/// `@deprecated`, `@specifiedBy`, `@oneOf`) and custom — are represented
/// by this single struct. Built-ins are distinguished by
/// [`kind()`](Self::kind) returning a non-`Custom` variant.
///
/// See [Type System Directives](https://spec.graphql.org/September2025/#sec-Type-System.Directives).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct DirectiveDefinition {
    pub(crate) description: Option<String>,
    pub(crate) is_repeatable: bool,
    pub(crate) kind: DirectiveDefinitionKind,
    pub(crate) locations: Vec<DirectiveLocationKind>,
    pub(crate) name: DirectiveName,
    pub(crate) parameters: IndexMap<FieldName, ParameterDefinition>,
    pub(crate) span: Span,
}

impl DirectiveDefinition {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn is_builtin(&self) -> bool { self.kind.is_builtin() }
    pub fn is_repeatable(&self) -> bool { self.is_repeatable }
    pub fn kind(&self) -> DirectiveDefinitionKind { self.kind }
    pub fn locations(&self) -> &[DirectiveLocationKind] {
        &self.locations
    }
    pub fn name(&self) -> &DirectiveName { &self.name }
    pub fn parameters(&self) -> &IndexMap<FieldName, ParameterDefinition> {
        &self.parameters
    }
    pub fn span(&self) -> Span { self.span }
}
