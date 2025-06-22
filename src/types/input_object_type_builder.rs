use crate::ast;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::TypeBuilder;
use crate::types::TypeBuilderHelpers;
use crate::types::TypesMapBuilder;
use crate::types::InputObjectType;
use crate::types::GraphQLType;
use crate::types::InputField;
use inherent::inherent;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

// TODO(!!!): InputObjects' fields are actually InputValues (not fields).
//            Need to build these types accordingly...
#[derive(Debug)]
pub struct InputObjectTypeBuilder {
    extensions: Vec<(PathBuf, ast::schema::InputObjectTypeExtension)>,
}

impl InputObjectTypeBuilder {
    pub fn new() -> Self {
        Self {
            extensions: vec![],
        }
    }

    fn merge_type_extension(
        &mut self,
        inputobj_type: &mut InputObjectType,
        ext_file_path: &Path,
        ext: ast::schema::InputObjectTypeExtension,
    ) -> Result<()> {
        inputobj_type.directives.append(&mut TypeBuilderHelpers::directive_refs_from_ast(
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
            if let Some(existing_field) = inputobj_type.fields.get(ext_field.name.as_str()) {
                return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                    type_name: ext.name.to_string(),
                    field_name: ext_field.name.to_string(),
                    field_def1: existing_field.def_location.clone(),
                    field_def2: ext_field_loc,
                })?;
            }
            inputobj_type.fields.insert(ext_field.name.to_string(), InputField {
                def_location: ext_field_loc,
                // TODO: ...InputValue fields...
            });
        }

        Ok(())
    }
}

#[inherent]
impl TypeBuilder for InputObjectTypeBuilder {
    type AstTypeDef = ast::schema::InputObjectType;
    type AstTypeExtension = ast::schema::InputObjectTypeExtension;

    pub(crate) fn finalize(mut self, types_builder: &mut TypesMapBuilder) -> Result<()> {
        while let Some((ext_path, ext)) = self.extensions.pop() {
            let type_name = ext.name.as_str();
            match types_builder.get_type_mut(type_name) {
                Some(GraphQLType::InputObject(inputobj_type)) =>
                    self.merge_type_extension(inputobj_type, ext_path.as_path(), ext)?,

                Some(non_obj_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_obj_type.clone(),
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

        let fields = TypeBuilderHelpers::inputobject_fields_from_ast(
            &loc::SchemaDefLocation::Schema(
                file_position.clone(),
            ),
            &def.fields,
        )?;

        let directives = TypeBuilderHelpers::directive_refs_from_ast(
            file_path,
            &def.directives,
        );

        types_builder.add_new_type(
            file_position.clone(),
            def.name.as_str(),
            GraphQLType::InputObject(InputObjectType {
                def_location: file_position,
                directives,
                fields,
                name: def.name.to_string(),
            }),
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
            Some(GraphQLType::InputObject(inputobj_type)) =>
                self.merge_type_extension(inputobj_type, file_path, ext),

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
