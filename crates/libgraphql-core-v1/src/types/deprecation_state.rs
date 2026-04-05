/// Deprecation status of a type, field, enum value, or argument,
/// derived from the presence of a
/// [`@deprecated`](https://spec.graphql.org/September2025/#sec--deprecated)
/// directive annotation.
#[derive(Clone, Debug, PartialEq)]
pub enum DeprecationState<'a> {
    Active,
    Deprecated { reason: Option<&'a str> },
}

impl DeprecationState<'_> {
    #[inline]
    pub fn is_deprecated(&self) -> bool {
        matches!(self, Self::Deprecated { .. })
    }
}
