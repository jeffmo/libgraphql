use crate::operation::Mutation;
use crate::operation::Query;
use crate::operation::Subscription;

#[derive(Debug)]
pub enum Operation<'schema> {
    Mutation(Mutation<'schema>),
    Query(Query<'schema>),
    Subscription(Subscription<'schema>),
}
