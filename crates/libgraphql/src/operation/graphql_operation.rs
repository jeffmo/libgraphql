use crate::loc;
use crate::operation::Query;
use crate::operation::Mutation;
use crate::operation::Subscription;

#[derive(Debug, PartialEq)]
pub enum GraphQLOperation<'schema, 'fragreg> {
    Query(Box<Query<'schema, 'fragreg>>),
    Mutation(Box<Mutation<'schema, 'fragreg>>),
    Subscription(Box<Subscription<'schema, 'fragreg>>),
}
impl<'schema, 'fragreg> GraphQLOperation<'schema, 'fragreg> {
    pub fn as_query(&self) -> Option<&Query<'schema, 'fragreg>> {
        if let Self::Query(op) = self {
            Some(op)
        } else {
            None
        }
    }

    pub fn as_mutation(&self) -> Option<&Mutation<'schema, 'fragreg>> {
        if let Self::Mutation(op) = self {
            Some(op)
        } else {
            None
        }
    }

    pub fn as_subscription(&self) -> Option<&Subscription<'schema, 'fragreg>> {
        if let Self::Subscription(op) = self {
            Some(op)
        } else {
            None
        }
    }

    pub fn def_location(&self) -> &loc::SourceLocation {
        match self {
            Self::Query(query) => query.def_location(),
            Self::Mutation(mutation) => mutation.def_location(),
            Self::Subscription(subscription) => subscription.def_location(),
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Query(query) => query.name(),
            Self::Mutation(mutation) => mutation.name(),
            Self::Subscription(subscription) => subscription.name(),
        }
    }
}
