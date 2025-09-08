use crate::ast;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::GraphQLType;
use crate::types::InputObjectType;
use crate::types::InputField;
use crate::types::TypeAnnotation;
use crate::types::TypeBuilder;
use crate::types::TypeBuilderHelpers;
use crate::types::TypesMapBuilder;
use inherent::inherent;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[derive(Debug)]
pub(crate) struct InputObjectTypeBuilder {
    extensions: Vec<(Option<PathBuf>, ast::schema::InputObjectTypeExtension)>,
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
        ext_file_path: Option<&Path>,
        ext: &ast::schema::InputObjectTypeExtension,
    ) -> Result<()> {
        let ext_srcloc = loc::SourceLocation::from_schema_ast_position(
            ext_file_path,
            &ext.position,
        );

        inputobj_type.directives.append(&mut TypeBuilderHelpers::directive_refs_from_ast(
            &ext_srcloc,
            &ext.directives,
        ));

        for ext_field in ext.fields.iter() {
            let fielddef_srcloc = ext_srcloc.with_ast_position(&ext_field.position);

            // Error if this field is already defined.
            if let Some(existing_field) = inputobj_type.fields.get(ext_field.name.as_str()) {
                return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                    type_name: ext.name.to_string(),
                    field_name: ext_field.name.to_string(),
                    field_def1: existing_field.def_location.clone(),
                    field_def2: fielddef_srcloc,
                })?;
            }
            inputobj_type.fields.insert(ext_field.name.to_string(), InputField {
                description: ext_field.description.to_owned(),
                directives: TypeBuilderHelpers::directive_refs_from_ast(
                    &fielddef_srcloc,
                    &ext_field.directives,
                ),
                name: ext_field.name.to_string(),
                type_annotation: TypeAnnotation::from_ast_type(
                    &fielddef_srcloc,
                    &ext_field.value_type,
                ),
                def_location: fielddef_srcloc,
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
                    self.merge_type_extension(inputobj_type, ext_path.as_deref(), &ext)?,

                Some(non_obj_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_obj_type.clone(),
                        extension_location: loc::SourceLocation::from_schema_ast_position(
                            ext_path.as_deref(),
                            &ext.position,
                        ),
                    }),

                None =>
                    return Err(SchemaBuildError::ExtensionOfUndefinedType {
                        type_name: ext.name.to_string(),
                        extension_location: loc::SourceLocation::from_schema_ast_position(
                            ext_path.as_deref(),
                            &ext.position,
                        ),
                    })
            }
        }
        Ok(())
    }

    pub(crate) fn visit_type_def(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        def: &<Self as TypeBuilder>::AstTypeDef,
    ) -> Result<()> {
        let inputobjdef_srcloc = loc::SourceLocation::from_schema_ast_position(
            file_path,
            &def.position,
        );

        let fields = TypeBuilderHelpers::inputobject_fields_from_ast(
            &inputobjdef_srcloc,
            &def.name,
            &def.fields,
        )?;

        let directives = TypeBuilderHelpers::directive_refs_from_ast(
            &inputobjdef_srcloc,
            &def.directives,
        );

        types_builder.add_new_type(
            def.name.as_str(),
            &inputobjdef_srcloc.to_owned(),
            GraphQLType::InputObject(InputObjectType {
                description: def.description.to_owned(),
                directives,
                fields,
                name: def.name.to_string(),
                def_location: inputobjdef_srcloc,
            }.into()),
        )
    }

    pub(crate) fn visit_type_extension(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        ext: <Self as TypeBuilder>::AstTypeExtension,
    ) -> Result<()> {
        let type_name = ext.name.as_str();
        match types_builder.get_type_mut(type_name) {
            Some(GraphQLType::InputObject(inputobj_type)) =>
                self.merge_type_extension(inputobj_type, file_path, &ext),

            Some(non_obj_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_obj_type.clone(),
                    extension_location: loc::SourceLocation::from_schema_ast_position(
                        file_path,
                        &ext.position,
                    ),
                }),

            None => {
                self.extensions.push(
                    (file_path.map(|p| p.to_path_buf()), ext)
                );
                Ok(())
            },
        }
    }
}
