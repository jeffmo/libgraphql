use crate::DirectiveAnnotation;
use crate::loc;
use crate::types::InputField;
use std::collections::BTreeMap;

/// Represents an
/// [input object type](https://spec.graphql.org/October2021/#sec-Input-Objects)
/// defined within some [`Schema`](crate::schema::Schema).
#[derive(Clone, Debug, PartialEq)]
pub struct InputObjectType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<DirectiveAnnotation>,
    pub fields: BTreeMap<String, InputField>,
    pub name: String,
}

