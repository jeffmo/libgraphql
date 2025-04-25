use crate::loc;
use crate::types::TypeAnnotation;

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

    pub fn nullable(&self) -> bool {
        self.nullable
    }
}
