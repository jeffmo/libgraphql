use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::schema::Schema;
use crate::types::EnumType;
use crate::types::InputObjectType;
use crate::types::InterfaceType;
use crate::types::ObjectType;
use crate::types::ScalarType;
use crate::types::UnionType;
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
    pub fn as_enum(&self) -> Option<&EnumType> {
        if let Self::Enum(type_) = self {
            Some(type_)
        } else {
            None
        }
    }

    pub fn as_interface(&self) -> Option<&InterfaceType> {
        if let Self::Interface(type_) = self {
            Some(type_)
        } else {
            None
        }
    }

    pub fn as_object(&self) -> Option<&ObjectType> {
        if let Self::Object(type_) = self {
            Some(type_)
        } else {
            None
        }
    }

    pub fn as_scalar(&self) -> Option<&ScalarType> {
        if let Self::Scalar(type_) = self {
            Some(type_)
        } else {
            None
        }
    }

    // TODO: Change this to return Option<&loc::SchemaDefLocation>
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

    pub fn name(&self) -> Option<&str> {
        match self {
            GraphQLType::Bool
                | GraphQLType::Float
                | GraphQLType::ID
                | GraphQLType::Int
                | GraphQLType::String => None,
            GraphQLType::Enum(t) => Some(t.name.as_str()),
            GraphQLType::InputObject(t) => Some(t.name.as_str()),
            GraphQLType::Interface(t) => Some(t.name()),
            GraphQLType::Object(t) => Some(t.name()),
            GraphQLType::Scalar(t) => Some(t.name.as_str()),
            GraphQLType::Union(t) => Some(t.name.as_str()),
        }
    }

    pub fn requires_selection_set(&self) -> bool {
        matches!(
            self,
            Self::Interface(_) | Self::Object(_) | Self::Union(_),
        )
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
