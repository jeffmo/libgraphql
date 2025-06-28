use crate::operation::FragmentSet;
use crate::operation::Mutation;
use crate::operation::Query;
use crate::operation::Subscription;

#[derive(Debug)]
pub struct OperationSet<'schema, 'fragset: 'schema> {
    pub(crate) fragment_set: Option<FragmentSet<'schema>>,
    pub(crate) mutations: Vec<Mutation<'schema, 'fragset>>,
    pub(crate) queries: Vec<Query<'schema, 'fragset>>,
    pub(crate) subscriptions: Vec<Subscription<'schema, 'fragset>>,
}
