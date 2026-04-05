use crate::span::Span;

/// A value paired with the [`Span`] of its occurrence in source.
///
/// Used for name references that need to trace back to their
/// source location — e.g., each interface name in an `implements`
/// clause, or each member name in a union definition. The inner
/// value provides identity (for lookups), while the span provides
/// location (for error reporting).
///
/// `Located<T>` deliberately does **not** implement `Eq` or
/// `Hash`. Use the inner `.value` for identity comparisons and
/// map lookups.
///
/// # Example
///
/// ```ignore
/// use libgraphql_core::Located;
/// use libgraphql_core::names::TypeName;
/// use libgraphql_core::span::Span;
///
/// let located = Located {
///     value: TypeName::new("Node"),
///     span: Span::builtin(),
/// };
/// assert_eq!(located.value.as_str(), "Node");
/// ```
#[derive(Clone, Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Located<T> {
    pub value: T,
    pub span: Span,
}

impl<T> AsRef<T> for Located<T> {
    fn as_ref(&self) -> &T { &self.value }
}
