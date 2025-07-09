use crate::ast;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::Field;
use crate::types::GraphQLType;
use crate::types::TypeAnnotation;
use crate::types::InterfaceType;
use crate::types::NamedGraphQLTypeRef;
use crate::types::ObjectOrInterfaceTypeData;
use crate::types::Parameter;
use crate::types::TypeBuilder;
use crate::types::TypeBuilderHelpers;
use crate::types::TypesMapBuilder;
use inherent::inherent;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[derive(Debug)]
pub(crate) struct InterfaceTypeBuilder {
    extensions: Vec<(PathBuf, ast::schema::InterfaceTypeExtension)>,
}

impl InterfaceTypeBuilder {
    pub fn new() -> Self {
        Self {
            extensions: vec![],
        }
    }

    fn merge_type_extension(
        &mut self,
        iface_type: &mut InterfaceType,
        ext_file_path: &Path,
        ext: ast::schema::InterfaceTypeExtension,
    ) -> Result<()> {
        iface_type.0.directives.append(&mut TypeBuilderHelpers::directive_refs_from_ast(
            ext_file_path,
            &ext.directives,
        ));

        for ext_field in ext.fields.iter() {
            let ext_field_pos = loc::FilePosition::from_pos(
                ext_file_path,
                ext_field.position,
            );
            let ext_field_loc = loc::SchemaDefLocation::Schema(
                ext_field_pos.clone()
            );

            // Error if this field is already defined.
            if let Some(existing_field) = iface_type.0.fields.get(ext_field.name.as_str()) {
                return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                    type_name: ext.name.to_string(),
                    field_name: ext_field.name.to_string(),
                    field_def1: existing_field.def_location().clone(),
                    field_def2: ext_field_loc,
                })?;
            }
            iface_type.0.fields.insert(ext_field.name.to_string(), Field {
                def_location: ext_field_loc.clone(),
                directives: TypeBuilderHelpers::directive_refs_from_ast(
                    ext_file_path,
                    &ext_field.directives,
                ),
                name: ext_field.name.to_string(),
                parameters: ext_field.arguments.iter().map(|input_val| (
                    input_val.name.to_string(),
                    Parameter::from_ast(
                        ext_file_path,
                        input_val,
                    ),
                )).collect(),
                type_annotation: TypeAnnotation::from_ast_type(
                    &ext_field_pos.into(),
                    &ext_field.field_type,
                ),
            });
        }

        Ok(())
    }
}

#[inherent]
impl TypeBuilder for InterfaceTypeBuilder {
    type AstTypeDef = ast::schema::InterfaceType;
    type AstTypeExtension = ast::schema::InterfaceTypeExtension;

    pub(crate) fn finalize(mut self, types_builder: &mut TypesMapBuilder) -> Result<()> {
        while let Some((ext_path, ext)) = self.extensions.pop() {
            let type_name = ext.name.as_str();
            match types_builder.get_type_mut(type_name) {
                Some(GraphQLType::Interface(iface_type)) =>
                    self.merge_type_extension(iface_type, ext_path.as_path(), ext)?,

                Some(non_iface_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_iface_type.clone(),
                        extension_loc: loc::FilePosition::from_pos(
                            ext_path,
                            ext.position,
                        ).into(),
                    }),

                None =>
                    return Err(SchemaBuildError::ExtensionOfUndefinedType {
                        type_name: ext.name.to_string(),
                        extension_type_loc: loc::FilePosition::from_pos(
                            ext_path,
                            ext.position,
                        ).into(),
                    })
            }
        }
        Ok(())
    }

    pub(crate) fn visit_type_def(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: &Path,
        def: <Self as TypeBuilder>::AstTypeDef,
    ) -> Result<()> {
        let file_position = loc::FilePosition::from_pos(
            file_path,
            def.position,
        );

        let fields = TypeBuilderHelpers::object_fielddefs_from_ast(
            &file_position,
            def.name.as_str(),
            &def.fields,
        )?;

        let directives = TypeBuilderHelpers::directive_refs_from_ast(
            file_path,
            &def.directives,
        );

        let interfaces = {
            let mut interface_names = HashSet::new();
            let mut interface_refs = vec![];
            for iface_name in &def.implements_interfaces {
                if *iface_name == def.name {
                    return Err(
                        SchemaBuildError::InvalidSelfImplementingInterface {
                            def_location: file_position.to_owned().into(),
                            interface_name: iface_name.to_string(),
                        }
                    );
                }

                if interface_names.insert(iface_name) {
                    interface_refs.push(NamedGraphQLTypeRef::new(
                        iface_name,
                        file_position.to_owned().into(),
                    ));
                } else {
                    // Interface type declarations must declare a unique list of
                    // interfaces they implement.
                    //
                    // https://spec.graphql.org/October2021/#sel-HAHbnBFFABABtD5vM
                    return Err(
                        SchemaBuildError::DuplicateInterfaceImplementsDeclaration {
                            def_location: file_position.to_owned().into(),
                            duplicated_interface_name: iface_name.to_string(),
                            type_name: def.name.to_string(),
                        }
                    );
                }
            }
            interface_refs
        };

        types_builder.add_new_type(
            file_position.clone(),
            def.name.as_str(),
            GraphQLType::Interface(InterfaceType(ObjectOrInterfaceTypeData {
                def_location: file_position.into(),
                directives,
                fields,
                interfaces,
                name: def.name.to_string(),
            }).into()),
        )
    }

    pub(crate) fn visit_type_extension(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: &Path,
        ext: <Self as TypeBuilder>::AstTypeExtension,
    ) -> Result<()> {
        let type_name = ext.name.as_str();
        match types_builder.get_type_mut(type_name) {
            Some(GraphQLType::Interface(iface_type)) =>
                self.merge_type_extension(iface_type, file_path, ext),

            Some(non_obj_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_obj_type.clone(),
                    extension_loc: loc::FilePosition::from_pos(
                        file_path,
                        ext.position,
                    ).into(),
                }),

            None => {
                self.extensions.push((file_path.to_path_buf(), ext));
                Ok(())
            },
        }
    }
}
