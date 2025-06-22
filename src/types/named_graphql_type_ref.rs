
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::named_ref::NamedRef;

pub type NamedGraphQLTypeRef = NamedRef<Schema, GraphQLType>;
