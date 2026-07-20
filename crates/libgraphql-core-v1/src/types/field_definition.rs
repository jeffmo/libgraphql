use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::deprecation_state::DeprecationState;
use crate::types::parameter_definition::ParameterDefinition;
use crate::types::type_annotation::TypeAnnotation;
use indexmap::IndexMap;

/// A field definition on an
/// [`ObjectType`](crate::types::ObjectType) or
/// [`InterfaceType`](crate::types::InterfaceType).
///
/// This is a *schema-level* field definition — the shape and type
/// of a field as declared in the schema. A field *selected*
/// within an operation is a distinct concept, represented by the
/// operation layer's `FieldSelection` type.
///
/// See [Field Definitions](https://spec.graphql.org/September2025/#FieldsDefinition).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct FieldDefinition {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: FieldName,
    pub(crate) parameters: IndexMap<FieldName, ParameterDefinition>,
    pub(crate) parent_type_name: TypeName,
    pub(crate) span: Span,
    pub(crate) type_annotation: TypeAnnotation,
}

impl FieldDefinition {
    /// The deprecation status of this field, derived from the
    /// presence of a
    /// [`@deprecated`](https://spec.graphql.org/September2025/#sec--deprecated)
    /// directive annotation on the field definition.
    pub fn deprecation_state(&self) -> DeprecationState<'_> {
        DeprecationState::from_directives(&self.directives)
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn name(&self) -> &FieldName { &self.name }
    pub fn parameters(&self) -> &IndexMap<FieldName, ParameterDefinition> {
        &self.parameters
    }
    pub fn parent_type_name(&self) -> &TypeName {
        &self.parent_type_name
    }
    /// The name of the innermost type this field returns.
    /// Convenience for
    /// `self.type_annotation().innermost_type_name()`.
    pub fn return_type_name(&self) -> &TypeName {
        self.type_annotation.innermost_type_name()
    }
    pub fn span(&self) -> Span { self.span }
    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
