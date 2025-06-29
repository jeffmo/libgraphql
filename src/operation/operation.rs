use crate::operation::Query;
use crate::operation::Mutation;
use crate::operation::Subscription;
use std::boxed::Box;

#[derive(Debug, PartialEq)]
pub enum Operation<'schema, 'fragset> {
    Query(Box<Query<'schema, 'fragset>>),
    Mutation(Box<Mutation<'schema, 'fragset>>),
    Subscription(Box<Subscription<'schema, 'fragset>>),
}
