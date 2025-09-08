use crate::ast;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::schema::SchemaBuildError;
use crate::types::TypesMapBuilder;
use crate::types::Field;
use crate::types::NamedTypeAnnotation;
use crate::types::NamedGraphQLTypeRef;
use crate::types::TypeAnnotation;
use crate::types::InputField;
use crate::types::Parameter;
use crate::types::NamedDirectiveRef;
use crate::Value;
use indexmap::IndexMap;
use std::path::Path;

type Result<T> = std::result::Result<T, SchemaBuildError>;

pub trait TypeBuilder: Sized {
    type AstTypeDef;
    type AstTypeExtension;

    fn finalize(self, types_map_builder: &mut TypesMapBuilder) -> Result<()>;

    fn visit_type_def(
        &mut self,
        types_map_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        def: &Self::AstTypeDef,
    ) -> Result<()>;

    fn visit_type_extension(
        &mut self,
        types_map_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        def: Self::AstTypeExtension,
    ) -> Result<()>;
}

pub struct TypeBuilderHelpers;
impl TypeBuilderHelpers {
    pub fn directive_refs_from_ast(
        annotated_item_srcloc: &loc::SourceLocation,
        directives: &[ast::operation::Directive],
    ) -> Vec<DirectiveAnnotation> {
        directives.iter().map(|ast_annot| {
            let annot_srcloc =
                annotated_item_srcloc.with_ast_position(&ast_annot.position);
            let mut args = IndexMap::new();
            for (arg_name, ast_arg) in ast_annot.arguments.iter() {
                args.insert(
                    arg_name.to_string(),
                    Value::from_ast(ast_arg, &annot_srcloc),
                );
            }
            DirectiveAnnotation {
                args,
                directive_ref: NamedDirectiveRef::new(
                    &ast_annot.name,
                    annot_srcloc,
                ),
            }
        }).collect()
    }

    pub fn inputobject_fields_from_ast(
        inputobj_def_location: &loc::SourceLocation,
        type_name: &str,
        input_fields: &[ast::schema::InputValue],
    ) -> Result<IndexMap<String, InputField>> {
        let mut field_map = IndexMap::new();
        for field in input_fields {
            let fielddef_srcloc =
                inputobj_def_location.with_ast_position(&field.position);

            // The input field must not have a name which begins with the
            // characters "__" (two underscores).
            //
            // https://spec.graphql.org/October2021/#sel-IAHhBXDDBDCAACCTx5b
            if field.name.starts_with("__") {
                return Err(SchemaBuildError::InvalidDunderPrefixedFieldName {
                    location: fielddef_srcloc,
                    field_name: field.name.to_string(),
                    type_name: type_name.to_string(),
                });
            }

            field_map.insert(field.name.to_string(), InputField {
                description: field.description.to_owned(),
                directives: TypeBuilderHelpers::directive_refs_from_ast(
                    &fielddef_srcloc,
                    &field.directives,
                ),
                name: field.name.to_string(),
                type_annotation: TypeAnnotation::from_ast_type(
                    // Unfortunately, graphql_parser doesn't give us a location for
                    // the actual field-definition's type.
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
        fields: &[ast::schema::Field],
    ) -> Result<IndexMap<String, Field>> {
        let mut field_map = IndexMap::from([
            ("__typename".to_string(), Field {
                def_location: loc::SourceLocation::GraphQLBuiltIn,
                description: None,
                directives: vec![],
                name: "__typename".to_string(),
                parameters: IndexMap::new(),
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
            let fielddef_srcloc = obj_def_location.with_ast_position(&field.position);

            // https://spec.graphql.org/October2021/#sel-IAHZhCFDBDCAACCTl4L
            if field.name.starts_with("__") {
                return Err(SchemaBuildError::InvalidDunderPrefixedFieldName {
                    location: fielddef_srcloc,
                    field_name: field.name.to_string(),
                    type_name: type_name.to_string(),
                });
            }

            let mut params = IndexMap::new();
            for param in &field.arguments {
                let param_srcloc =
                    obj_def_location.with_ast_position(&param.position);

                // https://spec.graphql.org/October2021/#sel-KAHZhCFDBHBBCAACCTlrG
                if param.name.starts_with("__") {
                    return Err(SchemaBuildError::InvalidDunderPrefixedParamName {
                        location: param_srcloc,
                        field_name: field.name.to_string(),
                        param_name: param.name.to_string(),
                        type_name: type_name.to_string(),
                    });
                }

                params.insert(param.name.to_string(), Parameter {
                    def_location: param_srcloc.to_owned(),
                    default_value: param.default_value.as_ref().map(
                        |val| Value::from_ast(val, &param_srcloc)
                    ),
                    name: param.name.to_owned(),
                    type_annotation: TypeAnnotation::from_ast_type(
                        &param_srcloc,
                        &param.value_type,
                    ),
                });
            }

            field_map.insert(field.name.to_string(), Field {
                def_location: fielddef_srcloc.to_owned(),
                description: field.description.to_owned(),
                directives: TypeBuilderHelpers::directive_refs_from_ast(
                    &fielddef_srcloc,
                    &field.directives,
                ),
                name: field.name.to_string(),
                parameters: params,
                type_annotation: TypeAnnotation::from_ast_type(
                    // Unfortunately, graphql_parser doesn't give us a location for
                    // the actual field-definition's type.
                    &fielddef_srcloc,
                    &field.field_type,
                ),
            });
        }

        Ok(field_map)
    }
}
