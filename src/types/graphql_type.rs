use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::schema::Schema;
use crate::types::DeprecationState;
use crate::types::EnumType;
use crate::types::GraphQLTypeKind;
use crate::types::InputObjectType;
use crate::types::InterfaceType;
use crate::types::NamedTypeAnnotation;
use crate::types::ObjectType;
use crate::types::ScalarType;
use crate::types::TypeAnnotation;
use crate::types::UnionType;
use crate::NamedRef;
use std::boxed::Box;

/// Represents a defined GraphQL type
#[derive(Clone, Debug, PartialEq)]
pub enum GraphQLType {
    Bool,
    Enum(Box<EnumType>),
    Float,
    ID,
    InputObject(Box<InputObjectType>),
    Int,
    Interface(Box<InterfaceType>),
    Object(Box<ObjectType>),
    Scalar(Box<ScalarType>),
    String,
    Union(Box<UnionType>),
}
impl GraphQLType {
    /// If this [`GraphQLType`] is a [`GraphQLType::Enum`], unwrap and return
    /// a reference to the inner [`EnumType`].
    pub fn as_enum(&self) -> Option<&EnumType> {
        if let Self::Enum(type_) = self {
            Some(type_)
        } else {
            None
        }
    }

    /// If this [`GraphQLType`] is a [`GraphQLType::Interface`], unwrap and
    /// return a reference to the inner [`InterfaceType`].
    pub fn as_interface(&self) -> Option<&InterfaceType> {
        if let Self::Interface(type_) = self {
            Some(type_)
        } else {
            None
        }
    }

    /// If this [`GraphQLType`] is a [`GraphQLType::Object`], unwrap and return
    /// a reference to the inner [`ObjectType`].
    pub fn as_object(&self) -> Option<&ObjectType> {
        if let Self::Object(type_) = self {
            Some(type_)
        } else {
            None
        }
    }

    /// If this [`GraphQLType`] is a [`GraphQLType::Scalar`], unwrap and return
    /// a reference to the inner [`ScalarType`].
    pub fn as_scalar(&self) -> Option<&ScalarType> {
        if let Self::Scalar(type_) = self {
            Some(type_)
        } else {
            None
        }
    }

    pub fn as_type_annotation(&self, nullable: bool) -> TypeAnnotation {
        TypeAnnotation::Named(NamedTypeAnnotation {
            nullable,
            type_ref: NamedRef::new(
                self.name(),
                self.def_location(),
            ),
        })
    }

    /// If this [`GraphQLType`] is a [`GraphQLType::Union`], unwrap and return
    /// a reference to the inner [`UnionType`].
    pub fn as_union(&self) -> Option<&UnionType> {
        if let Self::Union(type_) = self {
            Some(type_)
        } else {
            None
        }
    }

    /// The [loc::SchemaDefLocation] indicating where this [GraphQLType] was
    /// defined within the schema.
    pub fn def_location(&self) -> loc::SchemaDefLocation {
        match self {
            GraphQLType::Bool
                | GraphQLType::Float
                | GraphQLType::ID
                | GraphQLType::Int
                | GraphQLType::String =>
                loc::SchemaDefLocation::GraphQLBuiltIn,
            GraphQLType::Enum(t) =>
                t.def_location.clone(),
            GraphQLType::InputObject(t) =>
                t.def_location.clone(),
            GraphQLType::Interface(t) =>
                t.def_location().clone(),
            GraphQLType::Object(t) =>
                t.def_location().clone(),
            GraphQLType::Scalar(t) =>
                t.def_location.clone(),
            GraphQLType::Union(t) =>
                t.def_location.clone(),
        }
    }

    /// The description of this [`GraphQLType`] as defined in the schema
    /// (e.g. in a """-string immediately before the type definition).
    pub fn description(&self) -> Option<&str> {
        match self {
            GraphQLType::Bool => None,
            GraphQLType::Enum(t) => t.description(),
            GraphQLType::Float => None,
            GraphQLType::ID => None,
            GraphQLType::InputObject(t) => t.description(),
            GraphQLType::Int => None,
            GraphQLType::Interface(t) => t.description(),
            GraphQLType::Object(t) => t.description(),
            GraphQLType::Scalar(t) => t.description(),
            GraphQLType::String => None,
            GraphQLType::Union(t) => t.description(),
        }
    }

    /// The [`DeprecationState`] of this [`GraphQLType`] as indicated by the
    /// presence of a `@deprecated` annotation.
    pub fn deprecation_state(&self) -> DeprecationState<'_> {
        match self {
            GraphQLType::Bool => DeprecationState::NotDeprecated,
            GraphQLType::Enum(t) => t.deprecation_state(),
            GraphQLType::Float => DeprecationState::NotDeprecated,
            GraphQLType::ID => DeprecationState::NotDeprecated,
            GraphQLType::InputObject(t) => t.deprecation_state(),
            GraphQLType::Int => DeprecationState::NotDeprecated,
            GraphQLType::Interface(t) => t.deprecation_state(),
            GraphQLType::Object(t) => t.deprecation_state(),
            GraphQLType::Scalar(t) => t.deprecation_state(),
            GraphQLType::String => DeprecationState::NotDeprecated,
            GraphQLType::Union(t) => t.deprecation_state(),
        }
    }

    /// Indicates whether this [`GraphQLType`] is built-in (vs one that was
    /// explicitly defined while building the [`Schema`]).
    pub fn is_builtin(&self) -> bool {
        matches!(self.def_location(), loc::SchemaDefLocation::GraphQLBuiltIn)
    }

    /// Indicates if this type can be used in an input position (e.g. as the
    /// type of an [`InputField`](crate::types::InputField), a
    /// [`Parameter`](crate::types::Parameter)), or a
    /// [`Variable`](crate::operation::Variable)).
    pub fn is_input_type(&self) -> bool {
        match self {
            GraphQLType::Bool
            | GraphQLType::Enum(_)
            | GraphQLType::Float
            | GraphQLType::ID
            | GraphQLType::InputObject(_)
            | GraphQLType::Int
            | GraphQLType::Scalar(_)
            | GraphQLType::String
                => true,

            GraphQLType::Interface(_)
            | GraphQLType::Object(_)
            | GraphQLType::Union(_)
                => false,
        }
    }

    /// Indicates if this type can be used in an output position (e.g. as the
    /// type of a [`Field`](crate::types::Field)).
    pub fn is_output_type(&self) -> bool {
        match self {
            GraphQLType::Bool
            | GraphQLType::Enum(_)
            | GraphQLType::Float
            | GraphQLType::ID
            | GraphQLType::Int
            | GraphQLType::Interface(_)
            | GraphQLType::Object(_)
            | GraphQLType::Scalar(_)
            | GraphQLType::String
            | GraphQLType::Union(_)
                => true,

            GraphQLType::InputObject(_)
                => false,
        }
    }

    /// The name of this [`GraphQLType`].
    pub fn name(&self) -> &str {
        match self {
            GraphQLType::Bool => "Boolean",
            GraphQLType::Enum(t) => t.name.as_str(),
            GraphQLType::Float => "Float",
            GraphQLType::ID => "ID",
            GraphQLType::InputObject(t) => t.name.as_str(),
            GraphQLType::Int => "Int",
            GraphQLType::Interface(t) => t.name(),
            GraphQLType::Object(t) => t.name(),
            GraphQLType::Scalar(t) => t.name.as_str(),
            GraphQLType::String => "String",
            GraphQLType::Union(t) => t.name.as_str(),
        }
    }

    /// Indicates whether an operation that selects a
    /// [`Field`](crate::types::Field) of this type must specify a selection set
    /// for that field.
    pub fn requires_selection_set(&self) -> bool {
        matches!(
            self,
            Self::Interface(_) | Self::Object(_) | Self::Union(_),
        )
    }

    /// Produces the corresponding [`GraphQLTypeKind`] for this [`GraphQLType`].
    pub fn type_kind(&self) -> GraphQLTypeKind {
        self.into()
    }
}
impl DerefByName for GraphQLType {
    type Source = Schema;

    fn deref_name<'a>(
        schema: &'a Schema,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        schema.types.get(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string()),
        )
    }
}
