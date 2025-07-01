use crate::DirectiveAnnotation;
use crate::loc;
use crate::types::InputField;
use std::collections::BTreeMap;

/// Represents an
/// [input object type](https://spec.graphql.org/October2021/#sec-Input-Objects)
/// defined within some [`Schema`](crate::schema::Schema).
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectType {
    pub(crate) def_location: loc::SchemaDefLocation,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) fields: BTreeMap<String, InputField>,
    pub(crate) name: String,
}
impl InputObjectType {
    /// The [`SchemaDefLocation`](loc::SchemaDefLocation) indicating where this
    /// [`InputObjectType`] was defined within the schema.
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`InputObjectType`].
    ///
    /// This list is guaranteed to be ordered the same as the order of
    /// annotations specified on the object type definition in the schema. Note
    /// that annotations added from a type extension will appear sequentially in
    /// the order they were applied on the type extension, but there is no
    /// guarantee about where in this list a given type extension's annotations
    /// are added.
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    /// A map from FieldName -> [`InputField`] for all fields defined on this
    /// [`InputObjectType`] in the schema.
    ///
    /// This returns a [`BTreeMap`] to guarantee that map entries retain the
    /// same ordering as the order of fields defined on the object type in the
    /// schema. Note that fields added from type extensions will appear in the
    /// order they were specified on the type extension, but there is no
    /// guarantee about where in this list a given type extension's fields will
    /// be added.
    pub fn fields(&self) -> &BTreeMap<String, InputField> {
        &self.fields
    }

    /// The name of this [`InputObjectType`].
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
