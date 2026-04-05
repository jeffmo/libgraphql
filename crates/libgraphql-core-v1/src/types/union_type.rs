use crate::directive_annotation::DirectiveAnnotation;
use crate::located::Located;
use crate::names::TypeName;
use crate::span::Span;

/// A GraphQL [union type](https://spec.graphql.org/September2025/#sec-Unions).
///
/// Unions represent a value that could be one of several object
/// types. Unlike interfaces, unions do not define shared fields
/// — the member types are opaque until resolved via a type
/// condition.
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct UnionType {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) members: Vec<Located<TypeName>>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

impl UnionType {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    /// The union's member types, each carrying the span of its
    /// occurrence in the schema source.
    pub fn members(&self) -> &[Located<TypeName>] {
        &self.members
    }
    pub fn name(&self) -> &TypeName { &self.name }
    pub fn span(&self) -> Span { self.span }
}
