use crate::ast;
use crate::types::EnumType;
use crate::types::InputObjectType;
use crate::types::InterfaceType;
use crate::types::ObjectType;
use crate::types::TypesMapBuilder;

pub(super) fn get_enum_type(
    types_map_builder: &mut TypesMapBuilder,
    type_name: &str,
) -> EnumType {
    types_map_builder.get_type_mut(type_name)
        .expect("Type was created")
        .as_enum()
        .expect("Type was an enum")
        .to_owned()
}

pub(super) fn get_input_object_type(
    types_map_builder: &mut TypesMapBuilder,
    input_obj_name: &str,
) -> InputObjectType {
    types_map_builder.get_type_mut(input_obj_name)
        .expect("Type was created")
        .as_input_object()
        .expect("Type was input object")
        .to_owned()
}

pub(super) fn get_interface_type(
    types_map_builder: &mut TypesMapBuilder,
    type_name: &str,
) -> InterfaceType {
    types_map_builder.get_type_mut(type_name)
        .expect("Type was created")
        .as_interface()
        .expect("Type was interface")
        .to_owned()
}

pub(super) fn get_object_type(
    types_map_builder: &mut TypesMapBuilder,
    enum_name: &str,
) -> ObjectType {
    types_map_builder.get_type_mut(enum_name)
        .expect("Type was created")
        .as_object()
        .expect("Type was object")
        .to_owned()
}

pub(super) fn parse_enum_type_def<'a>(
    type_name: &'a str,
    schema: &'a str,
) -> Option<(ast::EnumTypeDefinition<'a>, ast::SourceMap<'a>)> {
    let parse_result = ast::parse_schema(schema);
    let (ast_doc, source_map) = parse_result.into_valid()
        .expect("parse error");
    for def in &ast_doc.definitions {
        match def {
            ast::Definition::TypeDefinition(
                ast::TypeDefinition::Enum(enum_type)
            ) if enum_type.name.value == type_name => {
                return Some((enum_type.clone(), source_map));
            },

            _ => continue,
        }
    }
    None
}

pub(super) fn parse_enum_type_ext<'a>(
    type_name: &'a str,
    schema: &'a str,
) -> Option<(ast::EnumTypeExtension<'a>, ast::SourceMap<'a>)> {
    let parse_result = ast::parse_schema(schema);
    let (ast_doc, source_map) = parse_result.into_valid()
        .expect("parse error");
    for def in &ast_doc.definitions {
        match def {
            ast::Definition::TypeExtension(
                ast::TypeExtension::Enum(enum_ext)
            ) if enum_ext.name.value == type_name => {
                return Some((enum_ext.clone(), source_map));
            },

            _ => continue,
        }
    }
    None
}

pub(super) fn parse_input_object_type_def<'a>(
    type_name: &'a str,
    schema: &'a str,
) -> Option<(ast::InputObjectTypeDefinition<'a>, ast::SourceMap<'a>)> {
    let parse_result = ast::parse_schema(schema);
    let (ast_doc, source_map) = parse_result.into_valid()
        .expect("parse error");
    for def in &ast_doc.definitions {
        match def {
            ast::Definition::TypeDefinition(
                ast::TypeDefinition::InputObject(input_obj_type)
            ) if input_obj_type.name.value == type_name => {
                return Some((input_obj_type.clone(), source_map));
            },

            _ => continue,
        }
    }
    None
}

pub(super) fn parse_interface_type_def<'a>(
    type_name: &'a str,
    schema: &'a str,
) -> Option<(ast::InterfaceTypeDefinition<'a>, ast::SourceMap<'a>)> {
    let parse_result = ast::parse_schema(schema);
    let (ast_doc, source_map) = parse_result.into_valid()
        .expect("parse error");
    for def in &ast_doc.definitions {
        match def {
            ast::Definition::TypeDefinition(
                ast::TypeDefinition::Interface(iface_type)
            ) if iface_type.name.value == type_name => {
                return Some((iface_type.clone(), source_map));
            },

            _ => continue,
        }
    }
    None
}

pub(super) fn parse_interface_type_ext<'a>(
    type_name: &'a str,
    schema: &'a str,
) -> Option<(ast::InterfaceTypeExtension<'a>, ast::SourceMap<'a>)> {
    let parse_result = ast::parse_schema(schema);
    let (ast_doc, source_map) = parse_result.into_valid()
        .expect("parse error");
    for def in &ast_doc.definitions {
        match def {
            ast::Definition::TypeExtension(
                ast::TypeExtension::Interface(iface_ext)
            ) if iface_ext.name.value == type_name => {
                return Some((iface_ext.clone(), source_map));
            },

            _ => continue,
        }
    }
    None
}

pub(super) fn parse_object_type_def<'a>(
    type_name: &'a str,
    schema: &'a str,
) -> Option<(ast::ObjectTypeDefinition<'a>, ast::SourceMap<'a>)> {
    let parse_result = ast::parse_schema(schema);
    let (ast_doc, source_map) = parse_result.into_valid()
        .expect("parse error");
    for def in &ast_doc.definitions {
        match def {
            ast::Definition::TypeDefinition(
                ast::TypeDefinition::Object(obj_type)
            ) if obj_type.name.value == type_name => {
                return Some((obj_type.clone(), source_map));
            },

            _ => continue,
        }
    }
    None
}

pub(super) fn parse_object_type_ext<'a>(
    type_name: &'a str,
    schema: &'a str,
) -> Option<(ast::ObjectTypeExtension<'a>, ast::SourceMap<'a>)> {
    let parse_result = ast::parse_schema(schema);
    let (ast_doc, source_map) = parse_result.into_valid()
        .expect("parse error");
    for def in &ast_doc.definitions {
        match def {
            ast::Definition::TypeExtension(
                ast::TypeExtension::Object(object_ext)
            ) if object_ext.name.value == type_name => {
                return Some((object_ext.clone(), source_map));
            },

            _ => continue,
        }
    }
    None
}
