use indexmap::IndexMap;

use crate::loc;
use crate::operation::Query;
use crate::operation::Mutation;
use crate::operation::SelectionSet;
use crate::operation::Subscription;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::DirectiveAnnotation;
use std::boxed::Box;

#[derive(Clone, Debug, PartialEq)]
pub enum Operation<'schema: 'fragreg, 'fragreg> {
    Query(Box<Query<'schema, 'fragreg>>),
    Mutation(Box<Mutation<'schema, 'fragreg>>),
    Subscription(Box<Subscription<'schema, 'fragreg>>),
}
impl<'schema: 'fragreg, 'fragreg> Operation<'schema, 'fragreg> {
    pub fn def_location(&self) -> &loc::SourceLocation {
        match self {
            Self::Mutation(op) => op.def_location(),
            Self::Query(op) => op.def_location(),
            Self::Subscription(op) => op.def_location(),
        }
    }

    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        match self {
            Self::Mutation(op) => op.directives(),
            Self::Query(op) => op.directives(),
            Self::Subscription(op) => op.directives(),
        }
    }

    pub fn is_mutation(&self) -> bool {
        matches!(self, Self::Mutation(_))
    }

    pub fn is_query(&self) -> bool {
        matches!(self, Self::Query(_))
    }

    pub fn is_subscription(&self) -> bool {
        matches!(self, Self::Subscription(_))
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Mutation(op) => op.name(),
            Self::Query(op) => op.name(),
            Self::Subscription(op) => op.name(),
        }
    }

    pub fn root_graphql_type(&self, schema: &'schema Schema) -> &GraphQLType {
        match self {
            Self::Mutation(op) => op.root_graphql_type(schema),
            Self::Query(op) => op.root_graphql_type(schema),
            Self::Subscription(op) => op.root_graphql_type(schema),
        }
    }

    pub fn selection_set(&self) -> &SelectionSet<'fragreg> {
        match self {
            Self::Mutation(op) => op.selection_set(),
            Self::Query(op) => op.selection_set(),
            Self::Subscription(op) => op.selection_set(),
        }
    }

    pub fn variables(&self) -> &IndexMap<String, Variable> {
        match self {
            Self::Mutation(op) => op.variables(),
            Self::Query(op) => op.variables(),
            Self::Subscription(op) => op.variables(),
        }
    }
}
