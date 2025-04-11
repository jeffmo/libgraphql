use crate::operation::Mutation;
use crate::operation::NamedFragment;
use crate::operation::Query;
use crate::operation::Subscription;
use std::collections::HashMap;

#[derive(Debug)]
pub struct OperationSet<'schema> {
    pub(crate) fragments: HashMap<String, NamedFragment<'schema>>,
    pub(crate) mutations: Vec<Mutation<'schema>>,
    pub(crate) queries: Vec<Query<'schema>>,
    pub(crate) subscriptions: Vec<Subscription<'schema>>,
}
