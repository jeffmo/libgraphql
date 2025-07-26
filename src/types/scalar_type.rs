use crate::DirectiveAnnotation;
use crate::loc;
use crate::types::DeprecationState;

/// Represents a
/// [scalar type](https://spec.graphql.org/October2021/#sec-Scalars) defined
/// within some [`Schema`](crate::schema::Schema).
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarType {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) description: Option<String>,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) name: String,
}

impl ScalarType {
    /// The [loc::SchemaDefLocation] indicating where this [ScalarType] was
    /// defined within the schema.
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    /// The [`DeprecationState`] of this [`ScalarType`] as indicated by the
    /// presence of a `@deprecated` annotation.
    pub fn deprecation_state(&self) -> DeprecationState<'_> {
        (&self.directives).into()
    }

    /// The description of this [`ScalarType`] as defined in the schema
    /// (e.g. in a """-string immediately before the type definition).
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// The list of [DirectiveAnnotation]s applied to this [ScalarType].
    ///
    /// This list of [DirectiveAnnotation]s is guaranteed to be ordered the same
    /// as the order of annotations specified on the [ScalarType] definition in
    /// the schema. Note that [DirectiveAnnotation]s added from a type extension
    /// will appear sequentially in the order they were applied on the type
    /// extension, but there is no guarantee about where in this list a given
    /// type extension's annotations are added.
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    // The name of this [ScalarType].
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
