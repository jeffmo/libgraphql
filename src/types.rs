use crate::ast;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Directive {
    Custom {
        def_ast: ast::schema::DirectiveDefinition,
        def_location: ast::FileLocation,
        // TODO: parameters
    },
    Deprecated,
    Include,
    Skip,
    SpecifiedBy,
}
impl Directive {
    pub fn name(&self) -> &str {
        match self {
            Directive::Custom { def_ast, .. } => def_ast.name.as_str(),
            Directive::Deprecated => "deprecated",
            Directive::Include => "include",
            Directive::Skip => "skip",
            Directive::SpecifiedBy => "specifiedBy",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DirectiveReference {
    // TODO: arguments
    pub directive_name: String,
    pub location: ast::FileLocation,
}
impl DirectiveReference {
    pub fn from_ast(
        file_path: &PathBuf,
        ast: &ast::query::Directive,
    ) -> Self {
        DirectiveReference {
            directive_name: ast.name.to_string(),
            location: ast::FileLocation::from_pos(
                file_path.to_path_buf(),
                ast.position,
            ),
        }
    }
}
#[derive(Clone, Debug)]
pub struct EnumValue {
    pub def_ast: ast::schema::EnumValue,
    pub def_location: ast::FileLocation,
}

#[derive(Clone, Debug)]
pub struct FieldType {
    pub def_ast: ast::schema::Field,
    pub def_location: ast::FileLocation,
}

#[derive(Clone, Debug)]
pub struct InputFieldType {
    pub def_ast: ast::schema::InputValue,
    pub def_location: ast::FileLocation,
}

#[derive(Clone, Debug)]
pub enum SchemaType {
    Enum {
        def_ast: ast::schema::EnumType,
        def_location: ast::FileLocation,
        directives: Vec<DirectiveReference>,
        values: HashMap<String, EnumValue>,
    },

    InputObject {
        def_ast: ast::schema::InputObjectType,
        def_location: ast::FileLocation,
        directives: Vec<DirectiveReference>,
        fields: HashMap<String, InputFieldType>,
    },

    Interface {
        def_ast: ast::schema::InterfaceType,
        def_location: ast::FileLocation,
        directives: Vec<DirectiveReference>,
        fields: HashMap<String, FieldType>,
    },

    Object {
        def_ast: ast::schema::ObjectType,
        def_location: ast::FileLocation,
        directives: Vec<DirectiveReference>,
        fields: HashMap<String, FieldType>,
    },

    Scalar {
        def_ast: ast::schema::ScalarType,
        def_location: ast::FileLocation,
        directives: Vec<DirectiveReference>,
    },

    Union {
        def_ast: ast::schema::UnionType,
        def_location: ast::FileLocation,
        directives: Vec<DirectiveReference>,
        types: HashMap<String, SchemaTypeReference>
    }
}
impl SchemaType {
    pub fn get_def_location(&self) -> &ast::FileLocation {
        match self {
            SchemaType::Enum { def_location, .. } => def_location,
            SchemaType::InputObject { def_location, .. } => def_location,
            SchemaType::Interface { def_location, .. } => def_location,
            SchemaType::Object { def_location, .. } => def_location,
            SchemaType::Scalar { def_location, .. } => def_location,
            SchemaType::Union { def_location, .. } => def_location,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SchemaTypeReference {
    pub type_name: String,
    pub location: ast::FileLocation,
}
