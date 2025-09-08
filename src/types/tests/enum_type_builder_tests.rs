use crate::ast;
use crate::loc;
use crate::schema::schema_builder::SchemaBuildError;
use crate::types::EnumTypeBuilder;
use crate::types::ObjectTypeBuilder;
use crate::types::TypesMapBuilder;
use crate::types::GraphQLType;
use crate::types::tests::test_utils;
use crate::Value;
use indexmap::IndexMap;
use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[test]
fn visit_enum_with_no_type_directives() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Value1";
    let enum_def =
        test_utils::parse_enum_type_def(
            enum_name,
            format!("enum {enum_name} {{ {value1_name} }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &enum_def
    )?;
    let enum_type = test_utils::get_enum_type(&mut types_map_builder, enum_name);

    assert!(enum_type.directives().is_empty());

    Ok(())
}

#[test]
fn visit_enum_with_one_type_directive_no_args() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Variant1";
    let directive_name = "deprecated";
    let enum_def =
        test_utils::parse_enum_type_def(
            enum_name,
            format!("enum {enum_name} @{directive_name} {{ {value1_name} }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema_path = PathBuf::from("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path.as_path()),
        &enum_def
    )?;
    let enum_type = test_utils::get_enum_type(&mut types_map_builder, enum_name);

    assert_eq!(enum_type.directives().len(), 1);
    let directive = enum_type.directives().first().unwrap();

    assert_eq!(directive.args(), &IndexMap::new());
    assert_eq!(directive.def_location(), &loc::FilePosition {
        col: 15,
        file: schema_path.into(),
        line: 1,
    }.into_schema_source_location());
    assert_eq!(directive.directive_type_name(), directive_name);

    Ok(())
}

#[test]
fn visit_enum_with_one_type_directive_one_arg() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Variant1";
    let directive_name = "some_custom_directive";
    let arg_name = "arg1";
    let arg_value = 42;
    let enum_def =
        test_utils::parse_enum_type_def(
            enum_name,
            format!(
                "enum {enum_name} @{directive_name}({arg_name}: {arg_value}) {{
                  {value1_name}
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema_path = PathBuf::from("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path.as_path()),
        &enum_def
    )?;
    let enum_type = test_utils::get_enum_type(&mut types_map_builder, enum_name);

    assert_eq!(enum_type.directives().len(), 1);

    let directive = enum_type.directives().first().unwrap();
    assert_eq!(directive.args(), &IndexMap::from([
        (arg_name.to_string(), Value::Int(arg_value.into())),
    ]));
    assert_eq!(directive.def_location(), &loc::FilePosition {
        col: 15,
        file: schema_path.into(),
        line: 1,
    }.into_schema_source_location());
    assert_eq!(directive.directive_type_name(), directive_name);

    Ok(())
}

#[test]
fn visit_enum_with_no_values_is_an_error() -> Result<()> {
    let enum_name = "TestEnum";
    // graphql_parser gives a parse error if you try to parse an enum type def
    // with no values. Since we accept an AST structure -- which still permits
    // the expression of an enum with no values -- we just manually construct
    // the structure here.
    let enum_def_pos = ast::AstPos {
        line: 1,
        column: 2,
    };
    let enum_def = ast::schema::EnumType {
        position: enum_def_pos,
        description: None,
        name: enum_name.to_string(),
        directives: vec![],
        values: vec![],
    };
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    let result = enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &enum_def
    );

    let err = result.unwrap_err();
    assert_eq!(err, SchemaBuildError::EnumWithNoVariants {
        type_name: enum_name.to_string(),
        location: loc::SourceLocation::from_schema_ast_position(
            Some(schema_path),
            &enum_def_pos,
        ),
    });

    Ok(())
}

#[test]
fn visit_enum_with_one_value_with_no_directives() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Value1";
    let enum_def =
        test_utils::parse_enum_type_def(
            enum_name,
            format!("enum {enum_name} {{ {value1_name} }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &enum_def
    )?;
    let enum_type = test_utils::get_enum_type(&mut types_map_builder, enum_name);

    assert_eq!(enum_type.values().len(), 1);
    assert!(enum_type.values().contains_key(value1_name));

    let enum_value = enum_type.values().get(value1_name).unwrap();
    assert_eq!(enum_value.def_location(), &loc::FilePosition {
        col: 17,
        file: schema_path.to_path_buf().into(),
        line: 1,
    }.into_schema_source_location());
    assert!(enum_value.directives().is_empty());
    assert_eq!(enum_value.name(), value1_name);
    assert_eq!(enum_value.enum_type_name(), enum_name);

    Ok(())
}

#[test]
fn visit_enum_with_one_value_with_one_directive_no_args() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Value1";
    let directive_name = "deprecated";
    let enum_def =
        test_utils::parse_enum_type_def(
            enum_name,
            format!(
                "enum {enum_name} {{
                    {value1_name} @{directive_name}
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &enum_def
    )?;
    let enum_type = test_utils::get_enum_type(&mut types_map_builder, enum_name);

    assert_eq!(enum_type.values().len(), 1);
    assert!(enum_type.values().contains_key(value1_name));
    let enum_value = enum_type.values().get(value1_name).unwrap();

    assert_eq!(enum_value.directives().len(), 1);
    let directive = enum_value.directives().first().unwrap();

    assert_eq!(directive.args(), &IndexMap::new());
    assert_eq!(directive.def_location(), &loc::FilePosition {
        col: 28,
        file: schema_path.to_path_buf().into(),
        line: 2,
    }.into_schema_source_location());
    assert_eq!(directive.directive_type_name(), directive_name);

    Ok(())
}

#[test]
fn visit_enum_with_one_value_with_one_directive_one_arg() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Value1";
    let directive_name = "deprecated";
    let arg_name = "arg1";
    let arg_value = 42;
    let enum_def =
        test_utils::parse_enum_type_def(
            enum_name,
            format!(
                "enum {enum_name} {{
                    {value1_name} @{directive_name}({arg_name}: {arg_value})
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &enum_def
    )?;
    let enum_type = test_utils::get_enum_type(&mut types_map_builder, enum_name);

    assert_eq!(enum_type.values().len(), 1);
    assert!(enum_type.values().contains_key(value1_name));
    let enum_value = enum_type.values().get(value1_name).unwrap();

    assert_eq!(enum_value.directives().len(), 1);
    let directive = enum_value.directives().first().unwrap();

    assert_eq!(directive.args(), &IndexMap::from([
        (arg_name.to_string(), Value::Int(arg_value.into())),
    ]));
    assert_eq!(directive.def_location(), &loc::FilePosition {
        col: 28,
        file: schema_path.to_path_buf().into(),
        line: 2,
    }.into_schema_source_location());
    assert_eq!(directive.directive_type_name(), directive_name);

    Ok(())
}

#[test]
fn visit_enum_with_multiple_values() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Value1";
    let value2_name = "Value2";
    let value3_name = "Value3";
    let enum_def =
        test_utils::parse_enum_type_def(
            enum_name,
            format!(
                "enum {enum_name} {{
                    {value1_name},
                    {value2_name},
                    {value3_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &enum_def
    )?;
    let enum_type = test_utils::get_enum_type(&mut types_map_builder, enum_name);

    let enum_values = enum_type.values();
    assert_eq!(enum_values.keys().collect::<Vec<_>>(), vec![
        &value1_name.to_string(),
        &value2_name.to_string(),
        &value3_name.to_string(),
    ]);

    let enum_value1 = enum_values.get(value1_name).unwrap();
    assert_eq!(enum_value1.def_location(), &loc::FilePosition {
        col: 21,
        file: schema_path.to_path_buf().into(),
        line: 2,
    }.into_schema_source_location());
    assert_eq!(enum_value1.directives().len(), 0);
    assert_eq!(enum_value1.name(), value1_name);
    assert_eq!(enum_value1.enum_type_name(), enum_name);

    let enum_value2 = enum_values.get(value2_name).unwrap();
    assert_eq!(enum_value2.def_location(), &loc::FilePosition {
        col: 21,
        file: schema_path.to_path_buf().into(),
        line: 3,
    }.into_schema_source_location());
    assert_eq!(enum_value2.directives().len(), 0);
    assert_eq!(enum_value2.name(), value2_name);
    assert_eq!(enum_value2.enum_type_name(), enum_name);

    let enum_value3 = enum_values.get(value3_name).unwrap();
    assert_eq!(enum_value3.def_location(), &loc::FilePosition {
        col: 21,
        file: schema_path.to_path_buf().into(),
        line: 4,
    }.into_schema_source_location());
    assert_eq!(enum_value3.directives().len(), 0);
    assert_eq!(enum_value3.name(), value3_name);
    assert_eq!(enum_value3.enum_type_name(), enum_name);

    Ok(())
}

#[test]
fn visit_two_enums_with_same_value_names() -> Result<()> {
    let enum1_name = "TestEnum1";
    let enum2_name = "TestEnum2";
    let value1_name = "Value1";
    let value2_name = "Value2";
    let enum1_def =
        test_utils::parse_enum_type_def(
            enum1_name,
            format!(
                "enum {enum1_name} {{
                    {value1_name},
                    {value2_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let enum2_def =
        test_utils::parse_enum_type_def(
            enum2_name,
            format!(
                "enum {enum2_name} {{
                    {value1_name},
                    {value2_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema1_path),
        &enum1_def
    )?;
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema2_path),
        &enum2_def
    )?;
    let enum1_type = test_utils::get_enum_type(&mut types_map_builder, enum1_name);
    let enum2_type = test_utils::get_enum_type(&mut types_map_builder, enum2_name);

    let enum1_values = enum1_type.values();
    assert_eq!(enum1_values.keys().collect::<Vec<_>>(), vec![
        &value1_name.to_string(),
        &value2_name.to_string(),
    ]);

    let enum1_value1 = enum1_values.get(value1_name).unwrap();
    assert_eq!(enum1_value1.def_location(), &loc::FilePosition {
        col: 21,
        file: schema1_path.to_path_buf().into(),
        line: 2,
    }.into_schema_source_location());
    assert_eq!(enum1_value1.directives().len(), 0);
    assert_eq!(enum1_value1.name(), value1_name);
    assert_eq!(enum1_value1.enum_type_name(), enum1_name);

    let enum1_value2 = enum1_values.get(value2_name).unwrap();
    assert_eq!(enum1_value2.def_location(), &loc::FilePosition {
        col: 21,
        file: schema1_path.to_path_buf().into(),
        line: 3,
    }.into_schema_source_location());
    assert_eq!(enum1_value2.directives().len(), 0);
    assert_eq!(enum1_value2.name(), value2_name);
    assert_eq!(enum1_value2.enum_type_name(), enum1_name);

    let enum2_values = enum2_type.values();
    assert_eq!(enum2_values.keys().collect::<Vec<_>>(), vec![
        &value1_name.to_string(),
        &value2_name.to_string(),
    ]);

    let enum2_value1 = enum2_values.get(value1_name).unwrap();
    assert_eq!(enum2_value1.def_location(), &loc::FilePosition {
        col: 21,
        file: schema2_path.to_path_buf().into(),
        line: 2,
    }.into_schema_source_location());
    assert_eq!(enum2_value1.directives().len(), 0);
    assert_eq!(enum2_value1.name(), value1_name);
    assert_eq!(enum2_value1.enum_type_name(), enum2_name);

    let enum2_value2 = enum2_values.get(value2_name).unwrap();
    assert_eq!(enum2_value2.def_location(), &loc::FilePosition {
        col: 21,
        file: schema2_path.to_path_buf().into(),
        line: 3,
    }.into_schema_source_location());
    assert_eq!(enum2_value2.directives().len(), 0);
    assert_eq!(enum2_value2.name(), value2_name);
    assert_eq!(enum2_value2.enum_type_name(), enum2_name);

    Ok(())
}

#[test]
fn visit_enum_followed_by_extension_with_unique_value() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Value1";
    let value2_name = "Value2";
    let enum_def =
        test_utils::parse_enum_type_def(
            enum_name,
            format!(
                "enum {enum_name} {{
                    {value1_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let enum_ext =
        test_utils::parse_enum_type_ext(
            enum_name,
            format!(
                "extend enum {enum_name} {{
                    {value2_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema1_path),
        &enum_def
    )?;
    enum_builder.visit_type_extension(
        &mut types_map_builder,
        Some(schema2_path),
        enum_ext,
    )?;
    let enum_type = test_utils::get_enum_type(&mut types_map_builder, enum_name);

    let enum_values = enum_type.values();
    assert_eq!(enum_values.keys().collect::<Vec<_>>(), vec![
        &value1_name.to_string(),
        &value2_name.to_string(),
    ]);

    let enum_value1 = enum_values.get(value1_name).unwrap();
    assert_eq!(enum_value1.def_location(), &loc::FilePosition {
        col: 21,
        file: schema1_path.to_path_buf().into(),
        line: 2,
    }.into_schema_source_location());
    assert_eq!(enum_value1.directives().len(), 0);
    assert_eq!(enum_value1.name(), value1_name);
    assert_eq!(enum_value1.enum_type_name(), enum_name);

    let enum_value2 = enum_values.get(value2_name).unwrap();
    assert_eq!(enum_value2.def_location(), &loc::FilePosition {
        col: 21,
        file: schema2_path.to_path_buf().into(),
        line: 2,
    }.into_schema_source_location());
    assert_eq!(enum_value2.directives().len(), 0);
    assert_eq!(enum_value2.name(), value2_name);
    assert_eq!(enum_value2.enum_type_name(), enum_name);

    Ok(())
}

#[test]
fn visit_enum_followed_by_extension_with_colliding_value() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Value1";
    let value2_name = "Value2";
    let value3_name = "Value2";
    let enum_def =
        test_utils::parse_enum_type_def(
            enum_name,
            format!(
                "enum {enum_name} {{
                    {value1_name},
                    {value3_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let enum_ext =
        test_utils::parse_enum_type_ext(
            enum_name,
            format!(
                "extend enum {enum_name} {{
                    {value2_name},
                    {value3_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema1_path),
        &enum_def
    )?;
    let result = enum_builder.visit_type_extension(
        &mut types_map_builder,
        Some(schema2_path),
        enum_ext,
    );

    let err = result.unwrap_err();
    assert_eq!(err, SchemaBuildError::DuplicateEnumValueDefinition {
        enum_name: enum_name.to_string(),
        enum_def_location: loc::FilePosition {
            col: 1,
            file: schema1_path.to_path_buf().into(),
            line: 1,
        }.into_schema_source_location(),
        value_def1: loc::FilePosition {
            col: 21,
            file: schema1_path.to_path_buf().into(),
            line: 3,
        }.into_schema_source_location(),
        value_def2: loc::FilePosition {
            col: 21,
            file: schema2_path.to_path_buf().into(),
            line: 2,
        }.into_schema_source_location(),
    });

    Ok(())
}

#[test]
fn visit_enum_preceded_by_extension_with_unique_value() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Value1";
    let value2_name = "Value2";
    let enum_def =
        test_utils::parse_enum_type_def(
            enum_name,
            format!(
                "enum {enum_name} {{
                    {value1_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let enum_ext =
        test_utils::parse_enum_type_ext(
            enum_name,
            format!(
                "extend enum {enum_name} {{
                    {value2_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_extension(
        &mut types_map_builder,
        Some(schema2_path),
        enum_ext,
    )?;
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema1_path),
        &enum_def
    )?;
    enum_builder.finalize(&mut types_map_builder)?;

    let enum_type =
        types_map_builder.get_type_mut(enum_name)
            .expect("Type was created")
            .as_enum()
            .expect("Type was enum")
            .to_owned();

    let enum_values = enum_type.values();
    assert_eq!(enum_values.keys().collect::<Vec<_>>(), vec![
        &value1_name.to_string(),
        &value2_name.to_string(),
    ]);

    let enum_value1 = enum_values.get(value1_name).unwrap();
    assert_eq!(enum_value1.def_location(), &loc::FilePosition {
        col: 21,
        file: schema1_path.to_path_buf().into(),
        line: 2,
    }.into_schema_source_location());
    assert_eq!(enum_value1.directives().len(), 0);
    assert_eq!(enum_value1.name(), value1_name);
    assert_eq!(enum_value1.enum_type_name(), enum_name);

    let enum_value2 = enum_values.get(value2_name).unwrap();
    assert_eq!(enum_value2.def_location(), &loc::FilePosition {
        col: 21,
        file: schema2_path.to_path_buf().into(),
        line: 2,
    }.into_schema_source_location());
    assert_eq!(enum_value2.directives().len(), 0);
    assert_eq!(enum_value2.name(), value2_name);
    assert_eq!(enum_value2.enum_type_name(), enum_name);

    Ok(())
}

#[test]
fn enum_preceded_by_extension_with_colliding_value() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Value1";
    let value2_name = "Value2";
    let value3_name = "Value3";
    let enum_def =
        test_utils::parse_enum_type_def(
            enum_name,
            format!(
                "enum {enum_name} {{
                    {value1_name},
                    {value3_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let enum_ext =
        test_utils::parse_enum_type_ext(
            enum_name,
            format!(
                "extend enum {enum_name} {{
                    {value2_name},
                    {value3_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_extension(
        &mut types_map_builder,
        Some(schema2_path),
        enum_ext,
    )?;
    enum_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema1_path),
        &enum_def
    )?;

    let result = enum_builder.finalize(&mut types_map_builder);

    let err = result.unwrap_err();
    assert_eq!(err, SchemaBuildError::DuplicateEnumValueDefinition {
        enum_name: enum_name.to_string(),
        enum_def_location: loc::FilePosition {
            col: 1,
            file: schema1_path.to_path_buf().into(),
            line: 1,
        }.into_schema_source_location(),
        value_def1: loc::FilePosition {
            col: 21,
            file: schema1_path.to_path_buf().into(),
            line: 3,
        }.into_schema_source_location(),
        value_def2: loc::FilePosition {
            col: 21,
            file: schema2_path.to_path_buf().into(),
            line: 3,
        }.into_schema_source_location(),
    });

    Ok(())
}

#[test]
fn visit_enum_extension_without_type_def() -> Result<()> {
    let enum_name = "TestEnum";
    let value1_name = "Value1";
    let enum_ext =
        test_utils::parse_enum_type_ext(
            enum_name,
            format!(
                "extend enum {enum_name} {{
                    {value1_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");
    let schema1_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_extension(
        &mut types_map_builder,
        Some(schema1_path),
        enum_ext,
    )?;

    let result = enum_builder.finalize(&mut types_map_builder);

    let err = result.unwrap_err();
    assert_eq!(err, SchemaBuildError::ExtensionOfUndefinedType {
        type_name: enum_name.to_string(),
        extension_location: loc::FilePosition {
            col: 8,
            file: schema1_path.to_path_buf().into(),
            line: 1,
        }.into_schema_source_location(),
    });

    Ok(())
}

#[test]
fn visit_enum_extension_of_non_enum_type() -> Result<()> {
    let type_name = "TestType";
    let value1_name = "Variant1";
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");
    let obj_def =
        test_utils::parse_object_type_def(
            type_name,
            format!("type {type_name} {{ foo: Int }}").as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let enum_ext =
        test_utils::parse_enum_type_ext(
            type_name,
            format!(
                "extend enum {type_name} {{
                    {value1_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema1_path),
        &obj_def,
    )?;
    let result = enum_builder.visit_type_extension(
        &mut types_map_builder,
        Some(schema2_path),
        enum_ext,
    );

    let obj_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    let err = result.unwrap_err();
    assert_eq!(err, SchemaBuildError::InvalidExtensionType {
        schema_type: GraphQLType::Object(obj_type.into()),
        extension_location: loc::FilePosition {
            col: 8,
            file: schema2_path.to_path_buf().into(),
            line: 1,
        }.into_schema_source_location(),
    });

    Ok(())
}

#[test]
fn visit_enum_extension_preceding_def_of_non_enum_type() -> Result<()> {
    let type_name = "TestType";
    let value1_name = "Variant1";
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");
    let obj_def =
        test_utils::parse_object_type_def(
            type_name,
            format!("type {type_name} {{ foo: Int }}").as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let enum_ext =
        test_utils::parse_enum_type_ext(
            type_name,
            format!(
                "extend enum {type_name} {{
                    {value1_name},
                }}").as_str(),
        )
        .expect("parse error")
        .expect("no enum type def found");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    enum_builder.visit_type_extension(
        &mut types_map_builder,
        Some(schema2_path),
        enum_ext,
    )?;
    object_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema1_path),
        &obj_def,
    )?;
    let result = enum_builder.finalize(&mut types_map_builder);

    let obj_type = test_utils::get_object_type(&mut types_map_builder, type_name);

    let err = result.unwrap_err();

    assert_eq!(err, SchemaBuildError::InvalidExtensionType {
        schema_type: GraphQLType::Object(obj_type.into()),
        extension_location: loc::FilePosition {
            col: 8,
            file: schema2_path.to_path_buf().into(),
            line: 1,
        }.into_schema_source_location(),
    });

    Ok(())
}
