use crate::loc;

/// Represents an
/// [input field](https://spec.graphql.org/October2021/#InputFieldsDefinition)
/// defined on an [crate::types::InputObjectType].
#[derive(Clone, Debug, PartialEq)]
pub struct InputField {
    pub def_location: loc::SchemaDefLocation,
    // TODO: There's more to input fields...
}

