use crate::loc;
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::types::TypeAnnotation;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct ListTypeAnnotation {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) inner_type_ref: Box<TypeAnnotation>,
    pub(super) nullable: bool,
}
impl ListTypeAnnotation {
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    pub fn inner_type_annotation(&self) -> &TypeAnnotation {
        &self.inner_type_ref
    }

    pub fn is_subtype_of(
        &self,
        schema: &Schema,
        other: &Self,
    ) -> bool {
        self.is_subtype_of_impl(&schema.types, other)
    }

    pub(super) fn is_subtype_of_impl(
        &self,
        types_map: &HashMap<String, GraphQLType>,
        other: &Self,
    ) -> bool {
        self.inner_type_ref.is_subtype_of_impl(types_map, &other.inner_type_ref)
    }

    pub fn nullable(&self) -> bool {
        self.nullable
    }
}
