use crate::schema::TypeValidationError;
use crate::types::tests::test_utils;
use crate::types::InputObjectTypeBuilder;
use crate::types::InputObjectTypeValidator;
use crate::types::ObjectTypeBuilder;
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
        Some(schema_path),
        &input_obj_def,
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
        Some(schema_path),
        &input_obj1_def,
    ).expect("visit input object1 schema def");
    let input_obj_type1 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type1_name,
    );

    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &input_obj2_def,
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
        Some(schema_path),
        &input_obj1_def,
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
        Some(schema_path),
        &input_obj1_def,
    ).expect("visit input object1 schema def");
    let input_obj_type1 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type1_name,
    );

    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &input_obj2_def,
    ).expect("visit input object1 schema def");
    let input_obj_type2 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type2_name,
    );

    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &input_obj3_def,
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
        Some(schema_path),
        &input_obj1_def,
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
        Some(schema_path),
        &input_obj1_def,
    ).expect("visit input object1 schema def");
    let input_obj_type1 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type1_name,
    );

    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &input_obj2_def,
    ).expect("visit input object1 schema def");
    let input_obj_type2 = test_utils::get_input_object_type(
        &mut types_map_builder,
        type2_name,
    );

    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &input_obj3_def,
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

// The following tests were written by Claude Code and reviewed/iterated on by
// @jeffmo before being committed.

/// Validates that input object fields cannot reference Object (output) types.
///
/// Per the GraphQL specification, input object fields must only reference
/// input types (Scalar, Enum, or Input Object). Referencing an output Object
/// type is invalid and must produce an error.
///
/// Spec reference: https://spec.graphql.org/September2025/#sec-Input-Object.Type-Validation
/// "The input field must accept a type where IsInputType(inputFieldType) returns true."
#[test]
fn input_object_field_referencing_output_object_type_does_not_validate() -> Result<()> {
    let output_type_name = "OutputType";
    let input_type_name = "TestInputObject";
    let valid_field_name = "validField";
    let invalid_field_name = "invalidField";

    // Create an Object type (output type)
    let object_def =
        test_utils::parse_object_type_def(
            output_type_name,
            format!(
                "type {output_type_name} {{
                    someField: String,
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");

    // Create an InputObject that references the Object type (invalid)
    let input_obj_def =
        test_utils::parse_input_object_type_def(
            input_type_name,
            format!(
                "input {input_type_name} {{
                    {valid_field_name}: Int,
                    {invalid_field_name}: {output_type_name}!,
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object type def found");

    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();

    // Add the Object type first
    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &object_def,
    ).expect("visit object schema def");

    // Add the InputObject type
    let mut input_obj_builder = InputObjectTypeBuilder::new();
    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &input_obj_def,
    ).expect("visit input object schema def");
    let input_obj_type = test_utils::get_input_object_type(
        &mut types_map_builder,
        input_type_name,
    );

    let validator = InputObjectTypeValidator::new(
        &input_obj_type,
        &types_map_builder.types,
    );
    let errors = validator.validate();

    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0],
        TypeValidationError::InvalidInputFieldWithOutputType {
            field_name,
            invalid_type_name,
            parent_type_name,
            ..
        } if field_name == invalid_field_name
            && invalid_type_name == output_type_name
            && parent_type_name == input_type_name
    ));

    Ok(())
}

/// Validates that input object fields referencing undefined types produce an error.
///
/// When an input object field references a type name that doesn't exist in the
/// schema, the validator must report an UndefinedTypeName error.
///
/// Spec reference: https://spec.graphql.org/September2025/#sec-Input-Object.Type-Validation
/// (Type references must resolve to defined types)
#[test]
fn input_object_field_referencing_undefined_type_does_not_validate() -> Result<()> {
    let input_type_name = "TestInputObject";
    let valid_field_name = "validField";
    let invalid_field_name = "invalidField";
    let undefined_type_name = "NonExistentType";

    let input_obj_def =
        test_utils::parse_input_object_type_def(
            input_type_name,
            format!(
                "input {input_type_name} {{
                    {valid_field_name}: Int,
                    {invalid_field_name}: {undefined_type_name}!,
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
        Some(schema_path),
        &input_obj_def,
    ).expect("visit input object schema def");
    let input_obj_type = test_utils::get_input_object_type(
        &mut types_map_builder,
        input_type_name,
    );

    let validator = InputObjectTypeValidator::new(
        &input_obj_type,
        &types_map_builder.types,
    );
    let errors = validator.validate();

    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0],
        TypeValidationError::UndefinedTypeName {
            undefined_type_name: type_name,
            ..
        } if type_name == undefined_type_name
    ));

    Ok(())
}

/// Validates that the innermost type in a list is checked for input type validity.
///
/// Even when an output Object type is wrapped in a list (e.g., `[OutputType!]!`),
/// the validator must detect that the innermost type is not a valid input type
/// and produce an error.
///
/// Spec reference: https://spec.graphql.org/September2025/#sec-Input-Object.Type-Validation
/// "The input field must accept a type where IsInputType(inputFieldType) returns true."
#[test]
fn input_object_field_referencing_output_type_in_list_does_not_validate() -> Result<()> {
    let output_type_name = "OutputType";
    let input_type_name = "TestInputObject";
    let invalid_field_name = "invalidListField";

    // Create an Object type (output type)
    let object_def =
        test_utils::parse_object_type_def(
            output_type_name,
            format!(
                "type {output_type_name} {{
                    someField: String,
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");

    // Create an InputObject with a list field referencing the Object type
    let input_obj_def =
        test_utils::parse_input_object_type_def(
            input_type_name,
            format!(
                "input {input_type_name} {{
                    {invalid_field_name}: [{output_type_name}!]!,
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object type def found");

    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();

    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &object_def,
    ).expect("visit object schema def");

    let mut input_obj_builder = InputObjectTypeBuilder::new();
    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &input_obj_def,
    ).expect("visit input object schema def");
    let input_obj_type = test_utils::get_input_object_type(
        &mut types_map_builder,
        input_type_name,
    );

    let validator = InputObjectTypeValidator::new(
        &input_obj_type,
        &types_map_builder.types,
    );
    let errors = validator.validate();

    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0],
        TypeValidationError::InvalidInputFieldWithOutputType {
            field_name,
            invalid_type_name,
            ..
        } if field_name == invalid_field_name
            && invalid_type_name == output_type_name
    ));

    Ok(())
}

/// Validates that the validator accumulates and reports multiple errors.
///
/// When an input object has multiple invalid fields (e.g., one referencing an
/// output type and another referencing an undefined type), all errors should
/// be collected and reported rather than stopping at the first error.
///
/// Spec reference: https://spec.graphql.org/September2025/#sec-Input-Object.Type-Validation
#[test]
fn input_object_with_multiple_invalid_fields_reports_all_errors() -> Result<()> {
    let output_type_name = "OutputType";
    let input_type_name = "TestInputObject";
    let output_field_name = "outputField";
    let undefined_field_name = "undefinedField";
    let undefined_type_name = "MissingType";

    // Create an Object type (output type)
    let object_def =
        test_utils::parse_object_type_def(
            output_type_name,
            format!(
                "type {output_type_name} {{
                    someField: String,
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");

    // Create an InputObject with both kinds of invalid fields
    let input_obj_def =
        test_utils::parse_input_object_type_def(
            input_type_name,
            format!(
                "input {input_type_name} {{
                    validField: String,
                    {output_field_name}: {output_type_name}!,
                    {undefined_field_name}: {undefined_type_name}!,
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no input object type def found");

    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();

    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &object_def,
    ).expect("visit object schema def");

    let mut input_obj_builder = InputObjectTypeBuilder::new();
    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &input_obj_def,
    ).expect("visit input object schema def");
    let input_obj_type = test_utils::get_input_object_type(
        &mut types_map_builder,
        input_type_name,
    );

    let validator = InputObjectTypeValidator::new(
        &input_obj_type,
        &types_map_builder.types,
    );
    let errors = validator.validate();

    // Should have exactly 2 errors
    assert_eq!(errors.len(), 2);

    // Check that we have one of each error type
    let has_output_type_error = errors.iter().any(|e| matches!(
        e,
        TypeValidationError::InvalidInputFieldWithOutputType { field_name, .. }
        if field_name == output_field_name
    ));
    let has_undefined_type_error = errors.iter().any(|e| matches!(
        e,
        TypeValidationError::UndefinedTypeName { undefined_type_name: type_name, .. }
        if type_name == undefined_type_name
    ));

    assert!(has_output_type_error, "Expected InvalidInputFieldWithOutputType error");
    assert!(has_undefined_type_error, "Expected UndefinedTypeName error");

    Ok(())
}

/// Validates that recursive validation through nested input objects detects errors.
///
/// When a parent input object contains a field referencing another input object,
/// and that nested input object has an invalid field (referencing an output type),
/// the recursive validation in `validate_fields_recursive()` must detect and
/// report the error when validating from the parent type.
///
/// Spec reference: https://spec.graphql.org/September2025/#sec-Input-Object.Type-Validation
/// "The input field must accept a type where IsInputType(inputFieldType) returns true."
#[test]
fn nested_input_object_field_referencing_output_type_does_not_validate() -> Result<()> {
    let output_type_name = "OutputType";
    let nested_input_name = "NestedInput";
    let parent_input_name = "ParentInput";

    // Create an Object type (output type)
    let object_def =
        test_utils::parse_object_type_def(
            output_type_name,
            format!(
                "type {output_type_name} {{
                    someField: String,
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no object type def found");

    // Create a nested InputObject that references the Object type (invalid)
    let nested_input_def =
        test_utils::parse_input_object_type_def(
            nested_input_name,
            format!(
                "input {nested_input_name} {{
                    invalidField: {output_type_name}!,
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no nested input object type def found");

    // Create a parent InputObject that references the nested one
    let parent_input_def =
        test_utils::parse_input_object_type_def(
            parent_input_name,
            format!(
                "input {parent_input_name} {{
                    nested: {nested_input_name}!,
                }}"
            ).as_str(),
        )
        .expect("parse error")
        .expect("no parent input object type def found");

    let schema_path = Path::new("str://0");

    let mut types_map_builder = TypesMapBuilder::new();

    let mut object_builder = ObjectTypeBuilder::new();
    object_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &object_def,
    ).expect("visit object schema def");

    let mut input_obj_builder = InputObjectTypeBuilder::new();
    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &nested_input_def,
    ).expect("visit nested input object schema def");

    input_obj_builder.visit_type_def(
        &mut types_map_builder,
        Some(schema_path),
        &parent_input_def,
    ).expect("visit parent input object schema def");

    // Validate from the PARENT type to exercise recursive validation code path
    let parent_input_type = test_utils::get_input_object_type(
        &mut types_map_builder,
        parent_input_name,
    );

    let validator = InputObjectTypeValidator::new(
        &parent_input_type,
        &types_map_builder.types,
    );
    let errors = validator.validate();

    // The recursive validation should find the error in the nested type.
    // Note: The current implementation reports the root type being validated
    // as parent_type_name, not the immediate parent of the invalid field.
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0],
        TypeValidationError::InvalidInputFieldWithOutputType {
            field_name,
            invalid_type_name,
            parent_type_name,
            ..
        } if field_name == "invalidField"
            && invalid_type_name == output_type_name
            && parent_type_name == parent_input_name
    ));

    Ok(())
}
