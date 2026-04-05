use std::borrow::Borrow;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;

/// Constrains all GraphQL name newtypes to a consistent set of
/// capabilities. Every name type must be cloneable, hashable,
/// orderable, serializable, displayable, convertible from
/// `String`, and constructible via `new` from any input that
/// can be converted into a `String`.
///
/// This trait is `pub(crate)` — it enforces consistency at
/// definition time but is not part of the public API. Public
/// consumers interact with each name type's inherent methods
/// (delegated via `#[inherent]`).
pub(crate) trait GraphQLName:
    Clone
    + Debug
    + Display
    + Eq
    + Hash
    + Ord
    + AsRef<str>
    + Borrow<str>
    + From<String>
    + serde::Serialize
    + for<'de> serde::Deserialize<'de>
{
    fn new(s: impl Into<String>) -> Self;
    fn as_str(&self) -> &str;
}
