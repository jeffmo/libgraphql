use crate::ast;
use crate::DirectiveAnnotation;
use crate::DirectiveAnnotationBuilder;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::EnumType;
use crate::types::EnumValue;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use crate::types::TypeBuilder;
use crate::types::TypesMapBuilder;
use indexmap::IndexMap;
use std::path::Path;

type Result<T> = std::result::Result<T, SchemaBuildError>;

/// Owned representation of a deferred enum type extension, produced by
/// eagerly converting AST data so that no borrowed AST lifetime is needed.
#[derive(Debug)]
struct DeferredEnumExtension {
    directives: Vec<DirectiveAnnotation>,
    name: String,
    srcloc: loc::SourceLocation,
    values: Vec<EnumValue>,
}

#[derive(Debug)]
pub(crate) struct EnumTypeBuilder {
    extensions: Vec<DeferredEnumExtension>,
}

impl EnumTypeBuilder {
    pub fn new() -> Self {
        Self {
            extensions: vec![],
        }
    }

    fn merge_extension(
        &mut self,
        type_: &mut EnumType,
        ext: &mut DeferredEnumExtension,
    ) -> Result<()> {
        type_.directives.append(&mut ext.directives);

        for ext_val in ext.values.drain(..) {
            // Error if this value is already defined.
            if let Some(existing_value) = type_.values.get(ext_val.name.as_str()) {
                return Err(SchemaBuildError::DuplicateEnumValueDefinition {
                    enum_name: ext.name.to_string(),
                    enum_def_location: type_.def_location.clone(),
                    value_def1: existing_value.def_location.clone(),
                    value_def2: ext_val.def_location.clone(),
                });
            }
            type_.values.insert(ext_val.name.clone(), ext_val);
        }

        Ok(())
    }

    fn build_extension_from_ast(
        &self,
        ext_srcloc: &loc::SourceLocation,
        ext: &ast::EnumTypeExtension<'_>,
        source_map: &ast::SourceMap<'_>,
    ) -> DeferredEnumExtension {
        let ext_name = ext.name.value.as_ref().to_string();
        let directives = DirectiveAnnotationBuilder::from_ast(
            ext_srcloc,
            source_map,
            &ext.directives,
        );

        let values = ext.values.iter().map(|ext_val| {
            let ext_val_srcloc =
                ext_srcloc.with_span(ext_val.span, source_map);
            EnumValue {
                def_location: ext_val_srcloc.to_owned(),
                description: ext_val.description.as_ref()
                    .map(|d| d.value.to_string()),
                directives: DirectiveAnnotationBuilder::from_ast(
                    &ext_val_srcloc,
                    source_map,
                    &ext_val.directives,
                ),
                name: ext_val.name.value.as_ref().to_string(),
                type_ref: NamedGraphQLTypeRef::new(
                    ext_name.as_str(),
                    ext_val_srcloc,
                ),
            }
        }).collect();

        DeferredEnumExtension {
            directives,
            name: ext_name,
            srcloc: ext_srcloc.clone(),
            values,
        }
    }

    pub(crate) fn visit_type_def(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        def: &ast::EnumTypeDefinition<'_>,
        source_map: &ast::SourceMap<'_>,
    ) -> Result<()> {
        let enumdef_srcloc = loc::SourceLocation::from_schema_span(
            file_path,
            def.span,
            source_map,
        );

        let directives = DirectiveAnnotationBuilder::from_ast(
            &enumdef_srcloc,
            source_map,
            &def.directives,
        );

        let mut enum_values = IndexMap::<String, EnumValue>::new();
        for enum_value in &def.values {
            let value_name = enum_value.name.value.as_ref().to_string();
            let valuedef_srcloc =
                enumdef_srcloc.with_span(enum_value.span, source_map);
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
                        description: enum_value.description.as_ref()
                            .map(|d| d.value.to_string()),
                        directives: DirectiveAnnotationBuilder::from_ast(
                            &valuedef_srcloc,
                            source_map,
                            &enum_value.directives,
                        ),
                        type_ref: NamedGraphQLTypeRef::new(
                            def.name.value.as_ref(),
                            valuedef_srcloc,
                        ),
                        name: value_name,
                    },
                );
            }
        }

        if enum_values.is_empty() {
            return Err(SchemaBuildError::EnumWithNoVariants {
                type_name: def.name.value.as_ref().to_string(),
                location: enumdef_srcloc,
            });
        }

        types_builder.add_new_type(
            def.name.value.as_ref(),
            &enumdef_srcloc.to_owned(),
            GraphQLType::Enum(EnumType {
                def_location: enumdef_srcloc,
                description: def.description.as_ref()
                    .map(|d| d.value.to_string()),
                directives,
                name: def.name.value.as_ref().to_string(),
                values: enum_values,
            }.into()),
        )
    }

    pub(crate) fn visit_type_extension(
        &mut self,
        types_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        ext: &ast::EnumTypeExtension<'_>,
        source_map: &ast::SourceMap<'_>,
    ) -> Result<()> {
        let ext_srcloc = loc::SourceLocation::from_schema_span(
            file_path,
            ext.span,
            source_map,
        );
        let type_name = ext.name.value.as_ref();
        let mut deferred = self.build_extension_from_ast(
            &ext_srcloc, ext, source_map,
        );

        match types_builder.get_type_mut(type_name) {
            Some(GraphQLType::Enum(enum_type)) =>
                self.merge_extension(enum_type, &mut deferred),

            Some(non_enum_type) =>
                Err(SchemaBuildError::InvalidExtensionType {
                    schema_type: non_enum_type.clone(),
                    extension_location: ext_srcloc,
                }),

            None => {
                self.extensions.push(deferred);
                Ok(())
            },
        }
    }
}

impl TypeBuilder for EnumTypeBuilder {
    fn finalize(mut self, types_builder: &mut TypesMapBuilder) -> Result<()> {
        while let Some(mut ext) = self.extensions.pop() {
            let type_name = ext.name.as_str();
            match types_builder.get_type_mut(type_name) {
                Some(GraphQLType::Enum(enum_type)) =>
                    self.merge_extension(enum_type, &mut ext)?,

                Some(non_enum_type) =>
                    return Err(SchemaBuildError::InvalidExtensionType {
                        schema_type: non_enum_type.clone(),
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
