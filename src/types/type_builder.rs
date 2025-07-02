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
        schema_def_location: &loc::SchemaDefLocation,
        input_fields: &[ast::schema::InputValue],
    ) -> Result<BTreeMap<String, InputField>> {
        Ok(input_fields.iter().map(|input_field| {
            (input_field.name.to_string(), InputField {
                def_location: schema_def_location.clone(),
            })
        }).collect())
    }

    pub fn object_fielddefs_from_ast(
        ref_location: &loc::FilePosition,
        fields: &[ast::schema::Field],
    ) -> BTreeMap<String, Field> {
        let mut field_map = BTreeMap::from([
            ("__typename".to_string(), Field {
                def_location: loc::SchemaDefLocation::GraphQLBuiltIn,
                directives: vec![],
                name: "__typename".to_string(),
                params: BTreeMap::new(),
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
        fields.iter().for_each(|field| {
            let field_def_position = loc::FilePosition::from_pos(
                *ref_location.file.clone(),
                field.position,
            );

            field_map.insert(field.name.to_string(), Field {
                def_location: field_def_position.to_owned().into(),
                directives: TypeBuilderHelpers::directive_refs_from_ast(
                    ref_location.file.as_path(),
                    &field.directives,
                ),
                name: field.name.to_string(),
                params: field.arguments.iter().map(|input_val| {
                    let input_val_position = loc::FilePosition::from_pos(
                        *ref_location.file.clone(),
                        input_val.position,
                    );

                    (input_val.name.to_string(), Parameter {
                        def_location: input_val_position.clone().into(),
                        default_value: input_val.default_value.as_ref().map(
                            |val| Value::from_ast(val, input_val_position.clone())
                        ),
                        name: input_val.name.to_owned(),
                        type_ref: TypeAnnotation::from_ast_type(
                            &input_val_position.into(),
                            &input_val.value_type,
                        ),
                    })
                }).collect(),
                type_annotation: TypeAnnotation::from_ast_type(
                    // Unfortunately, graphql_parser doesn't give us a location for
                    // the actual field-definition's type.
                    &field_def_position.clone().into(),
                    &field.field_type,
                ),
            });
        });
        field_map
    }
}
