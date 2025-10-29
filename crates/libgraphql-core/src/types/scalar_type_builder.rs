use crate::ast;
use crate::DirectiveAnnotationBuilder;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::TypeBuilder;
use crate::types::TypesMapBuilder;
use crate::types::ScalarType;
use crate::types::GraphQLType;
use inherent::inherent;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[derive(Debug)]
pub(crate) struct ScalarTypeBuilder {
    extensions: Vec<(Option<PathBuf>, ast::schema::ScalarTypeExtension)>,
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
        ext_file_path: Option<&Path>,
        ext: &ast::schema::ScalarTypeExtension,
    ) -> Result<()> {
        let ext_srcloc = loc::SourceLocation::from_schema_ast_position(
            ext_file_path,
            &ext.position,
        );
        // TODO: Non-repeatable directives must not be repeated here:
        //       https://spec.graphql.org/October2021/#sec-Scalar-Extensions.Type-Validation
        scalar_type.directives.append(&mut DirectiveAnnotationBuilder::from_ast(
            &ext_srcloc,
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
                    self.merge_type_extension(scalar_type, ext_path.as_deref(), &ext)?,

                Some(non_scalar_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_scalar_type.clone(),
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
        let scalardef_srcloc = loc::SourceLocation::from_schema_ast_position(
            file_path,
            &def.position,
        );

        let directives = DirectiveAnnotationBuilder::from_ast(
            &scalardef_srcloc,
            &def.directives,
        );

        types_builder.add_new_type(
            def.name.as_str(),
            &scalardef_srcloc.to_owned(),
            GraphQLType::Scalar(ScalarType {
                def_location: scalardef_srcloc,
                description: def.description.to_owned(),
                directives,
                name: def.name.to_string(),
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
            Some(GraphQLType::Scalar(scalar_type)) =>
                self.merge_type_extension(scalar_type, file_path, &ext),

            Some(non_scalar_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_scalar_type.clone(),
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
