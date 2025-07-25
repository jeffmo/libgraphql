use crate::ast;
use crate::loc;
use crate::schema::schema_builder::SchemaBuildError;
use crate::types::EnumTypeBuilder;
use crate::types::GraphQLType;
use crate::types::ObjectTypeBuilder;
use crate::types::tests::test_utils;
use crate::types::TypesMapBuilder;
use crate::Value;
use std::boxed::Box;
use std::collections::BTreeMap;
use std::path::Path;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[test]
fn visit_object_with_no_type_directives() -> Result<()> {
    let type_name = "TestObject";
    let field_name = "field1";
    let field_type = "Int";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} {{
                    {field_name}: {field_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        object_def,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    assert!(object_type.directives().is_empty());

    Ok(())
}

#[test]
fn visit_object_with_one_type_directives_no_args() -> Result<()> {
    let type_name = "TestObject";
    let field_name = "field1";
    let field_type = "Int";
    let directive_name = "deprecated";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} @{directive_name} {{
                    {field_name}: {field_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        object_def,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    assert_eq!(object_type.directives().len(), 1);
    let directive = object_type.directives().first().unwrap();

    assert_eq!(directive.args(), &BTreeMap::new());
    assert_eq!(directive.def_location(), &loc::FilePosition {
        col: 17,
        file: Box::new(schema_path.to_path_buf()),
        line: 1,
    }.into());
    assert_eq!(directive.directive_type_name(), directive_name);

    Ok(())
}

#[test]
fn visit_object_with_one_type_directives_one_arg() -> Result<()> {
    let type_name = "TestObject";
    let field_name = "field1";
    let field_type = "Int";
    let directive_name = "custom";
    let arg_name = "arg1";
    let arg_value = 42;
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} @{directive_name}({arg_name}: {arg_value}) {{
                    {field_name}: {field_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        object_def,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    assert_eq!(object_type.directives().len(), 1);
    let directive = object_type.directives().first().unwrap();

    assert_eq!(directive.args(), &BTreeMap::from([
        (arg_name.to_string(), Value::Int(arg_value.into())),
    ]));
    assert_eq!(directive.def_location(), &loc::FilePosition {
        col: 17,
        file: Box::new(schema_path.to_path_buf()),
        line: 1,
    }.into());
    assert_eq!(directive.directive_type_name(), directive_name);

    Ok(())
}

#[test]
fn visit_object_with_no_interface() -> Result<()> {
    let type_name = "TestObject";
    let field_name = "field1";
    let field_type = "Int";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} {{
                    {field_name}: {field_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        object_def,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    assert!(object_type.interface_names().is_empty());

    Ok(())
}

#[test]
fn visit_object_with_one_interface() -> Result<()> {
    let type_name = "TestObject";
    let iface_name = "Iface1";
    let field_name = "field1";
    let field_type = "Int";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} implements {iface_name} {{
                    {field_name}: {field_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        object_def,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    assert_eq!(object_type.interface_names(), vec![
        iface_name,
    ]);

    Ok(())
}

#[test]
fn visit_object_with_multiple_interfaces() -> Result<()> {
    let type_name = "TestObject";
    let iface1_name = "Iface1";
    let iface2_name = "Iface2";
    let iface3_name = "Iface3";
    let field_name = "field1";
    let field_type = "Int";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} implements {iface1_name} & {iface2_name} & {iface3_name} {{
                    {field_name}: {field_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        object_def,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    assert_eq!(object_type.interface_names(), vec![
        iface1_name,
        iface2_name,
        iface3_name,
    ]);

    Ok(())
}

#[test]
fn visit_object_with_no_fields() -> Result<()> {
    let type_name = "TestObject";
    // graphql_parser gives a parse error if you try to parse an object type def
    // with no fields. Since we accept an AST structure -- which still permits
    // the expression of an object with no fields -- we just manually construct
    // the structure here.
    let object_def_pos = ast::AstPos {
        line: 1,
        column: 2,
    };
    let object_def = ast::schema::ObjectType {
        position: object_def_pos,
        description: None,
        implements_interfaces: vec![],
        name: type_name.to_string(),
        directives: vec![],
        fields: vec![],
    };
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        object_def,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    assert_eq!(object_type.fields().keys().collect::<Vec<_>>(), vec![
        &"__typename".to_string(),
    ]);

    Ok(())
}

#[test]
fn visit_object_with_one_field_with_no_directives() -> Result<()> {
    let type_name = "TestObject";
    let field_name = "field1";
    let field_type = "Int";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} {{
                    {field_name}: {field_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        object_def,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    let fields = object_type.fields();

    assert_eq!(fields.len(), 2);
    assert!(fields.contains_key(field_name));
    let field = fields.get(field_name).unwrap();

    assert_eq!(field.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema_path.to_path_buf()),
        line: 2,
    }.into());
    assert!(field.directives().is_empty());
    assert_eq!(field.name(), field_name);

    let field_type_annot = field.type_annotation();
    assert_eq!(field_type_annot.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema_path.to_path_buf()),
        line: 2,
    }.into());

    let field_type_named_annot =
        field_type_annot.as_named_annotation()
            .expect("is a named type annotation");
    assert_eq!(field_type_named_annot.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema_path.to_path_buf()),
        line: 2,
    }.into());
    assert_eq!(field_type_named_annot.graphql_type_name(), "Int");
    assert!(field_type_named_annot.nullable());

    Ok(())
}

#[test]
fn visit_object_with_one_field_with_one_directive_no_args() -> Result<()> {
    let type_name = "TestObject";
    let field_name = "field1";
    let field_type = "Int";
    let directive_name = "deprecated";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} {{
                    {field_name}: {field_type} @{directive_name},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        object_def,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    let fields = object_type.fields();

    assert_eq!(fields.len(), 2);
    assert!(fields.contains_key(field_name));
    let field = fields.get(field_name).unwrap();

    assert_eq!(field.directives().len(), 1);
    let directive = field.directives().first().unwrap();

    assert_eq!(directive.args(), &BTreeMap::new());
    assert_eq!(directive.def_location(), &loc::FilePosition {
        col: 33,
        file: Box::new(schema_path.to_path_buf()),
        line: 2,
    }.into());
    assert_eq!(directive.directive_type_name(), directive_name);

    Ok(())
}

#[test]
fn visit_object_with_one_field_with_one_directive_one_arg() -> Result<()> {
    let type_name = "TestObject";
    let field_name = "field1";
    let field_type = "Int";
    let directive_name = "custom";
    let arg_name = "arg1";
    let arg_value = 42;
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} {{
                    {field_name}: {field_type} @{directive_name}({arg_name}: {arg_value}),
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        object_def,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    let fields = object_type.fields();

    assert_eq!(fields.len(), 2);
    assert!(fields.contains_key("__typename"));
    assert!(fields.contains_key(field_name));
    let field = fields.get(field_name).unwrap();

    assert_eq!(field.directives().len(), 1);
    let directive = field.directives().first().unwrap();

    assert_eq!(directive.args(), &BTreeMap::from([
        (arg_name.to_string(), Value::Int(arg_value.into())),
    ]));
    assert_eq!(directive.def_location(), &loc::FilePosition {
        col: 33,
        file: Box::new(schema_path.to_path_buf()),
        line: 2,
    }.into());
    assert_eq!(directive.directive_type_name(), directive_name);

    Ok(())
}

#[test]
fn visit_object_with_multiple_fields() -> Result<()> {
    let type_name = "TestObject";
    let field1_name = "field1";
    let field1_type = "Int";
    let field2_name = "field2";
    let field2_type = "String";
    let field3_name = "field3";
    let field3_type = "OtherObject";
    let field4_name = "field4";
    let field4_p1_name = "num1";
    let field4_p1_type = "Float";
    let field4_p2_name = "num2";
    let field4_p2_default = "1.0";
    let field4_p2_type = "Float";
    let field4_type = "Float";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} {{
                    {field1_name}: {field1_type},
                    {field2_name}: [{field2_type}]!,
                    {field3_name}: {field3_type},
                    {field4_name}(
                        {field4_p1_name}: {field4_p1_type},
                        {field4_p2_name}: {field4_p2_type}! = {field4_p2_default},
                    ): {field4_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        object_def,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    let fields = object_type.fields();

    assert_eq!(fields.keys().collect::<Vec<_>>(), vec![
        &"__typename".to_string(),
        &field1_name.to_string(),
        &field2_name.to_string(),
        &field3_name.to_string(),
        &field4_name.to_string(),
    ]);

    let field1 = fields.get(field1_name).unwrap();
    assert_eq!(field1.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema_path.to_path_buf()),
        line: 2,
    }.into());
    assert!(field1.directives().is_empty());
    assert_eq!(field1.name(), field1_name);
    assert!(field1.parameters().is_empty());

    let field1_type_annot =
        field1.type_annotation()
            .as_named_annotation()
            .expect("is a NamedTypeAnnotation");
    assert_eq!(field1_type_annot.graphql_type_name(), field1_type);
    assert!(field1_type_annot.nullable());

    let field2 = fields.get(field2_name).unwrap();
    assert_eq!(field2.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema_path.to_path_buf()),
        line: 3,
    }.into());
    assert!(field2.directives().is_empty());
    assert_eq!(field2.name(), field2_name);
    assert!(field2.parameters().is_empty());

    let field2_list_type_annot =
        field2.type_annotation()
            .as_list_annotation()
            .expect("is a ListTypeAnnotation");
    assert_eq!(
        field2_list_type_annot
            .inner_type_annotation()
            .as_named_annotation()
            .expect("inner type is a NamedTypeAnnotation")
            .graphql_type_name(),
        field2_type,
    );
    assert!(!field2_list_type_annot.nullable());

    let field3 = fields.get(field3_name).unwrap();
    assert_eq!(field3.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema_path.to_path_buf()),
        line: 4,
    }.into());
    assert!(field3.directives().is_empty());
    assert_eq!(field3.name(), field3_name);
    assert!(field3.parameters().is_empty());

    let field3_type_annot =
        field3.type_annotation()
            .as_named_annotation()
            .expect("is a NamedTypeAnnotation");
    assert_eq!(field3_type_annot.graphql_type_name(), field3_type);
    assert!(field3_type_annot.nullable());

    let field4 = fields.get(field4_name).unwrap();
    assert_eq!(field4.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema_path.to_path_buf()),
        line: 5,
    }.into());
    assert!(field4.directives().is_empty());
    assert_eq!(field4.name(), field4_name);
    assert_eq!(field4.parameters().keys().collect::<Vec<_>>(), vec![
        &field4_p1_name.to_string(),
        &field4_p2_name.to_string(),
    ]);

    let field4_p1 = field4.parameters().get(field4_p1_name).unwrap();
    assert_eq!(field4_p1.def_location(), &loc::FilePosition {
        col: 25,
        file: Box::new(schema_path.to_path_buf()),
        line: 6,
    }.into());
    assert_eq!(field4_p1.default_value(), &None);
    assert_eq!(field4_p1.name(), field4_p1_name);

    let field4_p1_type_annot =
        field4_p1.type_annotation()
            .as_named_annotation()
            .expect("is a NamedTypeAnnotation");
    assert_eq!(field4_p1_type_annot.graphql_type_name(), field4_p1_type);
    assert!(field4_p1_type_annot.nullable);

    let field4_p2 = field4.parameters().get(field4_p2_name).unwrap();
    assert_eq!(field4_p2.def_location(), &loc::FilePosition {
        col: 25,
        file: Box::new(schema_path.to_path_buf()),
        line: 7,
    }.into());
    assert_eq!(
        field4_p2.default_value(),
        &Some(Value::Float(field4_p2_default.parse::<f64>().unwrap())),
    );
    assert_eq!(field4_p2.name(), field4_p2_name);

    let field4_p2_type_annot =
        field4_p2.type_annotation()
            .as_named_annotation()
            .expect("is a NamedTypeAnnotation");
    assert_eq!(field4_p2_type_annot.graphql_type_name(), field4_p2_type);
    assert!(!field4_p2_type_annot.nullable);

    let field4_type_annot =
        field4.type_annotation()
            .as_named_annotation()
            .expect("is a NamedTypeAnnotation");
    assert_eq!(field4_type_annot.graphql_type_name(), field4_type);
    assert!(field4_type_annot.nullable());

    Ok(())
}

#[test]
fn visit_object_followed_by_extension_with_unique_field() -> Result<()> {
    let type_name = "TestObject";
    let field1_name = "field1";
    let field1_type = "Int";
    let field2_name = "field2";
    let field2_p1_name = "param1";
    let field2_p1_type = "Boolean";
    let field2_type = "String";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} {{
                    {field1_name}: {field1_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let object_ext =
        test_utils::parse_object_type_ext(
            type_name,
            format!(
                "extend type {type_name} {{
                    {field2_name}(
                        {field2_p1_name}: {field2_p1_type},
                    ): {field2_type}!,
                }}"
            ).as_str(),
        )
        .expect("no parse error")
        .expect("no object type def found");
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema1_path,
        object_def,
    )?;
    object_builder.visit_type_extension(
        &mut types_map_builder,
        schema2_path,
        object_ext,
    )?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    let fields = object_type.fields();
    assert_eq!(fields.keys().collect::<Vec<_>>(), vec![
        &"__typename".to_string(),
        &field1_name.to_string(),
        &field2_name.to_string(),
    ]);

    let field1 = fields.get(field1_name).unwrap();
    assert_eq!(field1.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema1_path.to_path_buf()),
        line: 2,
    }.into());
    assert!(field1.directives().is_empty());
    assert_eq!(field1.name(), field1_name);

    let field1_type_annot = field1.type_annotation();
    assert_eq!(field1_type_annot.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema1_path.to_path_buf()),
        line: 2,
    }.into());
    assert_eq!(
        field1_type_annot.as_named_annotation()
            .expect("is a NamedTypeAnnotation")
            .graphql_type_name(),
        field1_type,
    );
    assert!(field1_type_annot.nullable());

    let field2 = fields.get(field2_name).unwrap();
    assert_eq!(field2.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema2_path.to_path_buf()),
        line: 2,
    }.into());
    assert!(field2.directives().is_empty());
    assert_eq!(field2.name(), field2_name);

    let field2_p1 = field2.parameters().get(field2_p1_name).unwrap();
    assert_eq!(field2_p1.def_location(), &loc::FilePosition {
        col: 25,
        file: Box::new(schema2_path.to_path_buf()),
        line: 3,
    }.into());
    assert_eq!(field2_p1.default_value(), &None);
    assert_eq!(field2_p1.name(), field2_p1_name);

    let field2_p1_type_annot =
        field2_p1.type_annotation()
            .as_named_annotation()
            .expect("is a NamedTypeAnnotation");
    assert_eq!(field2_p1_type_annot.graphql_type_name(), field2_p1_type);
    assert!(field2_p1_type_annot.nullable);


    let field2_type_annot = field2.type_annotation();
    assert_eq!(field2_type_annot.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema2_path.to_path_buf()),
        line: 2,
    }.into());
    assert_eq!(
        field2_type_annot.as_named_annotation()
            .expect("is a NamedTypeAnnotation")
            .graphql_type_name(),
        field2_type,
    );
    assert!(!field2_type_annot.nullable());

    Ok(())
}

#[test]
fn visit_object_followed_by_extension_with_colliding_field_name() -> Result<()> {
    let type_name = "TestObject";
    let field_name = "field1";
    let field1_type = "Int";
    let field2_type = "String";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} {{
                    {field_name}: {field1_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let object_ext =
        test_utils::parse_object_type_ext(
            type_name,
            format!(
                "extend type {type_name} {{
                    {field_name}: {field2_type}!,
                }}"
            ).as_str(),
        )
        .expect("no parse error")
        .expect("no object type def found");
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema1_path,
        object_def,
    )?;
    let result = object_builder.visit_type_extension(
        &mut types_map_builder,
        schema2_path,
        object_ext,
    );

    let err = result.unwrap_err();
    assert_eq!(err, SchemaBuildError::DuplicateFieldNameDefinition {
        type_name: type_name.to_string(),
        field_name: field_name.to_string(),
        field_def1: loc::FilePosition {
            col: 21,
            file: Box::new(schema1_path.to_path_buf()),
            line: 2,
        }.into(),
        field_def2: loc::FilePosition {
            col: 21,
            file: Box::new(schema2_path.to_path_buf()),
            line: 2,
        }.into(),
    });

    Ok(())
}

#[test]
fn visit_object_preceded_by_extension_with_unique_field() -> Result<()> {
    let type_name = "TestObject";
    let field1_name = "field1";
    let field1_type = "Int";
    let field2_name = "field2";
    let field2_type = "String";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} {{
                    {field1_name}: {field1_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let object_ext =
        test_utils::parse_object_type_ext(
            type_name,
            format!(
                "extend type {type_name} {{
                    {field2_name}: {field2_type}!,
                }}"
            ).as_str(),
        )
        .expect("no parse error")
        .expect("no object type def found");
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_extension(
        &mut types_map_builder,
        schema2_path,
        object_ext,
    )?;
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema1_path,
        object_def,
    )?;
    object_builder.finalize(&mut types_map_builder)?;
    let object_type = test_utils::get_object_type(
        &mut types_map_builder,
        type_name,
    );

    let fields = object_type.fields();
    assert_eq!(fields.keys().collect::<Vec<_>>(), vec![
        &"__typename".to_string(),
        &field1_name.to_string(),
        &field2_name.to_string(),
    ]);

    let field1 = fields.get(field1_name).unwrap();
    assert_eq!(field1.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema1_path.to_path_buf()),
        line: 2,
    }.into());
    assert!(field1.directives().is_empty());
    assert_eq!(field1.name(), field1_name);

    let field1_type_annot = field1.type_annotation();
    assert_eq!(field1_type_annot.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema1_path.to_path_buf()),
        line: 2,
    }.into());
    assert_eq!(
        field1_type_annot.as_named_annotation()
            .expect("is a NamedTypeAnnotation")
            .graphql_type_name(),
        field1_type,
    );
    assert!(field1_type_annot.nullable());

    let field2 = fields.get(field2_name).unwrap();
    assert_eq!(field2.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema2_path.to_path_buf()),
        line: 2,
    }.into());
    assert!(field2.directives().is_empty());
    assert_eq!(field2.name(), field2_name);

    let field2_type_annot = field2.type_annotation();
    assert_eq!(field2_type_annot.def_location(), &loc::FilePosition {
        col: 21,
        file: Box::new(schema2_path.to_path_buf()),
        line: 2,
    }.into());
    assert_eq!(
        field2_type_annot.as_named_annotation()
            .expect("is a NamedTypeAnnotation")
            .graphql_type_name(),
        field2_type,
    );
    assert!(!field2_type_annot.nullable());

    Ok(())
}

#[test]
fn visit_object_preceded_by_extension_with_colliding_field() -> Result<()> {
    let type_name = "TestObject";
    let field_name = "field1";
    let field1_type = "Int";
    let field2_type = "String";
    let object_def =
        test_utils::parse_object_type_def(
            type_name,
            format!(
                "type {type_name} {{
                    {field_name}: {field1_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let object_ext =
        test_utils::parse_object_type_ext(
            type_name,
            format!(
                "extend type {type_name} {{
                    {field_name}: {field2_type}!,
                }}"
            ).as_str(),
        )
        .expect("no parse error")
        .expect("no object type def found");
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_extension(
        &mut types_map_builder,
        schema2_path,
        object_ext,
    )?;
    object_builder.visit_type_def(
        &mut types_map_builder,
        schema1_path,
        object_def,
    )?;
    let result = object_builder.finalize(&mut types_map_builder);

    let err = result.unwrap_err();
    assert_eq!(err, SchemaBuildError::DuplicateFieldNameDefinition {
        type_name: type_name.to_string(),
        field_name: field_name.to_string(),
        field_def1: loc::FilePosition {
            col: 21,
            file: Box::new(schema1_path.to_path_buf()),
            line: 2,
        }.into(),
        field_def2: loc::FilePosition {
            col: 21,
            file: Box::new(schema2_path.to_path_buf()),
            line: 2,
        }.into(),
    });

    Ok(())
}

#[test]
fn visit_object_extension_without_type_def() -> Result<()> {
    let type_name = "TestObject";
    let field_name = "field1";
    let field_type = "Int";
    let object_ext =
        test_utils::parse_object_type_ext(
            type_name,
            format!(
                "extend type {type_name} {{
                    {field_name}: {field_type}!,
                }}"
            ).as_str(),
        )
        .expect("no parse error")
        .expect("no object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_extension(
        &mut types_map_builder,
        schema_path,
        object_ext,
    )?;
    let result = object_builder.finalize(&mut types_map_builder);

    let err = result.unwrap_err();
    assert_eq!(err, SchemaBuildError::ExtensionOfUndefinedType {
        type_name: type_name.to_string(),
        extension_type_loc: loc::FilePosition {
            col: 8,
            file: Box::new(schema_path.to_path_buf()),
            line: 1,
        }.into(),
    });

    Ok(())
}

#[test]
fn visit_object_extension_of_non_object_type() -> Result<()> {
    let type_name = "TestType";
    let field_name = "field1";
    let field_type = "Int";
    let enum_def =
        test_utils::parse_enum_type_def(
            type_name,
            format!("enum {type_name} {{ value1 }}").as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let object_ext =
        test_utils::parse_object_type_ext(
            type_name,
            format!(
                "extend type {type_name} {{
                    {field_name}: {field_type}!,
                }}"
            ).as_str(),
        )
        .expect("no parse error")
        .expect("no object type def found");
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    enum_builder.visit_type_def(
        &mut types_map_builder,
        schema1_path,
        enum_def,
    )?;
    let result = object_builder.visit_type_extension(
        &mut types_map_builder,
        schema2_path,
        object_ext,
    );

    let enum_type = test_utils::get_enum_type(
        &mut types_map_builder,
        type_name,
    );

    let err = result.unwrap_err();
    assert_eq!(err, SchemaBuildError::InvalidExtensionType {
        schema_type: GraphQLType::Enum(enum_type.into()),
        extension_loc: loc::FilePosition {
            col: 8,
            file: Box::new(schema2_path.to_path_buf()),
            line: 1,
        }.into(),
    });

    Ok(())
}

#[test]
fn visit_object_extension_preceding_def_of_non_object_type() -> Result<()> {
    let type_name = "TestType";
    let field_name = "field1";
    let field_type = "Int";
    let enum_def =
        test_utils::parse_enum_type_def(
            type_name,
            format!("enum {type_name} {{ value1 }}").as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");
    let object_ext =
        test_utils::parse_object_type_ext(
            type_name,
            format!(
                "extend type {type_name} {{
                    {field_name}: {field_type}!,
                }}"
            ).as_str(),
        )
        .expect("no parse error")
        .expect("no object type def found");
    let schema1_path = Path::new("str://0");
    let schema2_path = Path::new("str://1");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut enum_builder = EnumTypeBuilder::new();
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_extension(
        &mut types_map_builder,
        schema2_path,
        object_ext,
    )?;
    enum_builder.visit_type_def(
        &mut types_map_builder,
        schema1_path,
        enum_def,
    )?;
    let result = object_builder.finalize(&mut types_map_builder);

    let enum_type = test_utils::get_enum_type(
        &mut types_map_builder,
        type_name,
    );

    let err = result.unwrap_err();
    assert_eq!(err, SchemaBuildError::InvalidExtensionType {
        schema_type: GraphQLType::Enum(enum_type.into()),
        extension_loc: loc::FilePosition {
            col: 8,
            file: Box::new(schema2_path.to_path_buf()),
            line: 1,
        }.into(),
    });

    Ok(())
}
