use crate::loc;
use crate::types::DirectiveAnnotation;
use crate::types::Field;
use std::collections::HashMap;

/// Information associated with [GraphQLType::Object]
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<DirectiveAnnotation>,
    pub fields: HashMap<String, Field>,
    pub name: String,
}

