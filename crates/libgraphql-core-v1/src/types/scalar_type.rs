use crate::directive_annotation::DirectiveAnnotation;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::scalar_kind::ScalarKind;

/// A GraphQL [scalar type](https://spec.graphql.org/September2025/#sec-Scalars).
///
/// Both built-in scalars (`Boolean`, `Float`, `ID`, `Int`, `String`)
/// and custom scalars are represented by this struct. Use
/// [`kind()`](Self::kind) to distinguish them, and
/// [`is_builtin()`](Self::is_builtin) as a convenience check.
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ScalarType {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) kind: ScalarKind,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

impl ScalarType {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn is_builtin(&self) -> bool {
        !matches!(self.kind, ScalarKind::Custom)
    }
    pub fn kind(&self) -> ScalarKind { self.kind }
    pub fn name(&self) -> &TypeName { &self.name }
    pub fn span(&self) -> Span { self.span }
}
