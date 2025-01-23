use crate::ast;
use crate::loc;
use crate::schema_builder::SchemaBuildError;
use crate::schema_builder::TypeBuilder;
use crate::schema_builder::TypesMapBuilder;
use crate::types::GraphQLDirectiveAnnotation;
use crate::types::GraphQLEnumVariant;
use crate::types::GraphQLEnumType;
use crate::types::GraphQLType;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[derive(Debug)]
pub(super) struct EnumTypeBuilder {
    extensions: Vec<(PathBuf, ast::schema::EnumTypeExtension)>,
}
impl EnumTypeBuilder {
    pub fn new() -> Self {
        Self {
            extensions: vec![],
        }
    }

    fn merge_type_extension(
        &mut self,
        type_: &mut GraphQLEnumType,
        ext_file_path: &Path,
        ext: ast::schema::EnumTypeExtension,
    ) -> Result<()> {
        type_.directives.append(&mut GraphQLDirectiveAnnotation::from_ast(
            ext_file_path,
            &ext.directives,
        ));

        for ext_val in ext.values.iter() {
            let ext_val_loc = loc::FilePosition::from_pos(
                ext_file_path,
                ext_val.position,
            );

            // Error if this value is already defined.
            if let Some(existing_value) = type_.variants.get(ext_val.name.as_str()) {
                return Err(SchemaBuildError::DuplicateEnumValueDefinition {
                    enum_name: ext.name.to_string(),
                    enum_def_location: type_.def_location.clone(),
                    value_def1: existing_value.def_location.clone(),
                    value_def2: ext_val_loc,
                });
            }
            type_.variants.insert(ext_val.name.to_string(), GraphQLEnumVariant {
                def_location: ext_val_loc,
                directives: GraphQLDirectiveAnnotation::from_ast(
                    ext_file_path,
                    &ext_val.directives,
                ),
                name: ext_val.name.to_string(),
            });
        }

        Ok(())
    }
}
impl TypeBuilder for EnumTypeBuilder {
    type AstTypeDef = ast::schema::EnumType;
    type AstTypeExtension = ast::schema::EnumTypeExtension;

    fn finalize(mut self, types_builder: &mut TypesMapBuilder) -> Result<()> {
        while let Some((ext_path, ext)) = self.extensions.pop() {
            let type_name = ext.name.as_str();
            match types_builder.get_type_mut(type_name) {
                Some(GraphQLType::Enum(enum_type)) =>
                    self.merge_type_extension(enum_type, ext_path.as_path(), ext)?,

                Some(non_enum_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_enum_type.clone(),
                        extension_loc: loc::FilePosition::from_pos(
                            ext_path,
                            ext.position,
                        ),
                    }),

                None =>
                    return Err(SchemaBuildError::ExtensionOfUndefinedType {
                        type_name: ext.name.to_string(),
                        extension_type_loc: loc::FilePosition::from_pos(
                            ext_path,
                            ext.position,
                        ),
                    })
            }
        }
        Ok(())
    }

    fn visit_type_def(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: &Path,
        def: Self::AstTypeDef,
    ) -> Result<()> {
        let file_position =
            loc::FilePosition::from_pos(file_path, def.position);

        let directives = GraphQLDirectiveAnnotation::from_ast(
            file_path,
            &def.directives,
        );

        let variants: HashMap<String, GraphQLEnumVariant> =
            def.values
                .iter()
                .map(|val| (val.name.to_string(), GraphQLEnumVariant {
                    def_location: loc::FilePosition::from_pos(
                        file_path,
                        val.position,
                    ),
                    directives: GraphQLDirectiveAnnotation::from_ast(
                        file_path,
                        &val.directives,
                    ),
                    name: val.name.to_string(),
                }))
                .collect();

        if variants.is_empty() {
            return Err(SchemaBuildError::EnumWithNoVariants {
                type_name: def.name.to_string(),
                location: file_position,
            });
        }

        types_builder.add_new_type(
            file_position.clone(),
            def.name.as_str(),
            GraphQLType::Enum(GraphQLEnumType {
                def_location: file_position,
                directives,
                name: def.name.to_string(),
                variants,
            }),
        )
    }

    fn visit_type_extension(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: &Path,
        ext: Self::AstTypeExtension,
    ) -> Result<()> {
        let type_name = ext.name.as_str();
        match types_builder.get_type_mut(type_name) {
            Some(GraphQLType::Enum(enum_type)) =>
                self.merge_type_extension(enum_type, file_path, ext),

            Some(non_enum_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_enum_type.clone(),
                    extension_loc: loc::FilePosition::from_pos(
                        file_path,
                        ext.position,
                    ),
                }),

            None => {
                self.extensions.push((file_path.to_path_buf(), ext));
                Ok(())
            },
        }
    }
}
