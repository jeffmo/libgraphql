use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::span::Span;
use crate::value::Value;
use indexmap::IndexMap;

/// An applied directive annotation on a definition, field, or
/// argument (e.g. `@deprecated(reason: "Use newField")`).
///
/// This represents a *usage* of a directive — not its
/// *definition*. For the schema-level directive definition, see
/// [`DirectiveDefinition`](crate::types::DirectiveDefinition).
///
/// See
/// [Directives](https://spec.graphql.org/September2025/#sec-Language.Directives)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct DirectiveAnnotation {
    pub(crate) arguments: IndexMap<FieldName, Value>,
    pub(crate) name: DirectiveName,
    pub(crate) span: Span,
}

impl DirectiveAnnotation {
    pub fn arguments(&self) -> &IndexMap<FieldName, Value> {
        &self.arguments
    }

    #[inline]
    pub fn name(&self) -> &DirectiveName { &self.name }

    #[inline]
    pub fn span(&self) -> Span { self.span }
}
