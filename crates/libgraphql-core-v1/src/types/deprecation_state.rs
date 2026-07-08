use crate::directive_annotation::DirectiveAnnotation;
use crate::value::Value;

/// Deprecation status of a type, field, enum value, or argument,
/// derived from the presence of a
/// [`@deprecated`](https://spec.graphql.org/September2025/#sec--deprecated)
/// directive annotation.
#[derive(Clone, Debug, PartialEq)]
pub enum DeprecationState<'a> {
    Active,
    Deprecated { reason: Option<&'a str> },
}

impl<'a> DeprecationState<'a> {
    /// The default value of the `@deprecated` directive's `reason`
    /// argument, as defined by the built-in directive definition
    /// `directive @deprecated(reason: String! = "No longer supported")`.
    ///
    /// See
    /// [@deprecated](https://spec.graphql.org/September2025/#sec--deprecated).
    pub const DEFAULT_REASON: &'static str = "No longer supported";

    /// Derives a [`DeprecationState`] from a list of directive
    /// annotations.
    ///
    /// If a `@deprecated` annotation is present, the returned state
    /// is [`DeprecationState::Deprecated`] with its `reason` taken
    /// from the annotation's `reason` argument. When the `reason`
    /// argument is omitted, the built-in definition's default value
    /// ([`DeprecationState::DEFAULT_REASON`]) applies. A `reason`
    /// that is explicitly `null` or a non-string value (both invalid
    /// per the `String!` parameter type, but tolerated here) yields
    /// a `reason` of `None`.
    ///
    /// See
    /// [@deprecated](https://spec.graphql.org/September2025/#sec--deprecated).
    pub(crate) fn from_directives(directives: &'a [DirectiveAnnotation]) -> Self {
        let Some(annot) = find_deprecated_annotation(directives) else {
            return Self::Active;
        };
        let reason = match annot.arguments().get("reason") {
            None => Some(Self::DEFAULT_REASON),
            Some(Value::String(reason)) => Some(reason.as_str()),
            Some(_) => None,
        };
        Self::Deprecated { reason }
    }

    #[inline]
    pub fn is_deprecated(&self) -> bool {
        matches!(self, Self::Deprecated { .. })
    }
}

/// Finds the first `@deprecated` annotation within `directives`,
/// if any.
///
/// Useful for pointing error spans at the `@deprecated` annotation
/// itself rather than at the item it is applied to.
///
/// See
/// [@deprecated](https://spec.graphql.org/September2025/#sec--deprecated).
pub(crate) fn find_deprecated_annotation(
    directives: &[DirectiveAnnotation],
) -> Option<&DirectiveAnnotation> {
    directives.iter().find(|annot| annot.name().as_str() == "deprecated")
}
