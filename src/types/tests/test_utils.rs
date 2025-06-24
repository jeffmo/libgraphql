use crate::ast;
use crate::types::EnumType;
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

pub(super) fn parse_enum_type_def(
    type_name: &str,
    schema: &str,
) -> Result<Option<ast::schema::EnumType>, ast::schema::ParseError> {
    let doc = ast::schema::parse(schema)?;
    for def in doc.definitions {
        match &def {
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Enum(
                    enum_type @ ast::schema::EnumType {
                        name: enum_name,
                        ..
                    }
                )
            ) if enum_name == type_name => {
                return Ok(Some(enum_type.to_owned()));
            }

            _ => continue,
        }
    }
    Ok(None)
}

pub(super) fn parse_enum_type_ext(
    type_name: &str,
    schema: &str,
) -> Result<Option<ast::schema::EnumTypeExtension>, ast::schema::ParseError> {
    let doc = ast::schema::parse(schema)?;
    for def in doc.definitions {
        match &def {
            ast::schema::Definition::TypeExtension(
                ast::schema::TypeExtension::Enum(
                    enum_ext @ ast::schema::EnumTypeExtension {
                        name: enum_name,
                        ..
                    }
                )
            ) if enum_name == type_name => {
                return Ok(Some(enum_ext.to_owned()));
            }

            _ => continue,
        }
    }
    Ok(None)
}

pub(super) fn parse_interface_type_def(
    type_name: &str,
    schema: &str,
) -> Result<Option<ast::schema::InterfaceType>, ast::schema::ParseError> {
    let doc = ast::schema::parse(schema)?;
    for def in doc.definitions {
        match &def {
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Interface(
                    iface_type @ ast::schema::InterfaceType {
                        name: ifacet_name,
                        ..
                    }
                )
            ) if ifacet_name == type_name => {
                return Ok(Some(iface_type.to_owned()));
            }

            _ => continue,
        }
    }
    Ok(None)
}

pub(super) fn parse_interface_type_ext(
    type_name: &str,
    schema: &str,
) -> Result<Option<ast::schema::InterfaceTypeExtension>, ast::schema::ParseError> {
    let doc = ast::schema::parse(schema)?;
    for def in doc.definitions {
        match &def {
            ast::schema::Definition::TypeExtension(
                ast::schema::TypeExtension::Interface(
                    iface_ext @ ast::schema::InterfaceTypeExtension {
                        name: iface_name,
                        ..
                    }
                )
            ) if iface_name == type_name => {
                return Ok(Some(iface_ext.to_owned()));
            }

            _ => continue,
        }
    }
    Ok(None)
}

pub(super) fn parse_object_type_def(
    type_name: &str,
    schema: &str,
) -> Result<Option<ast::schema::ObjectType>, ast::schema::ParseError> {
    let doc = ast::schema::parse(schema)?;
    for def in doc.definitions {
        match &def {
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Object(
                    obj_type @ ast::schema::ObjectType {
                        name: objt_name,
                        ..
                    }
                )
            ) if objt_name == type_name => {
                return Ok(Some(obj_type.to_owned()));
            }

            _ => continue,
        }
    }
    Ok(None)
}

pub(super) fn parse_object_type_ext(
    type_name: &str,
    schema: &str,
) -> Result<Option<ast::schema::ObjectTypeExtension>, ast::schema::ParseError> {
    let doc = ast::schema::parse(schema)?;
    for def in doc.definitions {
        match &def {
            ast::schema::Definition::TypeExtension(
                ast::schema::TypeExtension::Object(
                    object_ext @ ast::schema::ObjectTypeExtension {
                        name: object_name,
                        ..
                    }
                )
            ) if object_name == type_name => {
                return Ok(Some(object_ext.to_owned()));
            }

            _ => continue,
        }
    }
    Ok(None)
}
