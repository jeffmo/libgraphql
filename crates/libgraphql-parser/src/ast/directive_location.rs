use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;

/// A directive location with its own span (unlike
/// `graphql_parser` which uses a plain enum).
///
/// See
/// [Directive Locations](https://spec.graphql.org/September2025/#DirectiveLocations)
/// in the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveLocation<'src> {
    pub kind: DirectiveLocationKind,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<DirectiveLocationSyntax<'src>>,
}

/// The kind of location where a directive may be applied.
///
/// See
/// [Directive Locations](https://spec.graphql.org/September2025/#DirectiveLocations)
/// in the spec.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DirectiveLocationKind {
    ArgumentDefinition,
    Enum,
    EnumValue,
    Field,
    FieldDefinition,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    InputFieldDefinition,
    InputObject,
    Interface,
    Mutation,
    Object,
    Query,
    Scalar,
    Schema,
    Subscription,
    Union,
    VariableDefinition,
}

/// Syntax detail for a [`DirectiveLocation`].
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveLocationSyntax<'src> {
    /// The `|` pipe token before this location (`None` for
    /// the first location).
    pub pipe: Option<GraphQLToken<'src>>,
    /// The location name token (e.g. `FIELD`, `QUERY`).
    pub token: GraphQLToken<'src>,
}
