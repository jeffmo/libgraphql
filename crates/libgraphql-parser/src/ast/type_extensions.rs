use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::EnumValueDefinition;
use crate::ast::FieldDefinition;
use crate::ast::InputValueDefinition;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;

// =========================================================
// Schema extension
// =========================================================

/// A schema extension.
///
/// See
/// [Schema Extension](https://spec.graphql.org/September2025/#sec-Schema-Extension)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub root_operations:
        Vec<crate::ast::RootOperationTypeDefinition<'src>>,
    pub syntax: Option<SchemaExtensionSyntax<'src>>,
}

// =========================================================
// Type extensions
// =========================================================

/// A type extension in a GraphQL schema.
///
/// See
/// [Type Extensions](https://spec.graphql.org/September2025/#sec-Type-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeExtension<'src> {
    Enum(EnumTypeExtension<'src>),
    InputObject(InputObjectTypeExtension<'src>),
    Interface(InterfaceTypeExtension<'src>),
    Object(ObjectTypeExtension<'src>),
    Scalar(ScalarTypeExtension<'src>),
    Union(UnionTypeExtension<'src>),
}

/// A scalar type extension.
///
/// See
/// [Scalar Extensions](https://spec.graphql.org/September2025/#sec-Scalar-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<ScalarTypeExtensionSyntax<'src>>,
}

/// An object type extension.
///
/// See
/// [Object Extensions](https://spec.graphql.org/September2025/#sec-Object-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub implements: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax: Option<ObjectTypeExtensionSyntax<'src>>,
}

/// An interface type extension.
///
/// See
/// [Interface Extensions](https://spec.graphql.org/September2025/#sec-Interface-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub implements: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax:
        Option<InterfaceTypeExtensionSyntax<'src>>,
}

/// A union type extension.
///
/// See
/// [Union Extensions](https://spec.graphql.org/September2025/#sec-Union-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub members: Vec<Name<'src>>,
    pub syntax: Option<UnionTypeExtensionSyntax<'src>>,
}

/// An enum type extension.
///
/// See
/// [Enum Extensions](https://spec.graphql.org/September2025/#sec-Enum-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub values: Vec<EnumValueDefinition<'src>>,
    pub syntax: Option<EnumTypeExtensionSyntax<'src>>,
}

/// An input object type extension.
///
/// See
/// [Input Object Extensions](https://spec.graphql.org/September2025/#sec-Input-Object-Extensions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<InputValueDefinition<'src>>,
    pub syntax:
        Option<InputObjectTypeExtensionSyntax<'src>>,
}

// =========================================================
// Extension syntax structs
// =========================================================

/// Syntax detail for a [`SchemaExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub schema_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}

/// Syntax detail for a [`ScalarTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub scalar_keyword: GraphQLToken<'src>,
}

/// Syntax detail for an [`ObjectTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub type_keyword: GraphQLToken<'src>,
    pub implements_keyword: Option<GraphQLToken<'src>>,
    pub leading_ampersand: Option<GraphQLToken<'src>>,
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
}

/// Syntax detail for an [`InterfaceTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub interface_keyword: GraphQLToken<'src>,
    pub implements_keyword: Option<GraphQLToken<'src>>,
    pub leading_ampersand: Option<GraphQLToken<'src>>,
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
}

/// Syntax detail for a [`UnionTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub union_keyword: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
    pub leading_pipe: Option<GraphQLToken<'src>>,
    pub pipes: Vec<GraphQLToken<'src>>,
}

/// Syntax detail for an [`EnumTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct EnumTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub enum_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}

/// Syntax detail for an [`InputObjectTypeExtension`].
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub input_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}
