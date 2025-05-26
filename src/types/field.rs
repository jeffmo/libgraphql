use crate::loc;
use crate::types::TypeAnnotation;
use crate::types::Parameter;
use std::collections::BTreeMap;

/// Represents a [field](https://spec.graphql.org/October2021/#FieldDefinition)
/// defined on an [`ObjectType`](crate::types::ObjectType) or
/// [`InterfaceType`](crate::types::InterfaceType).
///
/// *(Note that fields defined on
/// [`InputObjectType`](crate::types::InputObjectType)s are represented by
/// [`InputField`](crate::types::InputField).)*
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) params: BTreeMap<String, Parameter>,
    pub(super) type_annotation: TypeAnnotation,
}

impl Field {
    // TODO: Encode this into a commonly-used trait (to ensure it's consistent
    //       across all types)
    /// The [`SchemaDefLocation`](loc::SchemaDefLocation) indicating where this
    /// [`Field`] was defined within the schema.
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    /// A map from ParameterName -> [`Parameter`] for all parameters defined on
    /// this [`Field`].
    ///
    /// This returns a [`BTreeMap`] to guarantee that map entries retain the same
    /// ordering as the order of parameters defined on the [`Field`] in the
    /// schema. Note that parameterss added from type extensions will appear in the
    /// order they were specified on the type extension, but there is no
    /// guarantee about where in this list a given type extension's fields will
    /// be added.
    pub fn parameters(&self) -> &BTreeMap<String, Parameter> {
        &self.params
    }

    /// The [`TypeAnnotation`] specifying the schema-defined type of this [`Field`].
    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
