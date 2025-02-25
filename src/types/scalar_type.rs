use crate::loc;
use crate::types::DirectiveAnnotation;

/// Information associated with [GraphQLType::Scalar]
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<DirectiveAnnotation>,
    pub name: String,
}

