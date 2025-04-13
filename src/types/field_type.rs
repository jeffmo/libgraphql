use crate::Schema;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeRef;

#[derive(Clone, Debug, PartialEq)]
pub struct FieldType {
    nullable: bool,
    type_ref: GraphQLTypeRef,
}
impl FieldType {
    pub fn graphql_type(&self, schema: &Schema) -> &GraphQLTypeRef {
        &self.type_ref
    }
}
