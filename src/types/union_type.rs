use crate::loc;
use crate::types::DirectiveAnnotation;
use crate::types::GraphQLTypeRef;
use std::collections::HashMap;

/// Information associated with [GraphQLType::Union]
#[derive(Clone, Debug, PartialEq)]
pub struct UnionType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<DirectiveAnnotation>,
    pub name: String,
    pub members: HashMap<String, GraphQLTypeRef>,
}
