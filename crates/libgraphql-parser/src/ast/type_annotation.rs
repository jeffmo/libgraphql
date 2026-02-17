use crate::ast::DelimiterPair;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;

/// The nullability of a
/// [type reference](https://spec.graphql.org/September2025/#sec-Type-References).
///
/// Rather than modeling `NonNullType` as a recursive enum
/// variant (which would allow redundant same-level wrapping
/// like `NonNull(NonNull(...))`), nullability is flattened
/// into this enum on each concrete type annotation node.
///
/// Multi-level `NonNull` (e.g. `[String!]!`) is fully
/// supported: the inner `String!` is the list's
/// `element_type` (a separate [`TypeAnnotation`] with its own
/// `Nullability`), and the outer `!` is on the
/// [`ListTypeAnnotation`].
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum Nullability<'src> {
    NonNull {
        /// The `!` token. Present when syntax detail is
        /// retained.
        syntax: Option<GraphQLToken<'src>>,
    },
    Nullable,
}

/// A GraphQL
/// [type reference](https://spec.graphql.org/September2025/#sec-Type-References)
/// (type annotation).
///
/// Represents `NamedType` and `ListType` from the spec
/// grammar. The spec's `NonNullType` is not a separate
/// variant here — instead, nullability is expressed via the
/// [`Nullability`] field on each variant's inner struct.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum TypeAnnotation<'src> {
    List(ListTypeAnnotation<'src>),
    Named(NamedTypeAnnotation<'src>),
}

/// A named type reference (e.g. `String`, `String!`).
///
/// See
/// [Type References](https://spec.graphql.org/September2025/#sec-Type-References)
/// in the spec. The `span` covers the full annotation
/// including `!` when present. The underlying name span is
/// available via `name.span`.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedTypeAnnotation<'src> {
    pub name: Name<'src>,
    pub nullability: Nullability<'src>,
    pub span: GraphQLSourceSpan,
}

/// A list type reference (e.g. `[String]`, `[String!]!`).
///
/// See
/// [Type References](https://spec.graphql.org/September2025/#sec-Type-References)
/// in the spec. The `span` covers brackets and trailing `!`
/// when present.
#[derive(Clone, Debug, PartialEq)]
pub struct ListTypeAnnotation<'src> {
    pub element_type: Box<TypeAnnotation<'src>>,
    pub nullability: Nullability<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<ListTypeAnnotationSyntax<'src>>,
}

/// Syntax detail for a [`ListTypeAnnotation`].
#[derive(Clone, Debug, PartialEq)]
pub struct ListTypeAnnotationSyntax<'src> {
    pub brackets: DelimiterPair<'src>,
}
