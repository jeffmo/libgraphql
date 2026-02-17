use crate::ast::DelimiterPair;
use crate::ast::DirectiveAnnotation;
use crate::ast::DirectiveLocation;
use crate::ast::EnumValueDefinition;
use crate::ast::FieldDefinition;
use crate::ast::InputValueDefinition;
use crate::ast::Name;
use crate::ast::OperationKind;
use crate::ast::StringValue;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;

// =========================================================
// Schema definition
// =========================================================

/// A GraphQL schema definition.
///
/// See
/// [Schema](https://spec.graphql.org/September2025/#sec-Schema)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub root_operations:
        Vec<RootOperationTypeDefinition<'src>>,
    pub syntax: Option<SchemaDefinitionSyntax<'src>>,
}

/// A root operation type definition within a schema
/// definition (e.g. `query: Query`).
///
/// See
/// [Root Operation Types](https://spec.graphql.org/September2025/#sec-Root-Operation-Types)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct RootOperationTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub operation_kind: OperationKind,
    pub named_type: Name<'src>,
    pub syntax:
        Option<RootOperationTypeDefinitionSyntax<'src>>,
}

// =========================================================
// Type definitions
// =========================================================

/// A type definition in a GraphQL schema.
///
/// See
/// [Types](https://spec.graphql.org/September2025/#sec-Types)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeDefinition<'src> {
    Enum(EnumTypeDefinition<'src>),
    InputObject(InputObjectTypeDefinition<'src>),
    Interface(InterfaceTypeDefinition<'src>),
    Object(ObjectTypeDefinition<'src>),
    Scalar(ScalarTypeDefinition<'src>),
    Union(UnionTypeDefinition<'src>),
}

/// A scalar type definition (e.g. `scalar DateTime`).
///
/// See
/// [Scalars](https://spec.graphql.org/September2025/#sec-Scalars)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<ScalarTypeDefinitionSyntax<'src>>,
}

/// An object type definition.
///
/// See
/// [Objects](https://spec.graphql.org/September2025/#sec-Objects)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub implements: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax:
        Option<ObjectTypeDefinitionSyntax<'src>>,
}

/// An interface type definition.
///
/// See
/// [Interfaces](https://spec.graphql.org/September2025/#sec-Interfaces)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub implements: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax:
        Option<InterfaceTypeDefinitionSyntax<'src>>,
}

/// A union type definition.
///
/// See
/// [Unions](https://spec.graphql.org/September2025/#sec-Unions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub members: Vec<Name<'src>>,
    pub syntax: Option<UnionTypeDefinitionSyntax<'src>>,
}

/// An enum type definition.
///
/// See
/// [Enums](https://spec.graphql.org/September2025/#sec-Enums)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub values: Vec<EnumValueDefinition<'src>>,
    pub syntax: Option<EnumTypeDefinitionSyntax<'src>>,
}

/// An input object type definition.
///
/// See
/// [Input Objects](https://spec.graphql.org/September2025/#sec-Input-Objects)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<InputValueDefinition<'src>>,
    pub syntax:
        Option<InputObjectTypeDefinitionSyntax<'src>>,
}

// =========================================================
// Directive definition
// =========================================================

/// A directive definition.
///
/// See
/// [Directive Definitions](https://spec.graphql.org/September2025/#sec-Type-System.Directives)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub arguments: Vec<InputValueDefinition<'src>>,
    pub repeatable: bool,
    pub locations: Vec<DirectiveLocation<'src>>,
    pub syntax: Option<DirectiveDefinitionSyntax<'src>>,
}

// =========================================================
// Type system definition syntax structs
// =========================================================

/// Syntax detail for a [`SchemaDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaDefinitionSyntax<'src> {
    pub schema_keyword: GraphQLToken<'src>,
    pub braces: DelimiterPair<'src>,
}

/// Syntax detail for a
/// [`RootOperationTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct RootOperationTypeDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
}

/// Syntax detail for a [`ScalarTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarTypeDefinitionSyntax<'src> {
    pub scalar_keyword: GraphQLToken<'src>,
}

/// Syntax detail for an [`ObjectTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectTypeDefinitionSyntax<'src> {
    pub type_keyword: GraphQLToken<'src>,
    pub implements_keyword: Option<GraphQLToken<'src>>,
    pub leading_ampersand: Option<GraphQLToken<'src>>,
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
}

/// Syntax detail for an [`InterfaceTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceTypeDefinitionSyntax<'src> {
    pub interface_keyword: GraphQLToken<'src>,
    pub implements_keyword: Option<GraphQLToken<'src>>,
    pub leading_ampersand: Option<GraphQLToken<'src>>,
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
}

/// Syntax detail for a [`UnionTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct UnionTypeDefinitionSyntax<'src> {
    pub union_keyword: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
    pub leading_pipe: Option<GraphQLToken<'src>>,
    pub pipes: Vec<GraphQLToken<'src>>,
}

/// Syntax detail for an [`EnumTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct EnumTypeDefinitionSyntax<'src> {
    pub enum_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}

/// Syntax detail for an [`InputObjectTypeDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectTypeDefinitionSyntax<'src> {
    pub input_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}

/// Syntax detail for a [`DirectiveDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveDefinitionSyntax<'src> {
    pub directive_keyword: GraphQLToken<'src>,
    pub at_sign: GraphQLToken<'src>,
    pub argument_parens: Option<DelimiterPair<'src>>,
    pub repeatable_keyword: Option<GraphQLToken<'src>>,
    pub on_keyword: GraphQLToken<'src>,
}
