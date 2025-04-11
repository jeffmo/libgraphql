use crate::ast;
use crate::loc;
use crate::NamedRef;
use crate::types::EnumTypeBuilder;
use crate::types::Field;
use crate::schema_builder::SchemaBuildError;
use crate::types::TestBuildFromAst;
use crate::types::TypeBuilder;
use crate::types::TypesMapBuilder;
use crate::types::DirectiveAnnotation;
use crate::types::enum_type;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeRef;
use crate::types::NamedDirectiveRef;
use crate::types::ObjectType;
use crate::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[test]
fn enum_with_no_type_directives() -> Result<()> {
    let type_name = "TestEnum";
    let value1_name = "Variant1";

    let enum_def_ast = ast::test_helpers::mk_enum(
        type_name,
        &[value1_name],
    );
    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![enum_def_ast],
        ast_ext_after: vec![],
        ast_ext_before: vec![],
        file_path: PathBuf::from("str://0"),
        types_after: vec![],
        types_before: vec![],
    })?;

    let expected_type = enum_type::test_helpers::mk_enum(type_name, &[value1_name]);
    assert_eq!(
        types.get(type_name),
        Some(&GraphQLType::Enum(expected_type)),
    );

    Ok(())
}

#[test]
fn enum_with_type_directive_no_args() -> Result<()> {
    let type_name = "TestEnum";
    let value1_name = "Variant1";
    let file_path = PathBuf::from("str://0");

    let mut enum_def_ast = ast::test_helpers::mk_enum(
        type_name,
        &[value1_name],
    );
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
        ast_ext_after: vec![],
        ast_ext_before: vec![],
        file_path: file_path.clone(),
        types_after: vec![],
        types_before: vec![],
    })?;

    let mut expected_type = enum_type::test_helpers::mk_enum(type_name, &[value1_name]);
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
fn enum_with_single_arg_type_directive() -> Result<()> {
    let directive_name = "some_custom_directive";
    let directive_arg1_name = "arg1";
    let arg1_ast_number: ast::Number = 42.into();
    let arg1_ast_value = ast::Value::Int(arg1_ast_number.clone());
    let arg1_value = Value::Int(arg1_ast_number);
    let file_path = PathBuf::from("str://0");
    let type_name = "TestEnum";
    let value1_name = "Variant1";

    let mut enum_def_ast = ast::test_helpers::mk_enum(
        type_name,
        &[value1_name],
    );
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
        ast_ext_after: vec![],
        ast_ext_before: vec![],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![],
    })?;

    let mut expected_type = enum_type::test_helpers::mk_enum(type_name, &[value1_name]);
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

#[test]
fn enum_with_no_values_is_an_error() -> Result<()> {
    let file_path = PathBuf::from("str://0");
    let type_name = "TestEnum";

    let enum_def_ast = ast::test_helpers::mk_enum(type_name, &[]);
    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![enum_def_ast.clone()],
        ast_ext_after: vec![],
        ast_ext_before: vec![],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![],
    });

    assert_eq!(types.unwrap_err(), SchemaBuildError::EnumWithNoVariants {
        type_name: type_name.to_string(),
        location: loc::FilePosition::from_pos(file_path, enum_def_ast.position),
    });

    Ok(())
}

#[test]
fn enum_with_one_value() -> Result<()> {
    let file_path = PathBuf::from("str://0");
    let type_name = "TestEnum";
    let value1_name = "Variant1";

    let enum_def_ast = ast::test_helpers::mk_enum(
        type_name,
        &[value1_name],
    );
    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![enum_def_ast.clone()],
        ast_ext_after: vec![],
        ast_ext_before: vec![],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![],
    })?;

    let expected_type = enum_type::test_helpers::mk_enum(type_name, &[value1_name]);
    assert_eq!(
        types.get(type_name),
        Some(&GraphQLType::Enum(expected_type)),
    );

    Ok(())
}

#[test]
fn enum_with_multiple_values() -> Result<()> {
    let file_path = PathBuf::from("str://0");
    let type_name = "TestEnum";
    let value1_name = "Variant1";
    let value2_name = "Variant2";
    let value3_name = "Variant3";

    let enum_def_ast = ast::test_helpers::mk_enum(type_name, &[
        value1_name,
        value2_name,
        value3_name,
    ]);
    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![enum_def_ast.clone()],
        ast_ext_after: vec![],
        ast_ext_before: vec![],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![],
    })?;

    let expected_type = enum_type::test_helpers::mk_enum(type_name, &[
        value1_name,
        value2_name,
        value3_name,
    ]);
    assert_eq!(
        types.get(type_name),
        Some(&GraphQLType::Enum(expected_type)),
    );

    Ok(())
}

#[test]
fn two_enums_with_same_value_names() -> Result<()> {
    let file_path = PathBuf::from("str://0");
    let type1_name = "TestEnum1";
    let type2_name = "TestEnum2";
    let value1_name = "Variant1";
    let value2_name = "Variant2";
    let value3_name = "Variant3";

    let enum1_def_ast = ast::test_helpers::mk_enum(type1_name, &[
        value1_name,
        value2_name,
        value3_name,
    ]);
    let enum2_def_ast = ast::test_helpers::mk_enum(type2_name, &[
        value1_name,
        value2_name,
        value3_name,
    ]);
    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![
            enum1_def_ast.clone(),
            enum2_def_ast.clone(),
        ],
        ast_ext_after: vec![],
        ast_ext_before: vec![],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![],
    })?;

    let expected_type1 = enum_type::test_helpers::mk_enum(type1_name, &[
        value1_name,
        value2_name,
        value3_name,
    ]);
    assert_eq!(
        types.get(type1_name),
        Some(&GraphQLType::Enum(expected_type1)),
    );

    let expected_type2 = enum_type::test_helpers::mk_enum(type2_name, &[
        value1_name,
        value2_name,
        value3_name,
    ]);
    assert_eq!(
        types.get(type2_name),
        Some(&GraphQLType::Enum(expected_type2)),
    );

    Ok(())
}

#[test]
fn enum_with_value_directive_no_args() -> Result<()> {
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

    let mut enum_def_ast = ast::test_helpers::mk_enum(type_name, &[value1_name]);
    enum_def_ast.values
        .get_mut(0)
        .unwrap()
        .directives
        .push(value1_directive.clone());

    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![enum_def_ast.clone()],
        ast_ext_after: vec![],
        ast_ext_before: vec![],
        file_path: file_path.clone(),
        types_after: vec![],
        types_before: vec![],
    })?;

    let mut expected_type = enum_type::test_helpers::mk_enum(type_name, &[value1_name]);
    let value1 = expected_type.variants.get_mut(value1_name).unwrap();
    value1.directives.push(DirectiveAnnotation {
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
fn enum_with_value_directive_single_arg() -> Result<()> {
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

    let mut enum_def_ast = ast::test_helpers::mk_enum(type_name, &[value1_name]);
    enum_def_ast.values
        .get_mut(0)
        .unwrap()
        .directives
        .push(directive.clone());

    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![enum_def_ast.clone()],
        ast_ext_after: vec![],
        ast_ext_before: vec![],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![],
    })?;

    let mut expected_type = enum_type::test_helpers::mk_enum(type_name, &[value1_name]);
    let value1 = expected_type.variants.get_mut(value1_name).unwrap();
    value1.directives.push(DirectiveAnnotation {
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

#[test]
fn enum_followed_by_extension_with_unique_value() -> Result<()> {
    let type_name = "TestEnum";
    let value1_name = "Variant1";
    let value2_name = "Variant2";
    let file_path = PathBuf::from("str://0");

    let enum_def_ast = ast::test_helpers::mk_enum(type_name, &[value1_name]);
    let enum_extension_ast =
        ast::test_helpers::mk_enum_extension(type_name, &[value2_name]);

    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![enum_def_ast.clone()],
        ast_ext_after: vec![enum_extension_ast.clone()],
        ast_ext_before: vec![],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![],
    })?;

    let expected_type = enum_type::test_helpers::mk_enum(type_name, &[
        value1_name,
        value2_name,
    ]);

    assert_eq!(
        types.get(type_name),
        Some(&GraphQLType::Enum(expected_type)),
    );

    Ok(())
}

#[test]
fn enum_followed_by_extension_with_colliding_value_is_an_error() -> Result<()> {
    let type_name = "TestEnum";
    let value1_name = "Variant1";
    let file_path = PathBuf::from("str://0");

    let enum_def_ast = ast::test_helpers::mk_enum(type_name, &[value1_name]);
    let mut enum_extension_ast =
        ast::test_helpers::mk_enum_extension(type_name, &[value1_name]);
    let enum_extension_value_ast = enum_extension_ast.values.first_mut().unwrap();
    enum_extension_value_ast.position.column = 3;
    enum_extension_value_ast.position.line = 4;

    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![enum_def_ast.clone()],
        ast_ext_after: vec![enum_extension_ast.clone()],
        ast_ext_before: vec![],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![],
    });

    assert_eq!(types.unwrap_err(), SchemaBuildError::DuplicateEnumValueDefinition {
        enum_name: type_name.to_string(),
        enum_def_location: loc::FilePosition::from_pos(
            file_path.to_owned(),
            enum_def_ast.position,
        ),
        value_def1: loc::FilePosition {
            col: 2,
            file: file_path.to_owned(),
            line: 2,
        },
        value_def2: loc::FilePosition {
            col: 3,
            file: file_path.to_owned(),
            line: 4,
        },
    });

    Ok(())
}

#[test]
fn enum_preceded_by_extension_with_unique_value() -> Result<()> {
    let type_name = "TestEnum";
    let value1_name = "Variant1";
    let value2_name = "Variant2";
    let file_path = PathBuf::from("str://0");

    let enum_def_ast = ast::test_helpers::mk_enum(type_name, &[value1_name]);
    let enum_extension_ast =
        ast::test_helpers::mk_enum_extension(type_name, &[value2_name]);

    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![enum_def_ast.clone()],
        ast_ext_after: vec![],
        ast_ext_before: vec![enum_extension_ast.clone()],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![],
    })?;

    let expected_type = enum_type::test_helpers::mk_enum(type_name, &[
        value1_name,
        value2_name,
    ]);

    assert_eq!(
        types.get(type_name),
        Some(&GraphQLType::Enum(expected_type)),
    );

    Ok(())
}

#[test]
fn enum_preceded_by_extension_with_colliding_value_is_an_error() -> Result<()> {
    let type_name = "TestEnum";
    let value1_name = "Variant1";
    let file_path = PathBuf::from("str://0");

    let enum_def_ast = ast::test_helpers::mk_enum(type_name, &[value1_name]);
    let mut enum_extension_ast =
        ast::test_helpers::mk_enum_extension(type_name, &[value1_name]);
    let enum_extension_value_ast = enum_extension_ast.values.first_mut().unwrap();
    enum_extension_value_ast.position.column = 3;
    enum_extension_value_ast.position.line = 4;

    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![enum_def_ast.clone()],
        ast_ext_after: vec![],
        ast_ext_before: vec![enum_extension_ast.clone()],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![],
    });

    assert_eq!(types.unwrap_err(), SchemaBuildError::DuplicateEnumValueDefinition {
        enum_name: type_name.to_string(),
        enum_def_location: loc::FilePosition::from_pos(
            file_path.to_owned(),
            enum_def_ast.position,
        ),
        value_def1: loc::FilePosition {
            col: 2,
            file: file_path.to_owned(),
            line: 2,
        },
        value_def2: loc::FilePosition {
            col: 3,
            file: file_path.to_owned(),
            line: 4,
        },
    });

    Ok(())
}

#[test]
fn enum_extension_without_original_def_is_an_error() -> Result<()> {
    let type_name = "TestEnum";
    let value1_name = "Variant1";
    let file_path = PathBuf::from("str://0");

    let enum_extension_ast =
        ast::test_helpers::mk_enum_extension(type_name, &[value1_name]);

    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![],
        ast_ext_after: vec![enum_extension_ast.clone()],
        ast_ext_before: vec![],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![],
    });

    assert_eq!(types.unwrap_err(), SchemaBuildError::ExtensionOfUndefinedType {
        type_name: type_name.to_string(),
        extension_type_loc: loc::FilePosition::from_pos(
            file_path.to_owned(),
            enum_extension_ast.position,
        ),
    });

    Ok(())
}

#[test]
fn enum_extension_after_non_enum_type_is_an_error() -> Result<()> {
    let type_name = "TestType";
    let value1_name = "Variant1";
    let file_path = PathBuf::from("str://0");

    let object_type_def_location = loc::FilePosition {
        col: 10,
        file: file_path.to_owned(),
        line: 11,
    };
    let object_type = GraphQLType::Object(ObjectType {
        def_location: object_type_def_location.to_owned(),
        directives: vec![],
        fields: BTreeMap::from([
            (value1_name.to_string(), Field {
                def_location: loc::SchemaDefLocation::Schema(loc::FilePosition {
                    col: 12,
                    file: file_path.to_owned(),
                    line: 12,
                }),
                type_ref: GraphQLTypeRef::Named {
                    nullable: true,
                    type_ref: NamedRef::new("Foo", loc::FilePosition {
                        col: 20,
                        file: file_path.to_owned(),
                        line: 12,
                    }),
                },
            }),
        ]),
        interfaces: vec![],
        name: type_name.to_string(),
    });
    let mut enum_extension_ast =
        ast::test_helpers::mk_enum_extension(type_name, &[value1_name]);
    let enum_extension_value_ast = enum_extension_ast.values.first_mut().unwrap();
    enum_extension_value_ast.position.column = 3;
    enum_extension_value_ast.position.line = 4;

    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![],
        ast_ext_after: vec![],
        ast_ext_before: vec![enum_extension_ast.clone()],
        file_path: file_path.to_path_buf(),
        types_after: vec![object_type.to_owned()],
        types_before: vec![],
    });

    assert_eq!(types.unwrap_err(), SchemaBuildError::InvalidExtensionType {
        schema_type: object_type.to_owned(),
        extension_loc: loc::FilePosition::from_pos(
            file_path.to_owned(),
            enum_extension_ast.position,
        ),
    });

    Ok(())
}

#[test]
fn enum_extension_preceding_non_enum_type_is_an_error() -> Result<()> {
    let type_name = "TestType";
    let value1_name = "Variant1";
    let file_path = PathBuf::from("str://0");

    let object_type_def_location = loc::FilePosition {
        col: 10,
        file: file_path.to_owned(),
        line: 11,
    };
    let object_type = GraphQLType::Object(ObjectType {
        def_location: object_type_def_location.to_owned(),
        directives: vec![],
        fields: BTreeMap::from([
            (value1_name.to_string(), Field {
                def_location: loc::SchemaDefLocation::Schema(loc::FilePosition {
                    col: 12,
                    file: file_path.to_owned(),
                    line: 12,
                }),
                type_ref: GraphQLTypeRef::Named {
                    nullable: true,
                    type_ref: NamedRef::new("Foo", loc::FilePosition {
                        col: 20,
                        file: file_path.to_owned(),
                        line: 12,
                    }),
                },
            }),
        ]),
        interfaces: vec![],
        name: type_name.to_string(),
    });
    let mut enum_extension_ast =
        ast::test_helpers::mk_enum_extension(type_name, &[value1_name]);
    let enum_extension_value_ast = enum_extension_ast.values.first_mut().unwrap();
    enum_extension_value_ast.position.column = 3;
    enum_extension_value_ast.position.line = 4;

    let types = EnumTypeBuilder::new().build_from_ast(TestBuildFromAst {
        ast_def: vec![],
        ast_ext_after: vec![],
        ast_ext_before: vec![enum_extension_ast.clone()],
        file_path: file_path.to_path_buf(),
        types_after: vec![],
        types_before: vec![object_type.to_owned()],
    });

    assert_eq!(types.unwrap_err(), SchemaBuildError::InvalidExtensionType {
        schema_type: object_type.to_owned(),
        extension_loc: loc::FilePosition::from_pos(
            file_path.to_owned(),
            enum_extension_ast.position,
        ),
    });

    Ok(())
}
