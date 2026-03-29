use crate::ast;
use crate::DirectiveAnnotation;
use crate::DirectiveAnnotationBuilder;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::Field;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use crate::types::ObjectOrInterfaceTypeData;
use crate::types::ObjectType;
use crate::types::Parameter;
use crate::types::TypeAnnotation;
use crate::types::TypeBuilder;
use crate::types::TypeBuilderHelpers;
use crate::types::TypesMapBuilder;
use indexmap::IndexMap;
use std::collections::HashSet;
use std::path::Path;

type Result<T> = std::result::Result<T, SchemaBuildError>;

/// Owned representation of a deferred object type extension.
#[derive(Debug)]
struct DeferredObjectExtension {
    directives: Vec<DirectiveAnnotation>,
    fields: IndexMap<String, Field>,
    name: String,
    srcloc: loc::SourceLocation,
}

#[derive(Debug)]
pub(crate) struct ObjectTypeBuilder {
    extensions: Vec<DeferredObjectExtension>,
}

impl ObjectTypeBuilder {
    pub fn new() -> Self {
        Self {
            extensions: vec![],
        }
    }

    fn merge_extension(
        &mut self,
        obj_type: &mut ObjectType,
        ext: &mut DeferredObjectExtension,
    ) -> Result<()> {
        obj_type.0.directives.append(&mut ext.directives);

        for (field_name, field) in ext.fields.drain(..) {
            // Error if this field is already defined.
            if let Some(existing_field) = obj_type.0.fields.get(field_name.as_str()) {
                return Err(SchemaBuildError::DuplicateFieldNameDefinition {
                    type_name: ext.name.to_string(),
                    field_name,
                    field_def1: existing_field.def_location().clone(),
                    field_def2: field.def_location().clone(),
                })?;
            }
            obj_type.0.fields.insert(field_name, field);
        }

        Ok(())
    }

    fn build_extension_from_ast(
        &self,
        ext_srcloc: &loc::SourceLocation,
        ext: &ast::ObjectTypeExtension<'_>,
        source_map: &ast::SourceMap<'_>,
        obj_type_name: &str,
    ) -> DeferredObjectExtension {
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

            fields.insert(field_name.clone(), Field {
                description: ext_field.description.as_ref()
                    .map(|d| d.value.to_string()),
                directives: DirectiveAnnotationBuilder::from_ast(
                    &fielddef_srcloc,
                    source_map,
                    &ext_field.directives,
                ),
                name: field_name,
                parameters: ext_field.parameters.iter().map(|input_val| (
                    input_val.name.value.as_ref().to_string(),
                    Parameter::from_ast_with_parent_loc(
                        ext_srcloc,
                        input_val,
                        source_map,
                    )
                )).collect(),
                parent_type: NamedGraphQLTypeRef::new(
                    obj_type_name,
                    ext_srcloc.to_owned(),
                ),
                type_annotation: TypeAnnotation::from_ast_type(
                    &fielddef_srcloc,
                    &ext_field.field_type,
                ),
                def_location: fielddef_srcloc,
            });
        }

        DeferredObjectExtension {
            directives,
            fields,
            name: obj_type_name.to_string(),
            srcloc: ext_srcloc.clone(),
        }
    }
}

impl ObjectTypeBuilder {
    pub(crate) fn visit_type_def(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        def: &ast::ObjectTypeDefinition<'_>,
        source_map: &ast::SourceMap<'_>,
    ) -> Result<()> {
        let objdef_srcloc = loc::SourceLocation::from_schema_span(
            file_path,
            def.span,
            source_map,
        );

        let fields = TypeBuilderHelpers::object_fielddefs_from_ast(
            &objdef_srcloc,
            def.name.value.as_ref(),
            &def.fields,
            source_map,
        )?;

        let directives = DirectiveAnnotationBuilder::from_ast(
            &objdef_srcloc,
            source_map,
            &def.directives,
        );

        let interfaces = {
            let mut interface_names = HashSet::new();
            let mut interface_refs = vec![];
            for iface_name in &def.implements {
                let iface_name_str = iface_name.value.as_ref();
                if interface_names.insert(iface_name_str.to_string()) {
                    interface_refs.push(NamedGraphQLTypeRef::new(
                        iface_name_str,
                        objdef_srcloc.to_owned(),
                    ));
                } else {
                    // Object type declarations must declare a unique list of
                    // interfaces they implement.
                    //
                    // https://spec.graphql.org/October2021/#sel-HAHZhCFFABABsCqgY
                    return Err(
                        SchemaBuildError::DuplicateInterfaceImplementsDeclaration {
                            def_location: objdef_srcloc.to_owned(),
                            duplicated_interface_name: iface_name_str.to_string(),
                            type_name: def.name.value.as_ref().to_string(),
                        }
                    );
                }
            }
            interface_refs
        };

        types_builder.add_new_type(
            def.name.value.as_ref(),
            &objdef_srcloc.to_owned(),
            GraphQLType::Object(ObjectType(ObjectOrInterfaceTypeData {
                def_location: objdef_srcloc,
                description: def.description.as_ref()
                    .map(|d| d.value.to_string()),
                directives,
                fields,
                interfaces,
                name: def.name.value.as_ref().to_string(),
            }).into()),
        )
    }

    pub(crate) fn visit_type_extension(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        ext: &ast::ObjectTypeExtension<'_>,
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
            Some(GraphQLType::Object(obj_type)) =>
                self.merge_extension(obj_type, &mut deferred),

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

impl TypeBuilder for ObjectTypeBuilder {
    fn finalize(mut self, types_builder: &mut TypesMapBuilder) -> Result<()> {
        while let Some(mut ext) = self.extensions.pop() {
            let type_name = ext.name.as_str();
            match types_builder.get_type_mut(type_name) {
                Some(GraphQLType::Object(obj_type)) =>
                    self.merge_extension(obj_type, &mut ext)?,

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
