use crate::ast;
use crate::loc;
use crate::SchemaBuildError;
use crate::types::TypeBuilder;
use crate::types::TypeBuilderHelpers;
use crate::types::TypesMapBuilder;
use crate::types::ScalarType;
use crate::types::GraphQLType;
use inherent::inherent;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[derive(Debug)]
pub struct ScalarTypeBuilder {
    extensions: Vec<(PathBuf, ast::schema::ScalarTypeExtension)>,
}

impl ScalarTypeBuilder {
    pub fn new() -> Self {
        Self {
            extensions: vec![],
        }
    }

    fn merge_type_extension(
        &mut self,
        scalar_type: &mut ScalarType,
        ext_file_path: &Path,
        ext: ast::schema::ScalarTypeExtension,
    ) -> Result<()> {
        // TODO: Non-repeatable directives must not be repeated here:
        //       https://spec.graphql.org/October2021/#sec-Scalar-Extensions.Type-Validation
        scalar_type.directives.append(&mut TypeBuilderHelpers::directive_refs_from_ast(
            ext_file_path,
            &ext.directives,
        ));

        Ok(())
    }
}

#[inherent]
impl TypeBuilder for ScalarTypeBuilder {
    type AstTypeDef = ast::schema::ScalarType;
    type AstTypeExtension = ast::schema::ScalarTypeExtension;

    pub(crate) fn finalize(mut self, types_builder: &mut TypesMapBuilder) -> Result<()> {
        while let Some((ext_path, ext)) = self.extensions.pop() {
            let type_name = ext.name.as_str();
            match types_builder.get_type_mut(type_name) {
                Some(GraphQLType::Scalar(scalar_type)) =>
                    self.merge_type_extension(scalar_type, ext_path.as_path(), ext)?,

                Some(non_scalar_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_scalar_type.clone(),
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

        let directives = TypeBuilderHelpers::directive_refs_from_ast(
            file_path,
            &def.directives,
        );

        types_builder.add_new_type(
            file_position.clone(),
            def.name.as_str(),
            GraphQLType::Scalar(ScalarType {
                def_location: file_position,
                directives,
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
            Some(GraphQLType::Scalar(scalar_type)) =>
                self.merge_type_extension(scalar_type, file_path, ext),

            Some(non_scalar_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_scalar_type.clone(),
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
