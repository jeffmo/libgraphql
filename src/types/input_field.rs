use crate::DirectiveAnnotation;
use crate::loc;
use crate::types::TypeAnnotation;

/// Represents an
/// [input field](https://spec.graphql.org/October2021/#InputFieldsDefinition)
/// defined on an [crate::types::InputObjectType].
#[derive(Clone, Debug, PartialEq)]
pub struct InputField {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) name: String,
    pub(super) type_annotation: TypeAnnotation,
}
impl InputField {
    /// The [`SchemaDefLocation`](loc::SchemaDefLocation) indicating where this
    /// [`InputField`] was defined within the schema.
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`InputField`].
    ///
    /// This list of [`DirectiveAnnotation`]s is guaranteed to be ordered the same
    /// as the order of annotations specified on the [`InputField`] definition in
    /// the schema. Note that [`DirectiveAnnotation`]s added from a type extension
    /// will appear sequentially in the order they were applied on the type
    /// extension, but there is no guarantee about where in this list a given
    /// type extension's annotations are added.
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    /// The name of this [`InputField`].
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// The [`TypeAnnotation`] specifying the schema-defined type of this
    /// [`InputField`].
    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
