use crate::DirectiveAnnotation;
use crate::loc;
use crate::schema::Schema;
use crate::types::DeprecationState;
use crate::types::Field;
use crate::types::ObjectOrInterfaceTypeTrait;
use crate::types::ObjectOrInterfaceTypeData;
use indexmap::IndexMap;
use inherent::inherent;

/// Represents a
/// [interface type](https://spec.graphql.org/October2021/#sec-Interfaces)
/// defined within some [`Schema`].
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct InterfaceType(pub(super) ObjectOrInterfaceTypeData);

#[inherent]
impl ObjectOrInterfaceTypeTrait for InterfaceType {
    /// The [`loc::FilePosition`] indicating where this [`InterfaceType`] was
    /// defined in the schema.
    pub fn def_location(&self) -> &loc::SourceLocation {
        self.0.def_location()
    }

    /// The [`DeprecationState`] of this [`InterfaceType`] as indicated by the
    /// presence of a `@deprecated` annotation.
    pub fn deprecation_state(&self) -> DeprecationState<'_> {
        self.0.deprecation_state()
    }

    /// The description of this [`InterfaceType`] as defined in the schema
    /// (e.g. in a """-string immediately before the type definition).
    pub fn description(&self) -> Option<&str> {
        self.0.description()
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`InterfaceType`].
    ///
    /// This list of [`DirectiveAnnotation`]s is guaranteed to be ordered the same
    /// as the order of annotations specified on the [`InterfaceType`] definition
    /// in the schema. Note that [`DirectiveAnnotation`]s added from a type
    /// extension will appear sequentially in the order they were applied on the
    /// type extension, but there is no guarantee about where in this list a
    /// given type extension's annotations are added.
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        self.0.directives()
    }

    /// A map from FieldName -> [`Field`] for all fields defined on this
    /// [`InterfaceType`] in the schema.
    ///
    /// This returns an [`IndexMap`] to guarantee that map entries retain the same
    /// ordering as the order of fields defined on the [`InterfaceType`] in the
    /// schema. Note that [`Field`]s added from type extensions will appear in the
    /// order they were specified on the type extension, but there is no
    /// guarantee about where in this list a given type extension's fields will
    /// be added.
    pub fn fields(&self) -> &IndexMap<String, Field> {
        self.0.fields()
    }

    /// The list of [`InterfaceType`]s implemented by this `InterfaceType`.
    ///
    /// This list of [`InterfaceType`]s is guaranteed to be ordered the same as
    /// the order of interfaces specified on the [`InterfaceType`] definition in
    /// the schema. Note that interfaces added from a type extension will appear
    /// sequentially in the order they were applied on the type extension, but
    /// there is no guarantee about where in this list a given type extension's
    /// interfaces are added.
    pub fn interfaces<'schema>(&self, schema: &'schema Schema) -> Vec<&'schema InterfaceType> {
        self.0.interfaces(schema)
    }

    /// The names of all [`InterfaceType`]s implemented by this [`InterfaceType`].
    ///
    /// This can be useful when the [`Schema`] object is unavailable or
    /// inconvenient to access but the type's name is all that is needed.
    pub fn interface_names(&self) -> Vec<&str> {
        self.0.interface_names()
    }

    // The name of this [`InterfaceType`].
    pub fn name(&self) -> &str {
        self.0.name()
    }
}
