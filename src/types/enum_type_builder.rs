use crate::ast;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::TypeBuilder;
use crate::types::TypesMapBuilder;
use crate::types::EnumValue;
use crate::types::EnumType;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use inherent::inherent;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[derive(Debug)]
pub struct EnumTypeBuilder {
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
        type_: &mut EnumType,
        ext_file_path: &Path,
        ext: ast::schema::EnumTypeExtension,
    ) -> Result<()> {
        type_.directives.append(&mut DirectiveAnnotation::from_ast(
            ext_file_path,
            &ext.directives,
        ));

        for ext_val in ext.values.iter() {
            let ext_val_loc = loc::FilePosition::from_pos(
                ext_file_path,
                ext_val.position,
            );

            // Error if this value is already defined.
            if let Some(existing_value) = type_.values.get(ext_val.name.as_str()) {
                return Err(SchemaBuildError::DuplicateEnumValueDefinition {
                    enum_name: ext.name.to_string(),
                    enum_def_location: type_.def_location.clone(),
                    value_def1: existing_value.def_location.clone(),
                    value_def2: ext_val_loc.into(),
                });
            }
            type_.values.insert(ext_val.name.to_string(), EnumValue {
                def_location: ext_val_loc.to_owned().into(),
                directives: DirectiveAnnotation::from_ast(
                    ext_file_path,
                    &ext_val.directives,
                ),
                name: ext_val.name.to_string(),
                type_ref: NamedGraphQLTypeRef::new(
                    type_.name.as_str(),
                    ext_val_loc.into(),
                ),
            });
        }

        Ok(())
    }
}

#[inherent]
impl TypeBuilder for EnumTypeBuilder {
    type AstTypeDef = ast::schema::EnumType;
    type AstTypeExtension = ast::schema::EnumTypeExtension;

    pub(crate) fn finalize(mut self, types_builder: &mut TypesMapBuilder) -> Result<()> {
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
        let file_position =
            loc::FilePosition::from_pos(file_path, def.position);

        let directives = DirectiveAnnotation::from_ast(
            file_path,
            &def.directives,
        );

        let values: BTreeMap<String, EnumValue> =
            def.values
                .iter()
                .map(|val| {
                    let def_location = loc::FilePosition::from_pos(
                        file_path,
                        val.position,
                    );

                    (val.name.to_string(), EnumValue {
                        def_location: def_location.to_owned().into(),
                        directives: DirectiveAnnotation::from_ast(
                            file_path,
                            &val.directives,
                        ),
                        name: val.name.to_string(),
                        type_ref: NamedGraphQLTypeRef::new(
                            def.name.as_str(),
                            def_location.into(),
                        ),
                    })
                })
                .collect();

        if values.is_empty() {
            return Err(SchemaBuildError::EnumWithNoVariants {
                type_name: def.name.to_string(),
                location: file_position.into(),
            });
        }

        types_builder.add_new_type(
            file_position.clone(),
            def.name.as_str(),
            GraphQLType::Enum(EnumType {
                def_location: file_position.into(),
                directives,
                name: def.name.to_string(),
                values,
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
            Some(GraphQLType::Enum(enum_type)) =>
                self.merge_type_extension(enum_type, file_path, ext),

            Some(non_enum_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_enum_type.clone(),
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
