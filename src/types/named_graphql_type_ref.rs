
use crate::loc;
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::named_ref::NamedRef;

pub type NamedGraphQLTypeRef = NamedRef<
    /* TSource = */ Schema,
    /* TRefLocation = */ loc::SourceLocation,
    /* TResource = */ GraphQLType,
>;
