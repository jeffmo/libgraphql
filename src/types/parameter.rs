use crate::loc;
use crate::types::GraphQLTypeRef;
use crate::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub(super) def_location: loc::FilePosition,
    pub(super) default_value: Option<Value>,
    pub(super) name: String,
    pub(super) type_ref: GraphQLTypeRef,
}
impl Parameter {
    pub fn def_location(&self) -> &loc::FilePosition {
        &self.def_location
    }

    pub fn default_value(&self) -> &Option<Value> {
        &self.default_value
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn type_ref(&self) -> &GraphQLTypeRef {
        &self.type_ref
    }
}
