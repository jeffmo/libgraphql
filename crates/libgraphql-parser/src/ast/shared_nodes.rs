use crate::ast::DelimiterPair;
use crate::ast::Name;
use crate::ast::StringValue;
use crate::ast::TypeAnnotation;
use crate::ast::Value;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;

// =========================================================
// Field definitions (used in object types, interfaces)
// =========================================================

/// A field definition within an object type or interface
/// type.
///
/// See
/// [Field Definitions](https://spec.graphql.org/September2025/#FieldsDefinition)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub arguments: Vec<InputValueDefinition<'src>>,
    pub field_type: TypeAnnotation<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<FieldDefinitionSyntax<'src>>,
}

/// Syntax detail for a [`FieldDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct FieldDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
    pub argument_parens: Option<DelimiterPair<'src>>,
}

// =========================================================
// Input value definitions (arguments, input fields)
// =========================================================

/// An input value definition, used for field arguments and
/// input object fields.
///
/// See
/// [Input Values Definitions](https://spec.graphql.org/September2025/#InputValueDefinition)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct InputValueDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub value_type: TypeAnnotation<'src>,
    pub default_value: Option<Value<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<InputValueDefinitionSyntax<'src>>,
}

/// Syntax detail for an [`InputValueDefinition`].
#[derive(Clone, Debug, PartialEq)]
pub struct InputValueDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
}

// =========================================================
// Enum value definitions
// =========================================================

/// An enum value definition within an enum type.
///
/// See
/// [Enum Value Definitions](https://spec.graphql.org/September2025/#EnumValuesDefinition)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumValueDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
}

// =========================================================
// Directive annotations (applied directives)
// =========================================================

/// A directive annotation applied to a definition or field
/// (e.g. `@deprecated(reason: "Use newField")`).
///
/// See
/// [Directives](https://spec.graphql.org/September2025/#sec-Language.Directives)
/// in the spec. Note: this represents an *applied* directive
/// (an annotation), not a directive *definition*.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveAnnotation<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub arguments: Vec<Argument<'src>>,
    pub syntax: Option<DirectiveAnnotationSyntax<'src>>,
}

/// Syntax detail for a [`DirectiveAnnotation`].
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveAnnotationSyntax<'src> {
    pub at_sign: GraphQLToken<'src>,
    pub argument_parens: Option<DelimiterPair<'src>>,
}

// =========================================================
// Arguments
// =========================================================

/// A single argument in a field, directive, or field
/// definition.
///
/// See
/// [Arguments](https://spec.graphql.org/September2025/#sec-Language.Arguments)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct Argument<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub value: Value<'src>,
    pub syntax: Option<ArgumentSyntax<'src>>,
}

/// Syntax detail for an [`Argument`].
#[derive(Clone, Debug, PartialEq)]
pub struct ArgumentSyntax<'src> {
    pub colon: GraphQLToken<'src>,
}

// =========================================================
// Type condition
// =========================================================

/// A type condition (e.g., `on User`) used in fragment
/// definitions and inline fragments.
///
/// See
/// [Type Conditions](https://spec.graphql.org/September2025/#sec-Type-Conditions)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeCondition<'src> {
    pub span: GraphQLSourceSpan,
    pub named_type: Name<'src>,
    pub syntax: Option<TypeConditionSyntax<'src>>,
}

/// Syntax detail for a [`TypeCondition`].
#[derive(Clone, Debug, PartialEq)]
pub struct TypeConditionSyntax<'src> {
    pub on_keyword: GraphQLToken<'src>,
}
