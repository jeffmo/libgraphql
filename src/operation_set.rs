use crate::operation::Mutation;
use crate::operation::Query;
use crate::operation::Subscription;

#[derive(Debug)]
pub struct OperationSet<'schema> {
    pub(crate) mutations: Vec<Mutation<'schema>>,
    pub(crate) queries: Vec<Query<'schema>>,
    pub(crate) subscriptions: Vec<Subscription<'schema>>,
}
