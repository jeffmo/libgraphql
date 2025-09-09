use crate::loc;
use crate::schema::Schema;
use crate::Value;
use crate::types::Directive;
use crate::types::NamedDirectiveRef;
use indexmap::IndexMap;

/// Represents a
/// [directive annotation](https://spec.graphql.org/October2021/#sec-Language.Directives)
/// placed somewhere within a [`GraphQLType`](crate::types::GraphQLType),
/// [`Mutation`](crate::operation::Mutation),
/// [`Query`](crate::operation::Query), or
/// [`Subscription`](crate::operation::Subscription).
///
/// A [`DirectiveAnnotation`] can be thought of as a "pointer" to some
/// [`Directive`] paired with a set of named arguments ([`Value`]s).
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveAnnotation {
    pub(crate) args: IndexMap<String, Value>,
    pub(crate) directive_ref: NamedDirectiveRef,
}
impl DirectiveAnnotation {
    /// A map from ParameterName -> [`Value`] for all arguments passed to this
    /// [`DirectiveAnnotation`].
    ///
    /// This returns an [`IndexMap`] to guarantee that map entries retain the same
    /// ordering as the order of arguments passed to this directive annotation.
    pub fn args(&self) -> &IndexMap<String, Value> {
        &self.args
    }

    /// The [`SourceLocation`](loc::SourceLocation) indicating where this
    /// annotation was specified within some
    /// [`GraphQLType`](crate::types::GraphQLType),
    /// [`Mutation`](crate::operation::Mutation),
    /// [`Query`](crate::operation::Query),
    /// or [`Subscription`](crate::operation::Subscription).
    pub fn def_location(&self) -> &loc::SourceLocation {
        self.directive_ref.ref_location()
    }

    /// The [`Directive`] type for which this annotation refers to.
    pub fn directive_type<'schema>(
        &self,
        schema: &'schema Schema,
    ) -> &'schema Directive {
        self.directive_ref.deref(schema).unwrap()
    }

    /// The name of the [`Directive`] type for which this annotation refers to.
    ///
    /// This can be useful when the [`Schema`] object is unavailable or
    /// inconvenient to access but the type's name is all that's needed.
    pub fn directive_type_name(&self) -> &str {
        self.directive_ref.name()
    }
}
