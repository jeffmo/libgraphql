use crate::ast;
use crate::DirectiveAnnotation;
use crate::DirectiveAnnotationBuilder;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::GraphQLType;
use crate::types::ScalarType;
use crate::types::TypeBuilder;
use crate::types::TypesMapBuilder;
use std::path::Path;

type Result<T> = std::result::Result<T, SchemaBuildError>;

/// Owned representation of a deferred scalar type extension.
#[derive(Debug)]
struct DeferredScalarExtension {
    directives: Vec<DirectiveAnnotation>,
    name: String,
    srcloc: loc::SourceLocation,
}

#[derive(Debug)]
pub(crate) struct ScalarTypeBuilder {
    extensions: Vec<DeferredScalarExtension>,
}

impl ScalarTypeBuilder {
    pub fn new() -> Self {
        Self {
            extensions: vec![],
        }
    }

    fn merge_extension(
        &mut self,
        scalar_type: &mut ScalarType,
        ext: &mut DeferredScalarExtension,
    ) -> Result<()> {
        // TODO: Non-repeatable directives must not be repeated here:
        //       https://spec.graphql.org/October2021/#sec-Scalar-Extensions.Type-Validation
        scalar_type.directives.append(&mut ext.directives);
        Ok(())
    }

    pub(crate) fn visit_type_def(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        def: &ast::ScalarTypeDefinition<'_>,
        source_map: &ast::SourceMap<'_>,
    ) -> Result<()> {
        let scalardef_srcloc = loc::SourceLocation::from_schema_span(
            file_path,
            def.span,
            source_map,
        );

        let directives = DirectiveAnnotationBuilder::from_ast(
            &scalardef_srcloc,
            source_map,
            &def.directives,
        );

        types_builder.add_new_type(
            def.name.value.as_ref(),
            &scalardef_srcloc.to_owned(),
            GraphQLType::Scalar(ScalarType {
                def_location: scalardef_srcloc,
                description: def.description.as_ref()
                    .map(|d| d.value.to_string()),
                directives,
                name: def.name.value.as_ref().to_string(),
            }.into()),
        )
    }

    pub(crate) fn visit_type_extension(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        ext: &ast::ScalarTypeExtension<'_>,
        source_map: &ast::SourceMap<'_>,
    ) -> Result<()> {
        let ext_srcloc = loc::SourceLocation::from_schema_span(
            file_path,
            ext.span,
            source_map,
        );
        let type_name = ext.name.value.as_ref();
        let mut deferred = DeferredScalarExtension {
            directives: DirectiveAnnotationBuilder::from_ast(
                &ext_srcloc,
                source_map,
                &ext.directives,
            ),
            name: type_name.to_string(),
            srcloc: ext_srcloc.clone(),
        };

        match types_builder.get_type_mut(type_name) {
            Some(GraphQLType::Scalar(scalar_type)) =>
                self.merge_extension(scalar_type, &mut deferred),

            Some(non_scalar_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_scalar_type.clone(),
                    extension_location: ext_srcloc,
                }),

            None => {
                self.extensions.push(deferred);
                Ok(())
            },
        }
    }
}

impl TypeBuilder for ScalarTypeBuilder {
    fn finalize(mut self, types_builder: &mut TypesMapBuilder) -> Result<()> {
        while let Some(mut ext) = self.extensions.pop() {
            let type_name = ext.name.as_str();
            match types_builder.get_type_mut(type_name) {
                Some(GraphQLType::Scalar(scalar_type)) =>
                    self.merge_extension(scalar_type, &mut ext)?,

                Some(non_scalar_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_scalar_type.clone(),
                        extension_location: ext.srcloc,
                    }),

                None =>
                    return Err(SchemaBuildError::ExtensionOfUndefinedType {
                        type_name: ext.name,
                        extension_location: ext.srcloc,
                    }),
            }
        }
        Ok(())
    }
}
