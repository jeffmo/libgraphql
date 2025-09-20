use crate::types::GraphQLType;

/// Similar to [`GraphQLType`] except without the corresponding type metadata.
/// Useful when representing a group or category of [`GraphQLType`]s.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphQLTypeKind {
    Bool,
    Enum,
    Float,
    ID,
    InputObject,
    Int,
    Interface,
    Object,
    Scalar,
    String,
    Union,
}
impl GraphQLTypeKind {
    pub fn name(&self) -> &str {
        match self {
            Self::Bool => "Boolean",
            Self::Enum => "Enum",
            Self::Float => "Float",
            Self::ID => "ID",
            Self::InputObject => "InputObject",
            Self::Int => "Int",
            Self::Interface => "Interface",
            Self::Object => "Object",
            Self::Scalar => "Scalar",
            Self::String => "String",
            Self::Union => "Union",
        }
    }
}
impl std::convert::From<&GraphQLType> for GraphQLTypeKind {
    fn from(value: &GraphQLType) -> Self {
        match value {
            GraphQLType::Bool => GraphQLTypeKind::Bool,
            GraphQLType::Enum(_) => GraphQLTypeKind::Enum,
            GraphQLType::Float => GraphQLTypeKind::Float,
            GraphQLType::ID => GraphQLTypeKind::ID,
            GraphQLType::InputObject(_) => GraphQLTypeKind::InputObject,
            GraphQLType::Int => GraphQLTypeKind::Int,
            GraphQLType::Interface(_) => GraphQLTypeKind::Interface,
            GraphQLType::Object(_) => GraphQLTypeKind::Object,
            GraphQLType::Scalar(_) => GraphQLTypeKind::Scalar,
            GraphQLType::String => GraphQLTypeKind::String,
            GraphQLType::Union(_) => GraphQLTypeKind::Union,
        }
    }
}
