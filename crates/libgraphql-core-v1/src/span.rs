use libgraphql_parser::ByteSpan;

/// Identifies a source map within a
/// [`Schema`](crate::schema::Schema)'s collection of source maps.
///
/// Index `0` ([`BUILTIN_SOURCE_MAP_ID`]) is reserved for built-in
/// definitions that have no user-authored source.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct SourceMapId(pub(crate) u16);

/// The source map ID for built-in types and directives (`Boolean`,
/// `String`, `@skip`, `@include`, etc.).
pub const BUILTIN_SOURCE_MAP_ID: SourceMapId = SourceMapId(0);

/// A compact source location: a byte-offset range paired with the
/// [`SourceMapId`] of the source it belongs to.
///
/// At 12 bytes and `Copy`, `Span` is designed to be stored on every
/// AST-derived semantic node without significant memory overhead.
/// Line/column resolution is deferred until needed, via the
/// corresponding [`SchemaSourceMap`](crate::SchemaSourceMap) stored
/// on the [`Schema`](crate::schema::Schema).
///
/// See [`ByteSpan`](libgraphql_parser::ByteSpan) for the
/// underlying byte-offset representation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Span {
    pub byte_span: ByteSpan,
    pub source_map_id: SourceMapId,
}

impl Span {
    pub fn new(byte_span: ByteSpan, source_map_id: SourceMapId) -> Self {
        Self { byte_span, source_map_id }
    }

    /// A zero-width span for built-in definitions (`Boolean`,
    /// `@skip`, etc.) that have no user-authored source.
    #[inline]
    pub fn builtin() -> Self {
        Self {
            byte_span: ByteSpan::empty_at(0),
            source_map_id: BUILTIN_SOURCE_MAP_ID,
        }
    }

    /// A zero-width span for nodes constructed programmatically
    /// without source text. Semantically distinct from
    /// [`builtin()`](Self::builtin) (which is for spec-defined
    /// built-in definitions), though both carry the same
    /// representation.
    ///
    /// Use this when constructing types imperatively rather than
    /// from parsed source — e.g., in tests or code-generation
    /// scenarios where no `.graphql` source text exists.
    #[inline]
    pub fn dummy() -> Self {
        Self {
            byte_span: ByteSpan::empty_at(0),
            source_map_id: BUILTIN_SOURCE_MAP_ID,
        }
    }
}
