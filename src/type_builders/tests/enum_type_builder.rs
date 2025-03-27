use crate::ast;
use crate::loc;
use crate::type_builders::EnumTypeBuilder;
use crate::schema_builder::SchemaBuildError;
#[cfg(test)]
use crate::type_builders::TestBuildFromAst;
use crate::type_builders::TypeBuilder;
use crate::types::DirectiveAnnotation;
use crate::types::EnumVariant;
use crate::types::EnumType;
use crate::types::GraphQLType;
use crate::types::NamedDirectiveRef;
use crate::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

fn mkast_empty_enum(
    name: &str,
    values: &[&str],
) -> ast::schema::EnumType {
    ast::schema::EnumType {
        description: None,
        directives: vec![],
        name: name.to_string(),
        position: ast::AstPos {
            column: 1,
            line: 2,
        },
        values: values.iter().map(|name| mkast_enum_value(*name)).collect(),
    }
}

fn mkast_enum_value(name: &str) -> ast::schema::EnumValue {
    ast::schema::EnumValue {
        description: None,
        directives: vec![],
        name: name.to_string(),
        position: ast::AstPos {
            column: 2,
            line: 2,
        },
    }
}

fn mktype_empty_enum(name: &str, variant_names: &[&str]) -> EnumType {
    let mut variants = BTreeMap::new();
    for name in variant_names.iter() {
        variants.insert(name.to_string(), mktype_enum_variant(name));
    }

    EnumType {
        def_location: loc::FilePosition {
            col: 1,
            file: PathBuf::from("str://0"),
            line: 2,
        },
        directives: vec![],
        name: name.to_string(),
        variants,
    }
}

fn mktype_enum_variant(name: &str) -> EnumVariant {
    let file_path = PathBuf::from("str://0");
    EnumVariant {
        def_location: loc::FilePosition {
            col: 2,
            file: file_path.to_path_buf(),
            line: 2,
        },
        directives: DirectiveAnnotation::from_ast(
            file_path.as_path(),
            &[],
        ),
        name: name.to_string(),
    }
}

mod visit_type_def {
    use super::*;

    mod type_directives {
        use super::*;

        #[test]
        fn no_directives() -> Result<()> {
            let type_name = "TestEnum";
            let value1_name = "Variant1";

            let enum_def_ast = mkast_empty_enum(type_name, &[value1_name]);
            let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
                ast_def: vec![enum_def_ast],
                ast_ext: vec![],
                file_path: PathBuf::from("str://0"),
            })?;

            let expected_type = mktype_empty_enum(type_name, &[value1_name]);
            assert_eq!(
                types.get(type_name),
                Some(&GraphQLType::Enum(expected_type)),
            );

            Ok(())
        }

        #[test]
        fn directive_without_args() -> Result<()> {
            let type_name = "TestEnum";
            let value1_name = "Variant1";
            let file_path = PathBuf::from("str://0");

            let mut enum_def_ast = mkast_empty_enum(type_name, &[value1_name]);
            enum_def_ast.directives.push(ast::operation::Directive {
                arguments: vec![],
                name: "deprecated".to_string(),
                position: ast::AstPos {
                    column: 1,
                    line: 1,
                },
            });

            let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
                ast_def: vec![enum_def_ast.clone()],
                ast_ext: vec![],
                file_path: file_path.clone(),
            })?;

            let mut expected_type = mktype_empty_enum(type_name, &[value1_name]);
            expected_type.directives.push(DirectiveAnnotation {
                args: BTreeMap::new(),
                directive_ref: NamedDirectiveRef::new(
                    "deprecated".to_string(),
                    loc::FilePosition::from_pos(
                        file_path.to_path_buf(),
                        enum_def_ast.directives.get(0).unwrap().position,
                    ),
                ),
            });

            assert_eq!(
                types.get(type_name),
                Some(&GraphQLType::Enum(expected_type)),
            );

            Ok(())
        }

        #[test]
        fn directive_with_arg() -> Result<()> {
            let directive_name = "some_custom_directive";
            let directive_arg1_name = "arg1";
            let arg1_ast_number: ast::Number = 42.into();
            let arg1_ast_value = ast::Value::Int(arg1_ast_number.clone());
            let arg1_value = Value::Int(arg1_ast_number);
            let file_path = PathBuf::from("str://0");
            let type_name = "TestEnum";
            let value1_name = "Variant1";

            let mut enum_def_ast = mkast_empty_enum(type_name, &[value1_name]);
            enum_def_ast.directives.push(ast::operation::Directive {
                arguments: vec![
                    (directive_arg1_name.to_string(), arg1_ast_value),
                ],
                name: directive_name.to_string(),
                position: ast::AstPos {
                    column: 1,
                    line: 1,
                },
            });

            let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
                ast_def: vec![enum_def_ast.clone()],
                ast_ext: vec![],
                file_path: file_path.to_path_buf(),
            })?;

            let mut expected_type = mktype_empty_enum(type_name, &[value1_name]);
            expected_type.directives.push(DirectiveAnnotation {
                args: BTreeMap::from([
                    (directive_arg1_name.to_string(), arg1_value),
                ]),
                directive_ref: NamedDirectiveRef::new(
                    directive_name.to_string(),
                    loc::FilePosition::from_pos(
                        file_path.to_path_buf(),
                        enum_def_ast.directives.get(0).unwrap().position,
                    ),
                ),
            });

            assert_eq!(
                types.get(type_name),
                Some(&GraphQLType::Enum(expected_type)),
            );

            Ok(())
        }
    }

    mod variants {
        use super::*;

        #[test]
        fn no_variants_is_an_error() -> Result<()> {
            let file_path = PathBuf::from("str://0");
            let type_name = "TestEnum";

            let enum_def_ast = mkast_empty_enum(type_name, &[]);
            let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
                ast_def: vec![enum_def_ast.clone()],
                ast_ext: vec![],
                file_path: file_path.to_path_buf(),
            });

            assert_eq!(types.unwrap_err(), SchemaBuildError::EnumWithNoVariants {
                type_name: type_name.to_string(),
                location: loc::FilePosition::from_pos(file_path, enum_def_ast.position),
            });

            Ok(())
        }

        #[test]
        fn one_variant() -> Result<()> {
            let file_path = PathBuf::from("str://0");
            let type_name = "TestEnum";
            let variant1_name = "Variant1";

            let enum_def_ast = mkast_empty_enum(type_name, &[variant1_name]);
            let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
                ast_def: vec![enum_def_ast.clone()],
                ast_ext: vec![],
                file_path: file_path.to_path_buf(),
            })?;

            let expected_type = mktype_empty_enum(type_name, &[variant1_name]);
            assert_eq!(
                types.get(type_name),
                Some(&GraphQLType::Enum(expected_type)),
            );

            Ok(())
        }

        #[test]
        fn multiple_variants() -> Result<()> {
            let file_path = PathBuf::from("str://0");
            let type_name = "TestEnum";
            let variant1_name = "Variant1";
            let variant2_name = "Variant2";
            let variant3_name = "Variant3";

            let enum_def_ast = mkast_empty_enum(type_name, &[
                variant1_name,
                variant2_name,
                variant3_name,
            ]);
            let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
                ast_def: vec![enum_def_ast.clone()],
                ast_ext: vec![],
                file_path: file_path.to_path_buf(),
            })?;

            let expected_type = mktype_empty_enum(type_name, &[
                variant1_name,
                variant2_name,
                variant3_name,
            ]);
            assert_eq!(
                types.get(type_name),
                Some(&GraphQLType::Enum(expected_type)),
            );

            Ok(())
        }

        #[test]
        fn two_enums_with_same_variant_names() -> Result<()> {
            let file_path = PathBuf::from("str://0");
            let type1_name = "TestEnum1";
            let type2_name = "TestEnum2";
            let variant1_name = "Variant1";
            let variant2_name = "Variant2";
            let variant3_name = "Variant3";

            let enum1_def_ast = mkast_empty_enum(type1_name, &[
                variant1_name,
                variant2_name,
                variant3_name,
            ]);
            let enum2_def_ast = mkast_empty_enum(type2_name, &[
                variant1_name,
                variant2_name,
                variant3_name,
            ]);
            let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
                ast_def: vec![
                    enum1_def_ast.clone(),
                    enum2_def_ast.clone(),
                ],
                ast_ext: vec![],
                file_path: file_path.to_path_buf(),
            })?;

            let expected_type1 = mktype_empty_enum(type1_name, &[
                variant1_name,
                variant2_name,
                variant3_name,
            ]);
            assert_eq!(
                types.get(type1_name),
                Some(&GraphQLType::Enum(expected_type1)),
            );

            let expected_type2 = mktype_empty_enum(type2_name, &[
                variant1_name,
                variant2_name,
                variant3_name,
            ]);
            assert_eq!(
                types.get(type2_name),
                Some(&GraphQLType::Enum(expected_type2)),
            );

            Ok(())
        }

        #[test]
        fn variant_directive_without_args() -> Result<()> {
            let type_name = "TestEnum";
            let value1_name = "Variant1";
            let value1_directive = ast::operation::Directive {
                arguments: vec![],
                name: "deprecated".to_string(),
                position: ast::AstPos {
                    column: 1,
                    line: 1,
                },
            };
            let file_path = PathBuf::from("str://0");

            let mut enum_def_ast = mkast_empty_enum(type_name, &[value1_name]);
            enum_def_ast.values
                .get_mut(0)
                .unwrap()
                .directives
                .push(value1_directive.clone());

            let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
                ast_def: vec![enum_def_ast.clone()],
                ast_ext: vec![],
                file_path: file_path.clone(),
            })?;

            let mut expected_type = mktype_empty_enum(type_name, &[value1_name]);
            let variant1 = expected_type.variants.get_mut(value1_name).unwrap();
            variant1.directives.push(DirectiveAnnotation {
                args: BTreeMap::new(),
                directive_ref: NamedDirectiveRef::new(
                    "deprecated".to_string(),
                    loc::FilePosition::from_pos(
                        file_path.to_path_buf(),
                        value1_directive.position,
                    ),
                ),
            });

            assert_eq!(
                types.get(type_name),
                Some(&GraphQLType::Enum(expected_type)),
            );

            Ok(())
        }

        #[test]
        fn variant_directive_with_arg() -> Result<()> {
            let type_name = "TestEnum";
            let value1_name = "Variant1";
            let directive_name = "some_custom_directive";
            let arg1_name = "arg1";
            let arg1_ast_number: ast::Number = 42.into();
            let arg1_ast_value = ast::Value::Int(arg1_ast_number.clone());
            let arg1_value = Value::Int(arg1_ast_number);
            let directive = ast::operation::Directive {
                arguments: vec![
                    (arg1_name.to_string(), arg1_ast_value),
                ],
                name: directive_name.to_string(),
                position: ast::AstPos {
                    column: 2,
                    line: 2,
                },
            };
            let file_path = PathBuf::from("str://0");

            let mut enum_def_ast = mkast_empty_enum(type_name, &[value1_name]);
            enum_def_ast.values
                .get_mut(0)
                .unwrap()
                .directives
                .push(directive.clone());

            let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
                ast_def: vec![enum_def_ast.clone()],
                ast_ext: vec![],
                file_path: file_path.to_path_buf(),
            })?;

            let mut expected_type = mktype_empty_enum(type_name, &[value1_name]);
            let variant1 = expected_type.variants.get_mut(value1_name).unwrap();
            variant1.directives.push(DirectiveAnnotation {
                args: BTreeMap::from([
                    (arg1_name.to_string(), arg1_value),
                ]),
                directive_ref: NamedDirectiveRef::new(
                    directive_name.to_string(),
                    loc::FilePosition::from_pos(
                        file_path.to_path_buf(),
                        directive.position,
                    ),
                ),
            });

            assert_eq!(
                types.get(type_name),
                Some(&GraphQLType::Enum(expected_type)),
            );

            Ok(())
        }
    }
}

mod visit_type_extension {
    use super::*;

    // TODO
}
