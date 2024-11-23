use crate::ast;
use std::collections::HashMap;

#[derive(Debug)]
pub struct FieldType {
    pub def_ast: ast::schema::Field,
    pub def_location: ast::FileLocation,
}

#[derive(Debug)]
pub struct InputFieldType {
    pub def_ast: ast::schema::InputValue,
    pub def_location: ast::FileLocation,
}

#[derive(Debug)]
pub enum SchemaType {
    Enum {
        def_ast: ast::schema::EnumType,
        def_location: ast::FileLocation,
    },

    InputObject {
        def_ast: ast::schema::InputObjectType,
        def_location: ast::FileLocation,
        fields: HashMap<String, InputFieldType>,
    },

    Interface {
        def_ast: ast::schema::InterfaceType,
        def_location: ast::FileLocation,
        fields: HashMap<String, FieldType>,
    },

    Object {
        def_ast: ast::schema::ObjectType,
        def_location: ast::FileLocation,
        fields: HashMap<String, FieldType>,
    },

    Scalar {
        def_ast: ast::schema::ScalarType,
        def_location: ast::FileLocation,
    },
}
impl SchemaType {
    pub fn get_def_location(&self) -> &ast::FileLocation {
        match self {
            SchemaType::Enum { def_location, .. } => def_location,
            SchemaType::InputObject { def_location, .. } => def_location,
            SchemaType::Interface { def_location, .. } => def_location,
            SchemaType::Object { def_location, .. } => def_location,
            SchemaType::Scalar { def_location, .. } => def_location,
        }
    }
}

