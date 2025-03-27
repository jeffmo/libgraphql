use crate::loc;
use crate::types::DirectiveAnnotation;
use crate::types::Field;
use std::collections::BTreeMap;

/// Information associated with [GraphQLType::Object]
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<DirectiveAnnotation>,
    pub fields: BTreeMap<String, Field>,
    pub name: String,
}

