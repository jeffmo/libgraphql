use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::types::TypeAnnotation;

/// Represents an
/// [input field](https://spec.graphql.org/October2021/#InputFieldsDefinition)
/// defined on an [crate::types::InputObjectType].
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct InputField {
    pub(super) def_location: loc::SourceLocation,
    pub(super) description: Option<String>,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) name: String,
    pub(super) parent_type: NamedGraphQLTypeRef,
    pub(super) type_annotation: TypeAnnotation,
}
impl InputField {
    /// The [`SourceLocation`](loc::SourceLocation) indicating where this
    /// [`InputField`] was defined within the schema.
    pub fn def_location(&self) -> &loc::SourceLocation {
        &self.def_location
    }

    /// The description of this [`InputField`] as defined in the schema
    /// (e.g. in a `"""`-string immediately before the input field definition).
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
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

    pub fn parent_type<'schema>(
        &self,
        schema: &'schema Schema,
    ) -> &'schema GraphQLType {
        self.parent_type
            .deref(schema)
            .expect("type is present in schema")
    }

    /// The [`TypeAnnotation`] specifying the schema-defined type of this
    /// [`InputField`].
    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
