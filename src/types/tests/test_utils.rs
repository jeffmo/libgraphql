use crate::ast;

pub(crate) fn parse_enum_type_def(
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

pub(crate) fn parse_enum_type_ext(
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

pub(crate) fn parse_object_type_def(
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
