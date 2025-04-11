use crate::ast;
use crate::loc;
use crate::SchemaBuildError;
use crate::types::Field;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeRef;
use crate::types::NamedGraphQLTypeRef;
use crate::types::ObjectType;
use crate::types::TypeBuilder;
use crate::types::TypeBuilderHelpers;
use crate::types::TypesMapBuilder;
use inherent::inherent;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[derive(Debug)]
pub struct ObjectTypeBuilder {
    extensions: Vec<(PathBuf, ast::schema::ObjectTypeExtension)>,
    interfaces: Vec<NamedGraphQLTypeRef>,
}

impl ObjectTypeBuilder {
    pub fn new() -> Self {
        Self {
            extensions: vec![],
            interfaces: vec![],
        }
    }

    fn merge_type_extension(
        &mut self,
        obj_type: &mut ObjectType,
        ext_file_path: &Path,
        ext: ast::schema::ObjectTypeExtension,
    ) -> Result<()> {
        obj_type.directives.append(&mut TypeBuilderHelpers::directive_refs_from_ast(
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
            if let Some(existing_field) = obj_type.fields.get(ext_field.name.as_str()) {
                return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                    type_name: ext.name.to_string(),
                    field_name: ext_field.name.to_string(),
                    field_def1: existing_field.def_location.clone(),
                    field_def2: ext_field_loc,
                })?;
            }
            obj_type.fields.insert(ext_field.name.to_string(), Field {
                type_ref: GraphQLTypeRef::from_ast_type(
                    &ext_field_pos,
                    &ext_field.field_type,
                ),
                def_location: ext_field_loc,
            });
        }

        Ok(())
    }
}

#[inherent]
impl TypeBuilder for ObjectTypeBuilder {
    type AstTypeDef = ast::schema::ObjectType;
    type AstTypeExtension = ast::schema::ObjectTypeExtension;

    pub(crate) fn finalize(mut self, types_builder: &mut TypesMapBuilder) -> Result<()> {
        while let Some((ext_path, ext)) = self.extensions.pop() {
            let type_name = ext.name.as_str();
            match types_builder.get_type_mut(type_name) {
                Some(GraphQLType::Object(obj_type)) =>
                    self.merge_type_extension(obj_type, ext_path.as_path(), ext)?,

                Some(non_obj_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_obj_type.clone(),
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
            &def.fields,
        );

        let directives = TypeBuilderHelpers::directive_refs_from_ast(
            file_path,
            &def.directives,
        );

        let interfaces = def.implements_interfaces.iter().map(|iface_name| {
            NamedGraphQLTypeRef::new(iface_name, file_position.to_owned())
        }).collect();

        types_builder.add_new_type(
            file_position.clone(),
            def.name.as_str(),
            GraphQLType::Object(ObjectType {
                def_location: file_position,
                directives,
                fields,
                interfaces,
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
            Some(GraphQLType::Object(obj_type)) =>
                self.merge_type_extension(obj_type, file_path, ext),

            Some(non_obj_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_obj_type.clone(),
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
