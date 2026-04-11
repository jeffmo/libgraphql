use crate::names::FieldName;
use crate::names::TypeName;
use crate::schema::TypeValidationError;
use crate::schema::TypeValidationErrorKind;
use crate::span::Span;
use crate::types::FieldedTypeData;
use crate::types::GraphQLType;
use crate::types::InputField;
use crate::types::InputObjectType;
use crate::types::InterfaceType;
use crate::types::ObjectType;
use crate::types::ScalarKind;
use crate::types::ScalarType;
use crate::types::TypeAnnotation;
use crate::types::UnionType;
use crate::validators::InputObjectTypeValidator;
use indexmap::IndexMap;

fn string_scalar() -> GraphQLType {
    GraphQLType::Scalar(Box::new(ScalarType {
        description: None,
        directives: vec![],
        kind: ScalarKind::String,
        name: TypeName::new("String"),
        span: Span::builtin(),
    }))
}

fn int_scalar() -> GraphQLType {
    GraphQLType::Scalar(Box::new(ScalarType {
        description: None,
        directives: vec![],
        kind: ScalarKind::Int,
        name: TypeName::new("Int"),
        span: Span::builtin(),
    }))
}

fn make_input_field(
    name: &str,
    parent: &str,
    type_annot: TypeAnnotation,
) -> InputField {
    InputField {
        default_value: None,
        description: None,
        directives: vec![],
        name: FieldName::new(name),
        parent_type_name: TypeName::new(parent),
        span: Span::dummy(),
        type_annotation: type_annot,
    }
}

// Verifies that a valid input object with only input-type fields
// produces no errors.
// https://spec.graphql.org/September2025/#sec-Input-Objects
// Written by Claude Code, reviewed by a human.
#[test]
fn valid_input_object_type() {
    let mut fields = IndexMap::new();
    fields.insert(
        FieldName::new("name"),
        make_input_field(
            "name",
            "CreateUserInput",
            TypeAnnotation::named("String", /* nullable = */ false),
        ),
    );
    fields.insert(
        FieldName::new("age"),
        make_input_field(
            "age",
            "CreateUserInput",
            TypeAnnotation::named("Int", /* nullable = */ true),
        ),
    );
    let input_obj = InputObjectType {
        description: None,
        directives: vec![],
        fields,
        name: TypeName::new("CreateUserInput"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());
    types_map.insert(TypeName::new("Int"), int_scalar());

    let validator = InputObjectTypeValidator::new(
        &input_obj,
        &types_map,
    );
    let errors = validator.validate();
    assert!(
        errors.is_empty(),
        "expected no errors, got: {errors:?}",
    );
}

// Verifies that an input field referencing an Object type
// (output-only) produces an InvalidInputFieldWithOutputType
// error. This is the critical fix over v0 which only checked
// as_object().is_some() -- v1 uses !is_input_type() to also
// reject Interface and Union types.
// https://spec.graphql.org/September2025/#sel-IAHhBXDDBFCAACEB4iG
// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_with_object_type() {
    let result_obj = GraphQLType::Object(Box::new(
        ObjectType(FieldedTypeData {
            description: None,
            directives: vec![],
            fields: IndexMap::new(),
            interfaces: vec![],
            name: TypeName::new("User"),
            span: Span::dummy(),
        }),
    ));

    let mut fields = IndexMap::new();
    fields.insert(
        FieldName::new("user"),
        make_input_field(
            "user",
            "CreateInput",
            TypeAnnotation::named("User", /* nullable = */ true),
        ),
    );
    let input_obj = InputObjectType {
        description: None,
        directives: vec![],
        fields,
        name: TypeName::new("CreateInput"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("User"), result_obj);

    let validator = InputObjectTypeValidator::new(
        &input_obj,
        &types_map,
    );
    let errors = validator.validate();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidInputFieldWithOutputType {
            field_name,
            invalid_type_name,
            parent_type_name,
        } if field_name == "user"
            && invalid_type_name == "User"
            && parent_type_name == "CreateInput"
    ));
}

// Verifies that an input field referencing an Interface type
// (output-only) produces an InvalidInputFieldWithOutputType
// error. This covers the bug in v0 where only Object types
// were rejected.
// https://spec.graphql.org/September2025/#sel-IAHhBXDDBFCAACEB4iG
// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_with_interface_type() {
    let iface = GraphQLType::Interface(Box::new(
        InterfaceType(FieldedTypeData {
            description: None,
            directives: vec![],
            fields: IndexMap::new(),
            interfaces: vec![],
            name: TypeName::new("Node"),
            span: Span::dummy(),
        }),
    ));

    let mut fields = IndexMap::new();
    fields.insert(
        FieldName::new("node"),
        make_input_field(
            "node",
            "SearchInput",
            TypeAnnotation::named("Node", /* nullable = */ true),
        ),
    );
    let input_obj = InputObjectType {
        description: None,
        directives: vec![],
        fields,
        name: TypeName::new("SearchInput"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("Node"), iface);

    let validator = InputObjectTypeValidator::new(
        &input_obj,
        &types_map,
    );
    let errors = validator.validate();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidInputFieldWithOutputType {
            field_name,
            invalid_type_name,
            parent_type_name,
        } if field_name == "node"
            && invalid_type_name == "Node"
            && parent_type_name == "SearchInput"
    ));
}

// Verifies that an input field referencing a Union type
// (output-only) produces an InvalidInputFieldWithOutputType
// error. This covers the bug in v0 where only Object types
// were rejected.
// https://spec.graphql.org/September2025/#sel-IAHhBXDDBFCAACEB4iG
// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_with_union_type() {
    let union_type = GraphQLType::Union(Box::new(UnionType {
        description: None,
        directives: vec![],
        members: vec![],
        name: TypeName::new("SearchResult"),
        span: Span::dummy(),
    }));

    let mut fields = IndexMap::new();
    fields.insert(
        FieldName::new("result"),
        make_input_field(
            "result",
            "FilterInput",
            TypeAnnotation::named(
                "SearchResult",
                /* nullable = */ true,
            ),
        ),
    );
    let input_obj = InputObjectType {
        description: None,
        directives: vec![],
        fields,
        name: TypeName::new("FilterInput"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("SearchResult"), union_type);

    let validator = InputObjectTypeValidator::new(
        &input_obj,
        &types_map,
    );
    let errors = validator.validate();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidInputFieldWithOutputType {
            field_name,
            invalid_type_name,
            parent_type_name,
        } if field_name == "result"
            && invalid_type_name == "SearchResult"
            && parent_type_name == "FilterInput"
    ));
}

// Verifies that an input field referencing an undefined type
// produces an UndefinedTypeName error.
// https://spec.graphql.org/September2025/#sec-Input-Objects
// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_with_undefined_type() {
    let mut fields = IndexMap::new();
    fields.insert(
        FieldName::new("data"),
        make_input_field(
            "data",
            "MyInput",
            TypeAnnotation::named(
                "NonExistent",
                /* nullable = */ true,
            ),
        ),
    );
    let input_obj = InputObjectType {
        description: None,
        directives: vec![],
        fields,
        name: TypeName::new("MyInput"),
        span: Span::dummy(),
    };

    let types_map = IndexMap::new();
    let validator = InputObjectTypeValidator::new(
        &input_obj,
        &types_map,
    );
    let errors = validator.validate();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::UndefinedTypeName {
            undefined_type_name,
        } if undefined_type_name == "NonExistent"
    ));
}

// Verifies that a direct non-nullable circular reference between
// two input objects produces a CircularInputFieldChain error.
// https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn circular_non_nullable_input_field_chain() {
    // A! -> B! -> A! (circular)
    let mut a_fields = IndexMap::new();
    a_fields.insert(
        FieldName::new("b"),
        make_input_field(
            "b",
            "A",
            TypeAnnotation::named("B", /* nullable = */ false),
        ),
    );
    let a_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: a_fields,
        name: TypeName::new("A"),
        span: Span::dummy(),
    };

    let mut b_fields = IndexMap::new();
    b_fields.insert(
        FieldName::new("a"),
        make_input_field(
            "a",
            "B",
            TypeAnnotation::named("A", /* nullable = */ false),
        ),
    );
    let b_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: b_fields,
        name: TypeName::new("B"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("A"),
        GraphQLType::InputObject(Box::new(a_type.clone())),
    );
    types_map.insert(
        TypeName::new("B"),
        GraphQLType::InputObject(Box::new(b_type)),
    );

    let validator = InputObjectTypeValidator::new(
        &a_type,
        &types_map,
    );
    let errors = validator.validate();

    let circular_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(
            e.kind(),
            TypeValidationErrorKind::CircularInputFieldChain { .. }
        ))
        .collect();
    assert_eq!(circular_errors.len(), 1);

    // The path must contain the exact chain that forms the
    // cycle: A.b -> B -> B.a -> A.
    let TypeValidationErrorKind::CircularInputFieldChain {
        circular_field_path,
    } = circular_errors[0].kind()
    else {
        panic!(
            "expected CircularInputFieldChain, got: {:?}",
            circular_errors[0],
        );
    };
    assert_eq!(
        circular_field_path,
        &vec![
            "A.b".to_string(),
            "B".to_string(),
            "B.a".to_string(),
            "A".to_string(),
        ],
        "unexpected circular_field_path: {circular_field_path:?}",
    );

    // Also verify the Display output contains the expected
    // chain segments joined by " -> ".
    let msg = circular_errors[0].to_string();
    assert!(
        msg.contains("`A.b`"),
        "expected message to contain `A.b`: {msg}",
    );
    assert!(
        msg.contains("`B.a`"),
        "expected message to contain `B.a`: {msg}",
    );
    assert!(
        msg.contains(" -> "),
        "expected message to contain path separator ' -> ': {msg}",
    );
}

// Verifies that a self-referencing input object (A -> A) with
// a non-nullable field produces a CircularInputFieldChain error.
// A single-node cycle is the simplest form of circular reference.
// https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn circular_self_reference_detected() {
    let mut a_fields = IndexMap::new();
    a_fields.insert(
        FieldName::new("self_ref"),
        make_input_field(
            "self_ref",
            "A",
            TypeAnnotation::named("A", /* nullable = */ false),
        ),
    );
    let a_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: a_fields,
        name: TypeName::new("A"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("A"),
        GraphQLType::InputObject(Box::new(a_type.clone())),
    );

    let validator = InputObjectTypeValidator::new(
        &a_type,
        &types_map,
    );
    let errors = validator.validate();

    let circular_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(
            e.kind(),
            TypeValidationErrorKind::CircularInputFieldChain { .. }
        ))
        .collect();
    assert_eq!(circular_errors.len(), 1);

    // For a self-reference A.self_ref -> A, the path must be
    // exactly [A.self_ref, A].
    let TypeValidationErrorKind::CircularInputFieldChain {
        circular_field_path,
    } = circular_errors[0].kind()
    else {
        panic!(
            "expected CircularInputFieldChain, got: {:?}",
            circular_errors[0],
        );
    };
    assert_eq!(
        circular_field_path,
        &vec![
            "A.self_ref".to_string(),
            "A".to_string(),
        ],
        "unexpected circular_field_path: {circular_field_path:?}",
    );

    // Also verify the Display output contains the self-reference
    // segment.
    let msg = circular_errors[0].to_string();
    assert!(
        msg.contains("`A.self_ref`"),
        "expected message to contain `A.self_ref`: {msg}",
    );
    assert!(
        msg.contains("`A.self_ref` -> `A`"),
        "expected message to contain '`A.self_ref` -> `A`': {msg}",
    );
}

// Verifies that a three-node circular chain (A -> B -> C -> A)
// with all non-nullable fields produces a CircularInputFieldChain
// error. Longer chains must also be detected.
// https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn circular_three_node_chain_detected() {
    let mut a_fields = IndexMap::new();
    a_fields.insert(
        FieldName::new("b"),
        make_input_field(
            "b",
            "A",
            TypeAnnotation::named("B", /* nullable = */ false),
        ),
    );
    let a_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: a_fields,
        name: TypeName::new("A"),
        span: Span::dummy(),
    };

    let mut b_fields = IndexMap::new();
    b_fields.insert(
        FieldName::new("c"),
        make_input_field(
            "c",
            "B",
            TypeAnnotation::named("C", /* nullable = */ false),
        ),
    );
    let b_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: b_fields,
        name: TypeName::new("B"),
        span: Span::dummy(),
    };

    let mut c_fields = IndexMap::new();
    c_fields.insert(
        FieldName::new("a"),
        make_input_field(
            "a",
            "C",
            TypeAnnotation::named("A", /* nullable = */ false),
        ),
    );
    let c_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: c_fields,
        name: TypeName::new("C"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("A"),
        GraphQLType::InputObject(Box::new(a_type.clone())),
    );
    types_map.insert(
        TypeName::new("B"),
        GraphQLType::InputObject(Box::new(b_type)),
    );
    types_map.insert(
        TypeName::new("C"),
        GraphQLType::InputObject(Box::new(c_type)),
    );

    let validator = InputObjectTypeValidator::new(
        &a_type,
        &types_map,
    );
    let errors = validator.validate();

    let circular_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(
            e.kind(),
            TypeValidationErrorKind::CircularInputFieldChain { .. }
        ))
        .collect();
    assert_eq!(circular_errors.len(), 1);

    // For A.b -> B.c -> C.a -> A, the path must be exactly
    // [A.b, B, B.c, C, C.a, A].
    let TypeValidationErrorKind::CircularInputFieldChain {
        circular_field_path,
    } = circular_errors[0].kind()
    else {
        panic!(
            "expected CircularInputFieldChain, got: {:?}",
            circular_errors[0],
        );
    };
    assert_eq!(
        circular_field_path,
        &vec![
            "A.b".to_string(),
            "B".to_string(),
            "B.c".to_string(),
            "C".to_string(),
            "C.a".to_string(),
            "A".to_string(),
        ],
        "unexpected circular_field_path: {circular_field_path:?}",
    );

    // Also verify the Display output contains each node in the
    // chain.
    let msg = circular_errors[0].to_string();
    assert!(
        msg.contains("`A.b`"),
        "expected message to contain `A.b`: {msg}",
    );
    assert!(
        msg.contains("`B.c`"),
        "expected message to contain `B.c`: {msg}",
    );
    assert!(
        msg.contains("`C.a`"),
        "expected message to contain `C.a`: {msg}",
    );
}

// Verifies that a list type annotation breaks a circular input
// field chain, even when the list and inner type are both
// non-nullable (e.g. [A!]!). Per the September 2025 spec, ANY
// list wrapper breaks an input object cycle because list fields
// can always be satisfied with an empty list.
// https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn list_type_breaks_circular_chain() {
    // input A { b: [A!]! }
    // Non-nullable list of non-nullable A -- should NOT error
    // because the list wrapper breaks the cycle.
    let mut a_fields = IndexMap::new();
    a_fields.insert(
        FieldName::new("b"),
        make_input_field(
            "b",
            "A",
            TypeAnnotation::list(
                TypeAnnotation::named(
                    "A",
                    /* nullable = */ false,
                ),
                /* nullable = */ false,
            ),
        ),
    );
    let a_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: a_fields,
        name: TypeName::new("A"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("A"),
        GraphQLType::InputObject(Box::new(a_type.clone())),
    );

    let validator = InputObjectTypeValidator::new(
        &a_type,
        &types_map,
    );
    let errors = validator.validate();
    assert!(
        errors.is_empty(),
        "expected no errors (list breaks cycle), got: {errors:?}",
    );
}

// Verifies that a nullable field breaks a circular reference
// chain and produces no CircularInputFieldChain error.
// https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn nullable_field_breaks_circular_chain() {
    // A -> B (nullable) -> A (non-null)
    // The nullable B field breaks the cycle.
    let mut a_fields = IndexMap::new();
    a_fields.insert(
        FieldName::new("b"),
        make_input_field(
            "b",
            "A",
            // nullable breaks cycle
            TypeAnnotation::named("B", /* nullable = */ true),
        ),
    );
    let a_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: a_fields,
        name: TypeName::new("A"),
        span: Span::dummy(),
    };

    let mut b_fields = IndexMap::new();
    b_fields.insert(
        FieldName::new("a"),
        make_input_field(
            "a",
            "B",
            TypeAnnotation::named("A", /* nullable = */ false),
        ),
    );
    let b_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: b_fields,
        name: TypeName::new("B"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("A"),
        GraphQLType::InputObject(Box::new(a_type.clone())),
    );
    types_map.insert(
        TypeName::new("B"),
        GraphQLType::InputObject(Box::new(b_type)),
    );

    let validator = InputObjectTypeValidator::new(
        &a_type,
        &types_map,
    );
    let errors = validator.validate();
    assert!(
        errors.is_empty(),
        "expected no errors, got: {errors:?}",
    );
}

// Regression test for a double-backtick wrapping bug in
// CircularInputFieldChain error messages. The validator
// previously wrapped path items in backticks (`A.b`), and
// then thiserror's #[error] attribute wrapped them again,
// producing double backticks like `` `A.b` ``. After the fix
// the validator emits raw path segments and thiserror adds a
// single layer of backtick formatting.
//
// This test triggers a real circular chain through the
// InputObjectTypeValidator and then inspects the Display
// output of the resulting error to confirm no double backticks
// appear.
//
// https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn circular_chain_error_message_no_double_backticks() {
    // input A { b: B! }
    // input B { a: A! }
    let mut a_fields = IndexMap::new();
    a_fields.insert(
        FieldName::new("b"),
        make_input_field(
            "b",
            "A",
            TypeAnnotation::named("B", /* nullable = */ false),
        ),
    );
    let a_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: a_fields,
        name: TypeName::new("A"),
        span: Span::dummy(),
    };

    let mut b_fields = IndexMap::new();
    b_fields.insert(
        FieldName::new("a"),
        make_input_field(
            "a",
            "B",
            TypeAnnotation::named("A", /* nullable = */ false),
        ),
    );
    let b_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: b_fields,
        name: TypeName::new("B"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("A"),
        GraphQLType::InputObject(Box::new(a_type.clone())),
    );
    types_map.insert(
        TypeName::new("B"),
        GraphQLType::InputObject(Box::new(b_type)),
    );

    let validator = InputObjectTypeValidator::new(
        &a_type,
        &types_map,
    );
    let errors = validator.validate();

    let circular_errors: Vec<&TypeValidationError> = errors
        .iter()
        .filter(|e| matches!(
            e.kind(),
            TypeValidationErrorKind::CircularInputFieldChain { .. }
        ))
        .collect();
    assert_eq!(circular_errors.len(), 1);

    let msg = circular_errors[0].to_string();

    // The message should contain single-backtick-wrapped path
    // segments like `A.b`, NOT double-backtick-wrapped like
    // `` `A.b` ``.
    assert!(
        !msg.contains("``"),
        "error message contains double backticks, indicating \
        the double-wrapping bug has regressed: {msg}",
    );

    // Sanity check that the message still contains the expected
    // path segments.
    assert!(
        msg.contains("`A.b`"),
        "expected `A.b` in error message, got: {msg}",
    );
    assert!(
        msg.contains("`B.a`"),
        "expected `B.a` in error message, got: {msg}",
    );
}

// Regression test for a path-leaking bug in circular
// reference detection. The validator previously used
// extend_from_slice to push 2 items onto the path but only
// called pop() once, so stale entries from the first cycle
// leaked into the path for subsequent cycles.
//
// This test constructs:
//   input A { b: B!, c: C! }
//   input B { a: A! }
//   input C { a: A! }
//
// Two independent cycles exist:
//   A.b -> B.a -> A
//   A.c -> C.a -> A
//
// The error for the "c" cycle must NOT contain "A.b" or "B"
// — those belong to the first cycle only.
//
// https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn circular_chain_no_path_leaking_between_cycles() {
    let mut a_fields = IndexMap::new();
    a_fields.insert(
        FieldName::new("b"),
        make_input_field(
            "b",
            "A",
            TypeAnnotation::named("B", /* nullable = */ false),
        ),
    );
    a_fields.insert(
        FieldName::new("c"),
        make_input_field(
            "c",
            "A",
            TypeAnnotation::named("C", /* nullable = */ false),
        ),
    );
    let a_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: a_fields,
        name: TypeName::new("A"),
        span: Span::dummy(),
    };

    let mut b_fields = IndexMap::new();
    b_fields.insert(
        FieldName::new("a"),
        make_input_field(
            "a",
            "B",
            TypeAnnotation::named("A", /* nullable = */ false),
        ),
    );
    let b_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: b_fields,
        name: TypeName::new("B"),
        span: Span::dummy(),
    };

    let mut c_fields = IndexMap::new();
    c_fields.insert(
        FieldName::new("a"),
        make_input_field(
            "a",
            "C",
            TypeAnnotation::named("A", /* nullable = */ false),
        ),
    );
    let c_type = InputObjectType {
        description: None,
        directives: vec![],
        fields: c_fields,
        name: TypeName::new("C"),
        span: Span::dummy(),
    };

    let mut types_map = IndexMap::new();
    types_map.insert(
        TypeName::new("A"),
        GraphQLType::InputObject(Box::new(a_type.clone())),
    );
    types_map.insert(
        TypeName::new("B"),
        GraphQLType::InputObject(Box::new(b_type)),
    );
    types_map.insert(
        TypeName::new("C"),
        GraphQLType::InputObject(Box::new(c_type)),
    );

    let validator = InputObjectTypeValidator::new(
        &a_type,
        &types_map,
    );
    let errors = validator.validate();

    let circular_errors: Vec<&TypeValidationError> = errors
        .iter()
        .filter(|e| matches!(
            e.kind(),
            TypeValidationErrorKind::CircularInputFieldChain { .. }
        ))
        .collect();
    assert_eq!(
        circular_errors.len(), 2,
        "expected 2 circular chain errors (one per cycle), \
        got: {circular_errors:?}",
    );

    // Collect the Display output of each circular error
    let messages: Vec<String> = circular_errors
        .iter()
        .map(|e| e.to_string())
        .collect();

    // Find the error for the b-cycle (contains "A.b")
    let b_cycle_msg = messages.iter().find(|m| m.contains("A.b"));
    assert!(
        b_cycle_msg.is_some(),
        "expected an error for the A.b -> B.a -> A cycle, \
        got messages: {messages:?}",
    );
    let b_msg = b_cycle_msg.unwrap();
    assert!(
        b_msg.contains("B.a"),
        "b-cycle error should contain B.a: {b_msg}",
    );
    // The b-cycle error must NOT mention "C"
    assert!(
        !b_msg.contains("C"),
        "b-cycle error should not mention C (path leak): {b_msg}",
    );

    // Find the error for the c-cycle (contains "A.c")
    let c_cycle_msg = messages.iter().find(|m| m.contains("A.c"));
    assert!(
        c_cycle_msg.is_some(),
        "expected an error for the A.c -> C.a -> A cycle, \
        got messages: {messages:?}",
    );
    let c_msg = c_cycle_msg.unwrap();
    assert!(
        c_msg.contains("C.a"),
        "c-cycle error should contain C.a: {c_msg}",
    );
    // The c-cycle error must NOT mention "B" -- this is the
    // key regression check. If path entries from the b-cycle
    // leak into the c-cycle path, "B" would appear here.
    assert!(
        !c_msg.contains("B"),
        "c-cycle error should not mention B (path leak from \
        first cycle): {c_msg}",
    );
}
