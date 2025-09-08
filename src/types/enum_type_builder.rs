use crate::ast;
use crate::types::type_builder::TypeBuilderHelpers;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::TypeBuilder;
use crate::types::TypesMapBuilder;
use crate::types::EnumValue;
use crate::types::EnumType;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use indexmap::IndexMap;
use inherent::inherent;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[derive(Debug)]
pub(crate) struct EnumTypeBuilder {
    extensions: Vec<(Option<PathBuf>, ast::schema::EnumTypeExtension)>,
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
        ext_file_path: Option<&Path>,
        ext: &ast::schema::EnumTypeExtension,
    ) -> Result<()> {
        let ext_srcloc = loc::SourceLocation::from_schema_ast_position(
            ext_file_path,
            &ext.position,
        );
        type_.directives.append(&mut TypeBuilderHelpers::directive_refs_from_ast(
            &ext_srcloc,
            &ext.directives,
        ));

        for ext_val in ext.values.iter() {
            let ext_val_srcloc = ext_srcloc.with_ast_position(&ext_val.position);

            // Error if this value is already defined.
            if let Some(existing_value) = type_.values.get(ext_val.name.as_str()) {
                return Err(SchemaBuildError::DuplicateEnumValueDefinition {
                    enum_name: ext.name.to_string(),
                    enum_def_location: type_.def_location.clone(),
                    value_def1: existing_value.def_location.clone(),
                    value_def2: ext_val_srcloc,
                });
            }
            type_.values.insert(ext_val.name.to_string(), EnumValue {
                def_location: ext_val_srcloc.to_owned(),
                description: ext_val.description.to_owned(),
                directives: TypeBuilderHelpers::directive_refs_from_ast(
                    &ext_val_srcloc,
                    &ext_val.directives,
                ),
                name: ext_val.name.to_string(),
                type_ref: NamedGraphQLTypeRef::new(
                    type_.name.as_str(),
                    ext_val_srcloc,
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
                    self.merge_type_extension(enum_type, ext_path.as_deref(), &ext)?,

                Some(non_enum_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_enum_type.clone(),
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
        let enumdef_srcloc = loc::SourceLocation::from_schema_ast_position(
            file_path,
            &def.position,
        );

        let directives = TypeBuilderHelpers::directive_refs_from_ast(
            &enumdef_srcloc,
            &def.directives,
        );

        let mut enum_values = IndexMap::<String, EnumValue>::new();
        for enum_value in &def.values {
            let value_name = enum_value.name.to_string();
            let valuedef_srcloc = enumdef_srcloc.with_ast_position(&enum_value.position);
            if let Some(existing_value) = enum_values.get(value_name.as_str()) {
                return Err(SchemaBuildError::DuplicateEnumValueDefinition {
                    enum_name: value_name,
                    enum_def_location: enumdef_srcloc,
                    value_def1: existing_value.def_location().to_owned(),
                    value_def2: valuedef_srcloc,
                });
            } else {
                enum_values.insert(
                    value_name.to_string(),
                    EnumValue {
                        def_location: valuedef_srcloc.to_owned(),
                        description: enum_value.description.to_owned(),
                        directives: TypeBuilderHelpers::directive_refs_from_ast(
                            &valuedef_srcloc,
                            &enum_value.directives,
                        ),
                        type_ref: NamedGraphQLTypeRef::new(
                            def.name.as_str(),
                            valuedef_srcloc,
                        ),
                        name: value_name,
                    },
                );
            }
        }

        if enum_values.is_empty() {
            return Err(SchemaBuildError::EnumWithNoVariants {
                type_name: def.name.to_string(),
                location: enumdef_srcloc,
            });
        }

        types_builder.add_new_type(
            def.name.as_str(),
            &enumdef_srcloc.to_owned(),
            GraphQLType::Enum(EnumType {
                def_location: enumdef_srcloc,
                description: def.description.to_owned(),
                directives,
                name: def.name.to_string(),
                values: enum_values,
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
            Some(GraphQLType::Enum(enum_type)) =>
                self.merge_type_extension(enum_type, file_path, &ext),

            Some(non_enum_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_enum_type.clone(),
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
