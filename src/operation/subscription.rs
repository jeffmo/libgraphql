use crate::ast;
use crate::operation::FragmentSet;
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
pub struct Subscription<'schema: 'fragset, 'fragset>(
    pub(super) OperationData<'schema, 'fragset>,
);

#[inherent]
impl<'schema, 'fragset> OperationTrait<
    'schema,
    'fragset,
    ast::operation::Subscription,
    SubscriptionBuildError,
    SubscriptionBuilder<'schema, 'fragset>,
> for Subscription<'schema, 'fragset> {
    /// Convenience wrapper around [`SubscriptionBuilder::new()`].
    pub fn builder(
        schema: &'schema Schema,
        fragset: Option<&'fragset FragmentSet<'schema>>,
    ) -> SubscriptionBuilder<'schema, 'fragset> {
        SubscriptionBuilder::new(schema, fragset)
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`Subscription`].
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.0.directives
    }

    /// The [`DefLocation`](loc::FilePosition) indicating where this
    /// [`Subscription`] was defined.
    pub fn def_location(&self) -> Option<&loc::FilePosition> {
        self.0.def_location.as_ref()
    }

    /// Access the name of this [`Subscription`] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    /// Access the [`SelectionSet`] defined for this [`Subscription`].
    pub fn selection_set(&self) -> &SelectionSet<'fragset> {
        &self.0.selection_set
    }

    /// Access the [`Variable`]s defined on this [`Subscription`].
    pub fn variables(&self) -> &IndexMap<String, Variable> {
        &self.0.variables
    }
}
