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
use std::collections::BTreeMap;
use std::path::Path;

type Result<T> = std::result::Result<T, SchemaBuildError>;

pub trait TypeBuilder: Sized {
    type AstTypeDef;
    type AstTypeExtension;

    fn finalize(self, types_map_builder: &mut TypesMapBuilder) -> Result<()>;

    fn visit_type_def(
        &mut self,
        types_map_builder: &mut TypesMapBuilder,
        file_path: &Path,
        def: Self::AstTypeDef,
    ) -> Result<()>;

    fn visit_type_extension(
        &mut self,
        types_map_builder: &mut TypesMapBuilder,
        file_path: &Path,
        def: Self::AstTypeExtension,
    ) -> Result<()>;
}

pub struct TypeBuilderHelpers;
impl TypeBuilderHelpers {
    pub fn directive_refs_from_ast(
        file_path: &Path,
        directives: &[ast::operation::Directive],
    ) -> Vec<DirectiveAnnotation> {
        directives.iter().map(|ast_annot| {
            let annot_file_pos = loc::FilePosition::from_pos(
                file_path,
                ast_annot.position,
            );
            let mut args = BTreeMap::new();
            for (arg_name, ast_arg) in ast_annot.arguments.iter() {
                args.insert(
                    arg_name.to_string(),
                    Value::from_ast(ast_arg, annot_file_pos.clone()),
                );
            }
            DirectiveAnnotation {
                args,
                directive_ref: NamedDirectiveRef::new(
                    &ast_annot.name,
                    annot_file_pos.into(),
                ),
            }
        }).collect()
    }

    pub fn inputobject_fields_from_ast(
        schema_def_location: &loc::FilePosition,
        type_name: &str,
        input_fields: &[ast::schema::InputValue],
    ) -> Result<BTreeMap<String, InputField>> {
        let mut field_map = BTreeMap::new();
        for field in input_fields {
            let field_def_pos = loc::FilePosition::from_pos(
                schema_def_location.file(),
                field.position,
            );

            // The input field must not have a name which begins with the
            // characters "__" (two underscores).
            //
            // https://spec.graphql.org/October2021/#sel-IAHhBXDDBDCAACCTx5b
            if field.name.starts_with("__") {
                return Err(SchemaBuildError::InvalidDunderPrefixedFieldName {
                    def_location: field_def_pos.into(),
                    field_name: field.name.to_string(),
                    type_name: type_name.to_string(),
                });
            }

            field_map.insert(field.name.to_string(), InputField {
                def_location: field_def_pos.to_owned().into(),
                directives: TypeBuilderHelpers::directive_refs_from_ast(
                    schema_def_location.file().as_path(),
                    &field.directives,
                ),
                name: field.name.to_string(),
                type_annotation: TypeAnnotation::from_ast_type(
                    // Unfortunately, graphql_parser doesn't give us a location for
                    // the actual field-definition's type.
                    &field_def_pos.into(),
                    &field.value_type,
                ),
            });
        }
        Ok(field_map)
    }

    pub fn object_fielddefs_from_ast(
        ref_location: &loc::FilePosition,
        type_name: &str,
        fields: &[ast::schema::Field],
    ) -> Result<BTreeMap<String, Field>> {
        let mut field_map = BTreeMap::from([
            ("__typename".to_string(), Field {
                def_location: loc::SchemaDefLocation::GraphQLBuiltIn,
                directives: vec![],
                name: "__typename".to_string(),
                parameters: BTreeMap::new(),
                type_annotation: TypeAnnotation::Named(
                    NamedTypeAnnotation {
                        nullable: false,
                        type_ref: NamedGraphQLTypeRef::new(
                            "String",
                            loc::SchemaDefLocation::GraphQLBuiltIn,
                        ),
                    },
                ),
            }),
        ]);

        for field in fields {
            let field_def_position = loc::FilePosition::from_pos(
                *ref_location.file.clone(),
                field.position,
            );

            // https://spec.graphql.org/October2021/#sel-IAHZhCFDBDCAACCTl4L
            if field.name.starts_with("__") {
                return Err(SchemaBuildError::InvalidDunderPrefixedFieldName {
                    def_location: field_def_position.into(),
                    field_name: field.name.to_string(),
                    type_name: type_name.to_string(),
                });
            }

            let mut params = BTreeMap::new();
            for param in &field.arguments {
                let input_val_position = loc::FilePosition::from_pos(
                    *ref_location.file.clone(),
                    param.position,
                );

                // https://spec.graphql.org/October2021/#sel-KAHZhCFDBHBBCAACCTlrG
                if param.name.starts_with("__") {
                    return Err(SchemaBuildError::InvalidDunderPrefixedParamName {
                        def_location: input_val_position.into(),
                        field_name: field.name.to_string(),
                        param_name: param.name.to_string(),
                        type_name: type_name.to_string(),
                    });
                }

                params.insert(param.name.to_string(), Parameter {
                    def_location: input_val_position.clone().into(),
                    default_value: param.default_value.as_ref().map(
                        |val| Value::from_ast(val, input_val_position.clone())
                    ),
                    name: param.name.to_owned(),
                    type_annotation: TypeAnnotation::from_ast_type(
                        &input_val_position.into(),
                        &param.value_type,
                    ),
                });
            }

            field_map.insert(field.name.to_string(), Field {
                def_location: field_def_position.to_owned().into(),
                directives: TypeBuilderHelpers::directive_refs_from_ast(
                    ref_location.file.as_path(),
                    &field.directives,
                ),
                name: field.name.to_string(),
                parameters: params,
                type_annotation: TypeAnnotation::from_ast_type(
                    // Unfortunately, graphql_parser doesn't give us a location for
                    // the actual field-definition's type.
                    &field_def_position.clone().into(),
                    &field.field_type,
                ),
            });
        }

        Ok(field_map)
    }
}
