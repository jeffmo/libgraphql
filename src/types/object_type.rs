use crate::DirectiveAnnotation;
use crate::loc;
use crate::schema::Schema;
use crate::types::Field;
use crate::types::InterfaceType;
use crate::types::ObjectOrInterfaceTypeTrait;
use crate::types::ObjectOrInterfaceTypeData;
use inherent::inherent;
use std::collections::BTreeMap;

/// Represents a
/// [object type](https://spec.graphql.org/October2021/#sec-Objects) defined
/// within some [`Schema`].
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectType(pub(super) ObjectOrInterfaceTypeData);

#[inherent]
impl ObjectOrInterfaceTypeTrait for ObjectType {
    /// The [`SchemaDefLocation`](loc::SchemaDefLocation) indicating where this
    /// [`ObjectType`] was defined within the schema.
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        self.0.def_location()
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`ObjectType`].
    ///
    /// This list is guaranteed to be ordered the same as the order of
    /// annotations specified on the object type definition in the schema. Note
    /// that annotations added from a type extension will appear sequentially in
    /// the order they were applied on the type extension, but there is no
    /// guarantee about where in this list a given type extension's annotations
    /// are added.
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        self.0.directives()
    }

    /// A map from FieldName -> [`Field`] for all fields defined on this
    /// [`ObjectType`] in the schema.
    ///
    /// This returns a [`BTreeMap`] to guarantee that map entries retain the
    /// same ordering as the order of fields defined on the object type in the
    /// schema. Note that fields added from type extensions will appear in the
    /// order they were specified on the type extension, but there is no
    /// guarantee about where in this list a given type extension's fields will
    /// be added.
    pub fn fields(&self) -> &BTreeMap<String, Field> {
        self.0.fields()
    }

    /// The list of [`InterfaceType`]s implemented by this [`ObjectType`].
    ///
    /// This list is guaranteed to be ordered the same as the order of
    /// interfaces specified on the object type definition in the schema. Note
    /// that interfaces added from a type extension will appear sequentially in
    /// the order they were applied on the type extension, but there is no
    /// guarantee about where in this list a given type extension's interfaces
    /// are added.
    pub fn interfaces<'schema>(&self, schema: &'schema Schema) -> Vec<&'schema InterfaceType> {
        self.0.interfaces(schema)
    }

    /// The names of all [`InterfaceType`]s implemented by this [`ObjectType`].
    ///
    /// This can be useful when the [`Schema`] object is unavailable or
    /// inconvenient to access but the type's name is all that's needed.
    pub fn interface_names(&self) -> Vec<&str> {
        self.0.interface_names()
    }

    /// The name of this [`ObjectType`].
    pub fn name(&self) -> &str {
        self.0.name()
    }
}
