use crate::loc;
use crate::types::DirectiveAnnotation;
use crate::types::Field;
use crate::types::NamedGraphQLTypeRef;
use std::collections::BTreeMap;

/// Information associated with [GraphQLType::Object]
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectType {
    pub(super) def_location: loc::FilePosition,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) fields: BTreeMap<String, Field>,
    pub(super) interfaces: Vec<NamedGraphQLTypeRef>,
    pub(super) name: String,
}

impl ObjectType {
    pub fn def_location(&self) -> &loc::FilePosition {
        &self.def_location
    }

    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    pub fn fields(&self) -> &BTreeMap<String, Field> {
        &self.fields
    }

    pub fn interfaces(&self) -> &Vec<NamedGraphQLTypeRef> {
        &self.interfaces
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
