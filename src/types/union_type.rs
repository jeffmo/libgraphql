use crate::DirectiveAnnotation;
use crate::loc;
use crate::types::TypeAnnotation;
use std::collections::HashMap;

/// Represents a
/// [union type](https://spec.graphql.org/October2021/#sec-Unions) defined
/// within some [`Schema`](crate::Schema).
#[derive(Clone, Debug, PartialEq)]
pub struct UnionType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<DirectiveAnnotation>,
    pub name: String,
    pub members: HashMap<String, TypeAnnotation>,
}
