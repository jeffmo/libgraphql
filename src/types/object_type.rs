use crate::loc;
use crate::Schema;
use crate::types::DirectiveAnnotation;
use crate::types::Field;
use crate::types::InterfaceType;
use crate::types::ObjectOrInterfaceType;
use crate::types::ObjectOrInterfaceTypeData;
use inherent::inherent;
use std::collections::BTreeMap;

/// Represents a GraphQL [object type](https://spec.graphql.org/October2021/#sec-Objects).
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectType(pub(super) ObjectOrInterfaceTypeData);

#[inherent]
impl ObjectOrInterfaceType for ObjectType {
    /// The [loc::FilePosition] indicating where this [ObjectType] was defined
    /// in the schema.
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        self.0.def_location()
    }

    /// The list of [DirectiveAnnotation]s applied to this [ObjectType].
    ///
    /// This list of [DirectiveAnnotation]s is guaranteed to be ordered the same
    /// as the order of annotations specified on the [ObjectType] definition in
    /// the schema. Note that [DirectiveAnnotation]s added from a type extension
    /// will appear sequentially in the order they were applied on the type
    /// extension, but there is no guarantee about where in this list a given
    /// type extension's annotations are added.
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        self.0.directives()
    }

    /// A map from FieldName -> [Field] for all [Field]s defined on this
    /// [ObjectType] in the schema.
    ///
    /// This returns a [BTreeMap] to guarantee that map entries retain the same
    /// ordering as the order of field definitions on the [ObjectType]
    /// definition in the schema. Note that [Field]s added from a type
    /// extensions will appear in the order they were specified on the type
    /// extension, but there is no guarantee about where in this list a given
    /// type extension's fields will be added.
    pub fn fields(&self) -> &BTreeMap<String, Field> {
        self.0.fields()
    }

    /// The list of [InterfaceType]s implemented by this [ObjectType].
    ///
    /// This list of [InterfaceType]s is guaranteed to be ordered the same as
    /// the order of interfaces specified on the [ObjectType] definition in the
    /// schema. Note that interfaces added from a type extension will appear
    /// sequentially in the order they were applied on the type extension, but
    /// there is no guarantee about where in this list a given type extension's
    /// interfaces are added.
    pub fn interfaces<'schema>(&self, schema: &'schema Schema) -> Vec<&'schema InterfaceType> {
        self.0.interfaces(schema)
    }

    // The name of this [ObjectType].
    pub fn name(&self) -> &str {
        self.0.name()
    }
}
