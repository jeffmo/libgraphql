use crate::DirectiveAnnotation;
use crate::loc;
use crate::types::DeprecationState;
use crate::types::EnumValue;
use std::collections::BTreeMap;

/// Represents a
/// [enum type](https://spec.graphql.org/October2021/#sec-Enums) defined within
/// some [`Schema`](crate::schema::Schema).
#[derive(Clone, Debug, PartialEq)]
pub struct EnumType {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) description: Option<String>,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) name: String,
    pub(super) values: BTreeMap<String, EnumValue>,
}

impl EnumType {
    /// The [`loc::SchemaDefLocation`] indicating where this [`EnumType`] was
    /// defined within the schema.
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    /// The [`DeprecationState`] of this [`EnumType`] as indicated by the
    /// presence of a `@deprecated` annotation.
    pub fn deprecation_state(&self) -> DeprecationState<'_> {
        (&self.directives).into()
    }

    /// The description of this [`EnumType`] as defined in the schema
    /// (e.g. in a """-string immediately before the type definition).
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`EnumType`].
    ///
    /// This list of [`DirectiveAnnotation`]s is guaranteed to be ordered the same
    /// as the order of annotations specified on the [`EnumType`] definition in
    /// the schema. Note that [`DirectiveAnnotation`]s added from a type extension
    /// will appear sequentially in the order they were applied on the type
    /// extension, but there is no guarantee about where in this list a given
    /// type extension's annotations are added.
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    /// The name of this [`EnumType`].
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// A map from ValueName -> [`EnumValue`] for all [`EnumValue`]s defined for
    /// this [`EnumType`].
    ///
    /// This returns a [`BTreeMap`] to guarantee that map entries retain the same
    /// ordering as the order of field definitions on the [`EnumType`] in the
    /// schema.
    pub fn values(&self) -> &BTreeMap<String, EnumValue> {
        &self.values
    }
}
