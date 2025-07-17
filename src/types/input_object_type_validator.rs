use crate::schema::TypeValidationError;
use crate::types::GraphQLType;
use crate::types::InputObjectType;
use std::collections::HashMap;

pub(super) struct InputObjectTypeValidator<'a> {
    errors: Vec<TypeValidationError>,
    type_: &'a InputObjectType,
    types_map: &'a HashMap<String, GraphQLType>,
}
impl<'a> InputObjectTypeValidator<'a> {
    pub fn new(
        type_: &'a InputObjectType,
        types_map: &'a HashMap<String, GraphQLType>,
    ) -> Self {
        Self {
            errors: vec![],
            type_,
            types_map,
        }
    }

    pub fn validate(self) -> Vec<TypeValidationError> {
        // TODO: Input type fields must:
        //
        //       1) Only be declared with an input type (enum, scalar,
        //          inputobject, etc)
        //       2) Use a nullable type annotation to break infinite recursion
        //          where the input type references itself or a type that
        //          [recursively] references itself.
        self.errors
    }
}
