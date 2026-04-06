use crate::schema::schema_build_error::SchemaBuildError;

/// A collection of errors from
/// [`SchemaBuilder::build()`](crate::schema::SchemaBuilder::build).
///
/// Implements [`std::error::Error`] and
/// [`Display`](std::fmt::Display) for `?` propagation.
/// Implements [`IntoIterator`] for access to individual errors.
///
/// This type is never empty — construction via `new()` requires
/// at least one error (enforced by `debug_assert`).
#[derive(Debug, PartialEq)]
pub struct SchemaErrors {
    errors: Vec<SchemaBuildError>,
}

impl SchemaErrors {
    pub(crate) fn new(errors: Vec<SchemaBuildError>) -> Self {
        debug_assert!(!errors.is_empty());
        Self { errors }
    }

    pub fn errors(&self) -> &[SchemaBuildError] { &self.errors }

    // SchemaErrors is guaranteed non-empty (enforced by
    // debug_assert in new()), so is_empty() is not provided.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize { self.errors.len() }
}

impl std::fmt::Display for SchemaErrors {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for (i, error) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for SchemaErrors {}

impl IntoIterator for SchemaErrors {
    type Item = SchemaBuildError;
    type IntoIter = std::vec::IntoIter<SchemaBuildError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a> IntoIterator for &'a SchemaErrors {
    type Item = &'a SchemaBuildError;
    type IntoIter = std::slice::Iter<'a, SchemaBuildError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.iter()
    }
}
