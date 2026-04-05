use crate::names::TypeName;
use crate::span::Span;
use crate::types::enum_type::EnumType;
use crate::types::graphql_type_kind::GraphQLTypeKind;
use crate::types::input_object_type::InputObjectType;
use crate::types::interface_type::InterfaceType;
use crate::types::object_type::ObjectType;
use crate::types::scalar_type::ScalarType;
use crate::types::union_type::UnionType;

/// A defined GraphQL type.
///
/// This enum has 6 data-carrying variants — one per type
/// category. Built-in scalars (`Boolean`, `Float`, `ID`, `Int`,
/// `String`) are represented as
/// `Scalar(ScalarType { kind: ScalarKind::Boolean, .. })` rather
/// than separate enum variants, keeping accessor methods at 6
/// match arms instead of 11.
///
/// For exhaustive matching that distinguishes built-in scalar
/// identity, use [`type_kind()`](Self::type_kind) which returns
/// an 11-variant [`GraphQLTypeKind`].
///
/// See [Types](https://spec.graphql.org/September2025/#sec-Types).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum GraphQLType {
    Enum(Box<EnumType>),
    InputObject(Box<InputObjectType>),
    Interface(Box<InterfaceType>),
    Object(Box<ObjectType>),
    Scalar(Box<ScalarType>),
    Union(Box<UnionType>),
}

impl GraphQLType {
    #[inline]
    pub fn name(&self) -> &TypeName {
        match self {
            Self::Enum(t) => t.name(),
            Self::InputObject(t) => t.name(),
            Self::Interface(t) => t.name(),
            Self::Object(t) => t.name(),
            Self::Scalar(t) => t.name(),
            Self::Union(t) => t.name(),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Enum(t) => t.span(),
            Self::InputObject(t) => t.span(),
            Self::Interface(t) => t.span(),
            Self::Object(t) => t.span(),
            Self::Scalar(t) => t.span(),
            Self::Union(t) => t.span(),
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            Self::Enum(t) => t.description(),
            Self::InputObject(t) => t.description(),
            Self::Interface(t) => t.description(),
            Self::Object(t) => t.description(),
            Self::Scalar(t) => t.description(),
            Self::Union(t) => t.description(),
        }
    }

    /// Input types can appear in input positions (arguments,
    /// variables, input object fields): Scalar, Enum, InputObject.
    ///
    /// Note that Scalar and Enum are both input *and* output types,
    /// so `is_input_type()` and [`is_output_type()`](Self::is_output_type)
    /// are not opposites.
    ///
    /// See [Input and Output Types](https://spec.graphql.org/September2025/#sec-Input-and-Output-Types).
    pub fn is_input_type(&self) -> bool {
        matches!(
            self,
            Self::Enum(_) | Self::InputObject(_) | Self::Scalar(_),
        )
    }

    /// Output types can appear in output positions (field return
    /// types): Scalar, Enum, Object, Interface, Union.
    ///
    /// Note that Scalar and Enum are both input *and* output types,
    /// so `is_output_type()` and [`is_input_type()`](Self::is_input_type)
    /// are not opposites.
    ///
    /// See [Input and Output Types](https://spec.graphql.org/September2025/#sec-Input-and-Output-Types).
    pub fn is_output_type(&self) -> bool {
        matches!(
            self,
            Self::Enum(_) | Self::Interface(_) | Self::Object(_)
                | Self::Scalar(_) | Self::Union(_),
        )
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self, Self::Scalar(s) if s.is_builtin())
    }

    /// Composite types can have selection sets: Object,
    /// Interface, Union.
    pub fn is_composite_type(&self) -> bool {
        matches!(
            self,
            Self::Interface(_) | Self::Object(_) | Self::Union(_),
        )
    }

    /// Leaf types cannot have selection sets: Scalar, Enum.
    pub fn is_leaf_type(&self) -> bool {
        matches!(self, Self::Enum(_) | Self::Scalar(_))
    }

    /// Returns the fully-discriminated type kind, including
    /// built-in scalar identity (11 variants).
    pub fn type_kind(&self) -> GraphQLTypeKind {
        match self {
            Self::Enum(_) => GraphQLTypeKind::Enum,
            Self::InputObject(_) => GraphQLTypeKind::InputObject,
            Self::Interface(_) => GraphQLTypeKind::Interface,
            Self::Object(_) => GraphQLTypeKind::Object,
            Self::Scalar(s) => s.kind().into(),
            Self::Union(_) => GraphQLTypeKind::Union,
        }
    }

    pub fn as_enum(&self) -> Option<&EnumType> {
        if let Self::Enum(t) = self { Some(t) } else { None }
    }
    pub fn as_input_object(&self) -> Option<&InputObjectType> {
        if let Self::InputObject(t) = self { Some(t) } else { None }
    }
    pub fn as_interface(&self) -> Option<&InterfaceType> {
        if let Self::Interface(t) = self { Some(t) } else { None }
    }
    pub fn as_object(&self) -> Option<&ObjectType> {
        if let Self::Object(t) = self { Some(t) } else { None }
    }
    pub fn as_scalar(&self) -> Option<&ScalarType> {
        if let Self::Scalar(t) = self { Some(t) } else { None }
    }
    pub fn as_union(&self) -> Option<&UnionType> {
        if let Self::Union(t) = self { Some(t) } else { None }
    }
}
