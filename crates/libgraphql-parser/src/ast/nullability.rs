use crate::token::GraphQLToken;

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
