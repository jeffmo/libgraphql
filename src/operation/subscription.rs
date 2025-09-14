use crate::ast;
use crate::operation::FragmentRegistry;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::OperationTrait;
use crate::operation::OperationData;
use crate::operation::SelectionSet;
use crate::operation::SubscriptionBuilder;
use crate::operation::SubscriptionBuildError;
use crate::operation::Variable;
use crate::schema::Schema;
use indexmap::IndexMap;
use inherent::inherent;

#[derive(Clone, Debug, PartialEq)]
pub struct Subscription<'schema: 'fragreg, 'fragreg>(
    pub(super) OperationData<'schema, 'fragreg>,
);

#[inherent]
impl<'schema: 'fragreg, 'fragreg> OperationTrait<
    'schema,
    'fragreg,
    ast::operation::Subscription,
    SubscriptionBuildError,
    SubscriptionBuilder<'schema, 'fragreg>,
> for Subscription<'schema, 'fragreg> {
    /// Convenience wrapper around [`SubscriptionBuilder::new()`].
    pub fn builder(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
    ) -> SubscriptionBuilder<'schema, 'fragreg> {
        SubscriptionBuilder::new(schema, fragment_registry)
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`Subscription`].
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.0.directives
    }

    /// The [`loc::SourceLocation`] indicating where this [`Subscription`]
    /// operation was defined.
    pub fn def_location(&self) -> &loc::SourceLocation {
        &self.0.def_location
    }

    /// Access the name of this [`Subscription`] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    /// Access the [`SelectionSet`] defined for this [`Subscription`].
    pub fn selection_set(&self) -> &SelectionSet<'fragreg> {
        &self.0.selection_set
    }

    /// Access the [`Variable`]s defined on this [`Subscription`].
    pub fn variables(&self) -> &IndexMap<String, Variable> {
        &self.0.variables
    }
}
