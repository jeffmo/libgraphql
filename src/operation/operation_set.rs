use crate::operation::FragmentSet;
use crate::operation::Mutation;
use crate::operation::OperationSetBuilder;
use crate::operation::Query;
use crate::operation::Subscription;
use crate::schema::Schema;
use std::collections::HashMap;

/// Represets a set of fully typechecked and immutable GraphQL operations
/// (e.g. [`Query`]s, [`Mutation`]s, [`NamedFragment`]s, etc).
#[derive(Debug)]
pub struct OperationSet<'schema, 'fragset: 'schema> {
    pub(crate) fragment_set: Option<FragmentSet<'schema>>,
    pub(crate) named_mutations: HashMap<String, Mutation<'schema, 'fragset>>,
    pub(crate) named_queries: HashMap<String, Query<'schema, 'fragset>>,
    pub(crate) named_subscriptions: HashMap<String, Subscription<'schema, 'fragset>>,
    pub(crate) schema: &'schema Schema,
}
impl<'schema, 'fragset: 'schema> OperationSet<'schema, 'fragset> {
    /// Helper function that just delegates to [`SchemaBuilder::new()`]
    pub fn builder(schema: &'schema Schema) -> OperationSetBuilder<'schema, 'fragset> {
        OperationSetBuilder::new(schema)
    }

    /// Looks up a [`Mutation`] operation by name.
    pub fn lookup_mutation(
        &self,
        mutation_name: &str,
    ) -> Option<&Mutation<'schema, 'fragset>> {
        self.mutations.get(mutation_name)
    }

    /// Looks up a [`Query`] operation by name.
    pub fn lookup_query(
        &self,
        query_name: &str,
    ) -> Option<&Query<'schema, 'fragset>> {
        self.queries.get(query_name)
    }

    /// Looks up a [`Subscription`] operation by name.
    pub fn lookup_subscription(
        &self,
        subscription_name: &str,
    ) -> Option<&Subscription<'schema, 'fragset>> {
        self.subscriptions.get(subscription_name)
    }
}
