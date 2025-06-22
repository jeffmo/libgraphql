use crate::loc;
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;

#[derive(Clone, Debug, PartialEq)]
pub struct NamedTypeAnnotation {
    pub(super) nullable: bool,
    pub(super) type_ref: NamedGraphQLTypeRef,
}
impl NamedTypeAnnotation {
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        self.type_ref.def_location()
    }

    pub fn graphql_type<'schema>(
        &self,
        schema: &'schema Schema,
    ) -> &'schema GraphQLType {
        self.type_ref.deref(schema).unwrap()
    }

    pub fn graphql_type_name(&self) -> &str {
        self.type_ref.name.as_str()
    }

    pub fn nullable(&self) -> bool {
        self.nullable
    }
}
