use crate::operation::Query;
use crate::operation::Mutation;
use crate::operation::Subscription;
use std::boxed::Box;

#[derive(Clone, Debug, PartialEq)]
pub enum Operation<'schema: 'fragreg, 'fragreg> {
    Query(Box<Query<'schema, 'fragreg>>),
    Mutation(Box<Mutation<'schema, 'fragreg>>),
    Subscription(Box<Subscription<'schema, 'fragreg>>),
}
