use crate::schema::TypeValidationError;
use crate::types::tests::test_utils;
use crate::types::InputObjectTypeBuilder;
use crate::types::InputObjectTypeValidator;
use crate::types::TypesMapBuilder;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<TypeValidationError>>;

#[test]
fn basic_flat_input_object_type_validates() -> Result<()> {
    let type_name = "TestInputObject";
    let field1_name = "field1";
    let field1_type = "Int";
    let field2_name = "field2";
    let field2_type = "Int!";
    let input_obj_def =
        test_utils::parse_input_object_type_def(
            type_name,
            format!(
                "input {type_name} {{
                    {field1_name}: {field1_type},
                    {field2_name}: {field2_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object type def found");
    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut input_obj_builder = InputObjectTypeBuilder::new();
    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        input_obj_def,
    ).expect("visit input object schema def");
    let input_obj_type = test_utils::get_input_object_type(
        &mut types_map_builder,
        type_name,
    );

    let validator = InputObjectTypeValidator::new(
        &input_obj_type,
        &types_map_builder.types,
    );
    let errors = validator.validate();

    assert_eq!(errors, vec![]);

    Ok(())
}

#[test]
fn input_object_with_non_recursive_input_obj_field_validates() -> Result<()> {
    let type1_name = "TestInputObject1";
    let type2_name = "TestInputObject2";
    let field1_name = "field1";
    let field1_type = "Int";
    let field2_name = "field2";
    let field2_type = "Int!";
    let field3_name = "field3";
    let field3_type = "Int";
    let field4_name = "field4";
    let field4_type = format!("{type1_name}!");

    let input_obj1_def =
        test_utils::parse_input_object_type_def(
            type1_name,
            format!(
                "input {type1_name} {{
                    {field1_name}: {field1_type},
                    {field2_name}: {field2_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object type def found");

    let input_obj2_def =
        test_utils::parse_input_object_type_def(
            type2_name,
            format!(
                "input {type2_name} {{
                    {field3_name}: {field3_type},
                    {field4_name}: {field4_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object type def found");

    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut input_obj_builder = InputObjectTypeBuilder::new();
    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        input_obj1_def,
    ).expect("visit input object1 schema def");
    let input_obj_type1 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type1_name,
    );

    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        input_obj2_def,
    ).expect("visit input object2 schema def");
    let input_obj_type2 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type1_name,
    );

    let validator1 = InputObjectTypeValidator::new(
        &input_obj_type1,
        &types_map_builder.types,
    );
    let errors1 = validator1.validate();
    assert_eq!(errors1, vec![]);

    let validator2 = InputObjectTypeValidator::new(
        &input_obj_type2,
        &types_map_builder.types,
    );
    let errors2 = validator2.validate();
    assert_eq!(errors2, vec![]);

    Ok(())
}

#[test]
fn input_object_with_nullable_immediately_recursive_input_obj_field_validates() -> Result<()> {
    let type1_name = "TestInputObject1";
    let field1_name = "field1";
    let field1_type = "Int";
    let field2_name = "field2";
    let field2_type = type1_name;

    let input_obj1_def =
        test_utils::parse_input_object_type_def(
            type1_name,
            format!(
                "input {type1_name} {{
                    {field1_name}: {field1_type},
                    {field2_name}: {field2_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object type def found");

    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut input_obj_builder = InputObjectTypeBuilder::new();
    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        input_obj1_def,
    ).expect("visit input object1 schema def");
    let input_obj_type1 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type1_name,
    );

    let validator = InputObjectTypeValidator::new(
        &input_obj_type1,
        &types_map_builder.types,
    );
    let errors = validator.validate();
    assert_eq!(errors, vec![]);

    Ok(())
}

#[test]
fn input_object_with_nullable_distantly_recursive_input_obj_field_validates() -> Result<()> {
    let type1_name = "TestInputObject1";
    let type2_name = "TestInputObject2";
    let type3_name = "TestInputObject3";

    // Type1 fields
    let field1_name = "field1";
    let field1_type = "Int";
    let field2_name = "field2";
    let field2_type = format!("{type2_name}!");

    // Type2 fields
    let field3_name = "field3";
    let field3_type = type3_name;

    // Type3 fields
    let field4_name = "field4";
    let field4_type = type1_name;

    let input_obj1_def =
        test_utils::parse_input_object_type_def(
            type1_name,
            format!(
                "input {type1_name} {{
                    {field1_name}: {field1_type},
                    {field2_name}: {field2_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object1 type def found");
    let input_obj2_def =
        test_utils::parse_input_object_type_def(
            type2_name,
            format!(
                "input {type2_name} {{
                    {field3_name}: {field3_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object2 type def found");
    let input_obj3_def =
        test_utils::parse_input_object_type_def(
            type3_name,
            format!(
                "input {type3_name} {{
                    {field4_name}: {field4_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object3 type def found");

    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut input_obj_builder = InputObjectTypeBuilder::new();
    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        input_obj1_def,
    ).expect("visit input object1 schema def");
    let input_obj_type1 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type1_name,
    );

    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        input_obj2_def,
    ).expect("visit input object1 schema def");
    let input_obj_type2 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type2_name,
    );

    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        input_obj3_def,
    ).expect("visit input object1 schema def");
    let input_obj_type3 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type3_name,
    );

    let validator1 = InputObjectTypeValidator::new(
        &input_obj_type1,
        &types_map_builder.types,
    );
    let errors = validator1.validate();
    assert_eq!(errors, vec![]);

    let validator2 = InputObjectTypeValidator::new(
        &input_obj_type2,
        &types_map_builder.types,
    );
    let errors = validator2.validate();
    assert_eq!(errors, vec![]);

    let validator3 = InputObjectTypeValidator::new(
        &input_obj_type3,
        &types_map_builder.types,
    );
    let errors = validator3.validate();
    assert_eq!(errors, vec![]);

    Ok(())
}

#[test]
fn input_object_with_non_nullable_immediately_recursive_input_obj_field_does_not_validate() -> Result<()> {
    let type1_name = "TestInputObject1";
    let field1_name = "field1";
    let field1_type = "Int";
    let field2_name = "field2";
    let field2_type = format!("{type1_name}!");

    let input_obj1_def =
        test_utils::parse_input_object_type_def(
            type1_name,
            format!(
                "input {type1_name} {{
                    {field1_name}: {field1_type},
                    {field2_name}: {field2_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object type def found");

    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut input_obj_builder = InputObjectTypeBuilder::new();
    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        input_obj1_def,
    ).expect("visit input object1 schema def");
    let input_obj_type1 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type1_name,
    );

    let validator = InputObjectTypeValidator::new(
        &input_obj_type1,
        &types_map_builder.types,
    );
    let errors = validator.validate();
    assert_eq!(errors, vec![
        TypeValidationError::CircularInputFieldChain {
            circular_field_path: vec![
                format!("{type1_name}.{field2_name}"),
                type1_name.to_string(),
            ],
        }
    ]);

    Ok(())
}

#[test]
fn input_object_with_non_nullable_distantly_recursive_input_obj_field_does_not_validate() -> Result<()> {
    let type1_name = "TestInputObject1";
    let type2_name = "TestInputObject2";
    let type3_name = "TestInputObject3";

    // Type1 fields
    let field1_name = "field1";
    let field1_type = "Int";
    let field2_name = "field2";
    let field2_type = format!("{type2_name}!");

    // Type2 fields
    let field3_name = "field3";
    let field3_type = format!("{type3_name}!");

    // Type3 fields
    let field4_name = "field4";
    let field4_type = format!("{type1_name}!");

    let input_obj1_def =
        test_utils::parse_input_object_type_def(
            type1_name,
            format!(
                "input {type1_name} {{
                    {field1_name}: {field1_type},
                    {field2_name}: {field2_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object1 type def found");
    let input_obj2_def =
        test_utils::parse_input_object_type_def(
            type2_name,
            format!(
                "input {type2_name} {{
                    {field3_name}: {field3_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object2 type def found");
    let input_obj3_def =
        test_utils::parse_input_object_type_def(
            type3_name,
            format!(
                "input {type3_name} {{
                    {field4_name}: {field4_type},
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object3 type def found");

    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();
    let mut input_obj_builder = InputObjectTypeBuilder::new();
    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        input_obj1_def,
    ).expect("visit input object1 schema def");
    let input_obj_type1 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type1_name,
    );

    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        input_obj2_def,
    ).expect("visit input object1 schema def");
    let input_obj_type2 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type2_name,
    );

    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        schema_path,
        input_obj3_def,
    ).expect("visit input object1 schema def");
    let input_obj_type3 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type3_name,
    );

    let validator1 = InputObjectTypeValidator::new(
        &input_obj_type1,
        &types_map_builder.types,
    );
    let errors = validator1.validate();
    assert_eq!(errors, vec![
        TypeValidationError::CircularInputFieldChain {
            circular_field_path: vec![
                format!("{type1_name}.{field2_name}"),
                type2_name.to_string(),
                format!("{type2_name}.{field3_name}"),
                type3_name.to_string(),
                format!("{type3_name}.{field4_name}"),
                type1_name.to_string(),
            ],
        },
    ]);

    let validator2 = InputObjectTypeValidator::new(
        &input_obj_type2,
        &types_map_builder.types,
    );
    let errors = validator2.validate();
    assert_eq!(errors, vec![
        TypeValidationError::CircularInputFieldChain {
            circular_field_path: vec![
                format!("{type2_name}.{field3_name}"),
                type3_name.to_string(),
                format!("{type3_name}.{field4_name}"),
                type1_name.to_string(),
                format!("{type1_name}.{field2_name}"),
                type2_name.to_string(),
            ],
        },
    ]);

    let validator3 = InputObjectTypeValidator::new(
        &input_obj_type3,
        &types_map_builder.types,
    );
    let errors = validator3.validate();
    assert_eq!(errors, vec![
        TypeValidationError::CircularInputFieldChain {
            circular_field_path: vec![
                format!("{type3_name}.{field4_name}"),
                type1_name.to_string(),
                format!("{type1_name}.{field2_name}"),
                type2_name.to_string(),
                format!("{type2_name}.{field3_name}"),
                type3_name.to_string(),
            ],
        },
    ]);

    Ok(())
}
