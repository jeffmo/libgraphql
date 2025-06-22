use crate::DirectiveAnnotation;
use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::schema::Schema;
use crate::types::EnumType;
use crate::types::NamedGraphQLTypeRef;

/// Represents an
/// [enum value](https://spec.graphql.org/October2021/#sec-Enum-Value) defined
/// within a specific [`EnumType`].
#[derive(Clone, Debug, PartialEq)]
pub struct EnumValue {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) name: String,
    pub(super) type_ref: NamedGraphQLTypeRef,
}

impl EnumValue {
    /// The [`SchemaDefLocation`](loc::SchemaDefLocation) indicating where this
    /// [`EnumValue`] was defined within the schema.
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`EnumValue`].
    ///
    /// This list of [`DirectiveAnnotation`]s is guaranteed to be ordered the same
    /// as the order of annotations specified on the [`EnumValue`] definition in
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

    /// The [`EnumType`] that this [`EnumValue`] belongs to.
    pub fn enum_type<'schema>(
        &self,
        schema: &'schema Schema,
    ) -> &'schema EnumType {
        self.type_ref.deref(schema)
            .expect("type is present in schema")
            .as_enum()
            .expect("type is an enum type")
    }

    /// The name of the [`EnumType`] type to which this value belongs.
    ///
    /// This can be useful when the [`Schema`] object is unavailable or
    /// inconvenient to access but the type's name is all that's needed.
    pub fn enum_type_name(&self) -> &str {
        self.type_ref.name.as_str()
    }
}

impl DerefByName for EnumValue {
    type Source = EnumType;

    fn deref_name<'a>(
        enum_type: &'a Self::Source,
        value_name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        enum_type.values.get(value_name).ok_or_else(
            || DerefByNameError::DanglingReference(value_name.to_string())
        )
    }
}

pub type NamedEnumValueRef = NamedRef<EnumType, EnumValue>;
