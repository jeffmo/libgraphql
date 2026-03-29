use crate::ast;
use crate::DirectiveAnnotation;
use crate::DirectiveAnnotationBuilder;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::GraphQLType;
use crate::types::InputField;
use crate::types::InputObjectType;
use crate::types::NamedGraphQLTypeRef;
use crate::types::TypeAnnotation;
use crate::types::TypeBuilder;
use crate::types::TypeBuilderHelpers;
use crate::types::TypesMapBuilder;
use indexmap::IndexMap;
use std::path::Path;

type Result<T> = std::result::Result<T, SchemaBuildError>;

/// Owned representation of a deferred input object type extension.
#[derive(Debug)]
struct DeferredInputObjectExtension {
    directives: Vec<DirectiveAnnotation>,
    fields: IndexMap<String, InputField>,
    name: String,
    srcloc: loc::SourceLocation,
}

#[derive(Debug)]
pub(crate) struct InputObjectTypeBuilder {
    extensions: Vec<DeferredInputObjectExtension>,
}

impl InputObjectTypeBuilder {
    pub fn new() -> Self {
        Self {
            extensions: vec![],
        }
    }

    fn merge_extension(
        &mut self,
        inputobj_type: &mut InputObjectType,
        ext: &mut DeferredInputObjectExtension,
    ) -> Result<()> {
        inputobj_type.directives.append(&mut ext.directives);

        for (field_name, field) in ext.fields.drain(..) {
            // Error if this field is already defined.
            if let Some(existing_field) =
                inputobj_type.fields.get(field_name.as_str())
            {
                return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                    type_name: ext.name.to_string(),
                    field_name,
                    field_def1: existing_field.def_location.clone(),
                    field_def2: field.def_location.clone(),
                })?;
            }
            inputobj_type.fields.insert(field_name, field);
        }

        Ok(())
    }

    fn build_extension_from_ast(
        &self,
        ext_srcloc: &loc::SourceLocation,
        ext: &ast::InputObjectTypeExtension<'_>,
        source_map: &ast::SourceMap<'_>,
        type_name: &str,
    ) -> DeferredInputObjectExtension {
        let directives = DirectiveAnnotationBuilder::from_ast(
            ext_srcloc,
            source_map,
            &ext.directives,
        );

        let mut fields = IndexMap::new();
        for ext_field in ext.fields.iter() {
            let field_name = ext_field.name.value.as_ref().to_string();
            let fielddef_srcloc =
                ext_srcloc.with_span(ext_field.span, source_map);

            fields.insert(field_name.clone(), InputField {
                description: ext_field.description.as_ref()
                    .map(|d| d.value.to_string()),
                directives: DirectiveAnnotationBuilder::from_ast(
                    &fielddef_srcloc,
                    source_map,
                    &ext_field.directives,
                ),
                name: field_name,
                parent_type: NamedGraphQLTypeRef::new(
                    type_name,
                    ext_srcloc.to_owned(),
                ),
                type_annotation: TypeAnnotation::from_ast_type(
                    &fielddef_srcloc,
                    &ext_field.value_type,
                ),
                def_location: fielddef_srcloc,
            });
        }

        DeferredInputObjectExtension {
            directives,
            fields,
            name: type_name.to_string(),
            srcloc: ext_srcloc.clone(),
        }
    }
}

impl InputObjectTypeBuilder {
    pub(crate) fn visit_type_def(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        def: &ast::InputObjectTypeDefinition<'_>,
        source_map: &ast::SourceMap<'_>,
    ) -> Result<()> {
        let inputobjdef_srcloc = loc::SourceLocation::from_schema_span(
            file_path,
            def.span,
            source_map,
        );

        let def_name = def.name.value.as_ref();
        let fields = TypeBuilderHelpers::inputobject_fields_from_ast(
            &inputobjdef_srcloc,
            def_name,
            &def.fields,
            source_map,
        )?;

        let directives = DirectiveAnnotationBuilder::from_ast(
            &inputobjdef_srcloc,
            source_map,
            &def.directives,
        );

        types_builder.add_new_type(
            def_name,
            &inputobjdef_srcloc.to_owned(),
            GraphQLType::InputObject(InputObjectType {
                description: def.description.as_ref()
                    .map(|d| d.value.to_string()),
                directives,
                fields,
                name: def_name.to_string(),
                def_location: inputobjdef_srcloc,
            }.into()),
        )
    }

    pub(crate) fn visit_type_extension(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        ext: &ast::InputObjectTypeExtension<'_>,
        source_map: &ast::SourceMap<'_>,
    ) -> Result<()> {
        let ext_srcloc = loc::SourceLocation::from_schema_span(
            file_path,
            ext.span,
            source_map,
        );
        let type_name = ext.name.value.as_ref();
        let mut deferred = self.build_extension_from_ast(
            &ext_srcloc, ext, source_map, type_name,
        );

        match types_builder.get_type_mut(type_name) {
            Some(GraphQLType::InputObject(inputobj_type)) =>
                self.merge_extension(inputobj_type, &mut deferred),

            Some(non_obj_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_obj_type.clone(),
                    extension_location: ext_srcloc,
                }),

            None => {
                self.extensions.push(deferred);
                Ok(())
            },
        }
    }
}

impl TypeBuilder for InputObjectTypeBuilder {
    fn finalize(
        mut self,
        types_builder: &mut TypesMapBuilder,
    ) -> Result<()> {
        while let Some(mut ext) = self.extensions.pop() {
            let type_name = ext.name.as_str();
            match types_builder.get_type_mut(type_name) {
                Some(GraphQLType::InputObject(inputobj_type)) =>
                    self.merge_extension(inputobj_type, &mut ext)?,

                Some(non_obj_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_obj_type.clone(),
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
