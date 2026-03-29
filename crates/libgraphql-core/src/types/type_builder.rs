use crate::ast;
use crate::DirectiveAnnotationBuilder;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::Field;
use crate::types::InputField;
use crate::types::NamedGraphQLTypeRef;
use crate::types::NamedTypeAnnotation;
use crate::types::Parameter;
use crate::types::TypeAnnotation;
use crate::types::TypesMapBuilder;
use crate::Value;
use indexmap::IndexMap;
type Result<T> = std::result::Result<T, SchemaBuildError>;

pub trait TypeBuilder: Sized {
    fn finalize(self, types_map_builder: &mut TypesMapBuilder) -> Result<()>;
}

pub struct TypeBuilderHelpers;
impl TypeBuilderHelpers {
    pub fn inputobject_fields_from_ast(
        inputobj_def_location: &loc::SourceLocation,
        type_name: &str,
        input_fields: &[ast::InputValueDefinition<'_>],
        source_map: &ast::SourceMap<'_>,
    ) -> Result<IndexMap<String, InputField>> {
        let mut field_map = IndexMap::new();
        for field in input_fields {
            let fielddef_srcloc =
                inputobj_def_location.with_span(field.span, source_map);
            let field_name = field.name.value.as_ref();

            // The input field must not have a name which begins with the
            // characters "__" (two underscores).
            //
            // https://spec.graphql.org/October2021/#sel-IAHhBXDDBDCAACCTx5b
            if field_name.starts_with("__") {
                return Err(SchemaBuildError::InvalidDunderPrefixedFieldName {
                    location: fielddef_srcloc,
                    field_name: field_name.to_string(),
                    type_name: type_name.to_string(),
                });
            }

            field_map.insert(field_name.to_string(), InputField {
                description: field.description.as_ref()
                    .map(|d| d.value.to_string()),
                directives: DirectiveAnnotationBuilder::from_ast(
                    &fielddef_srcloc,
                    source_map,
                    &field.directives,
                ),
                name: field_name.to_string(),
                parent_type: NamedGraphQLTypeRef::new(
                    type_name,
                    inputobj_def_location.to_owned(),
                ),
                type_annotation: TypeAnnotation::from_ast_type(
                    &fielddef_srcloc,
                    &field.value_type,
                ),
                def_location: fielddef_srcloc,
            });
        }
        Ok(field_map)
    }

    pub fn object_fielddefs_from_ast(
        obj_def_location: &loc::SourceLocation,
        type_name: &str,
        fields: &[ast::FieldDefinition<'_>],
        source_map: &ast::SourceMap<'_>,
    ) -> Result<IndexMap<String, Field>> {
        let mut field_map = IndexMap::from([
            ("__typename".to_string(), Field {
                def_location: loc::SourceLocation::GraphQLBuiltIn,
                description: None,
                directives: vec![],
                name: "__typename".to_string(),
                parameters: IndexMap::new(),
                parent_type: NamedGraphQLTypeRef::new(
                    type_name,
                    obj_def_location.to_owned(),
                ),
                type_annotation: TypeAnnotation::Named(
                    NamedTypeAnnotation {
                        nullable: false,
                        type_ref: NamedGraphQLTypeRef::new(
                            "String",
                            obj_def_location.to_owned(),
                        ),
                    },
                ),
            }),
        ]);

        for field in fields {
            let field_name = field.name.value.as_ref();
            let fielddef_srcloc =
                obj_def_location.with_span(field.span, source_map);

            // https://spec.graphql.org/October2021/#sel-IAHZhCFDBDCAACCTl4L
            if field_name.starts_with("__") {
                return Err(SchemaBuildError::InvalidDunderPrefixedFieldName {
                    location: fielddef_srcloc,
                    field_name: field_name.to_string(),
                    type_name: type_name.to_string(),
                });
            }

            let mut params = IndexMap::new();
            for param in &field.parameters {
                let param_name = param.name.value.as_ref();
                let param_srcloc =
                    obj_def_location.with_span(param.span, source_map);

                // https://spec.graphql.org/October2021/#sel-KAHZhCFDBHBBCAACCTlrG
                if param_name.starts_with("__") {
                    return Err(SchemaBuildError::InvalidDunderPrefixedParamName {
                        location: param_srcloc,
                        field_name: field_name.to_string(),
                        param_name: param_name.to_string(),
                        type_name: type_name.to_string(),
                    });
                }

                params.insert(param_name.to_string(), Parameter {
                    def_location: param_srcloc.to_owned(),
                    default_value: param.default_value.as_ref().map(
                        |val| Value::from_ast(val, &param_srcloc)
                    ),
                    name: param_name.to_owned(),
                    type_annotation: TypeAnnotation::from_ast_type(
                        &param_srcloc,
                        &param.value_type,
                    ),
                });
            }

            field_map.insert(field_name.to_string(), Field {
                def_location: fielddef_srcloc.to_owned(),
                description: field.description.as_ref()
                    .map(|d| d.value.to_string()),
                directives: DirectiveAnnotationBuilder::from_ast(
                    &fielddef_srcloc,
                    source_map,
                    &field.directives,
                ),
                name: field_name.to_string(),
                parameters: params,
                parent_type: NamedGraphQLTypeRef::new(
                    type_name,
                    obj_def_location.to_owned(),
                ),
                type_annotation: TypeAnnotation::from_ast_type(
                    &fielddef_srcloc,
                    &field.field_type,
                ),
            });
        }

        Ok(field_map)
    }
}
