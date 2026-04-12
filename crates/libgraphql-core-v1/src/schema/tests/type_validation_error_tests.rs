use crate::schema::TypeValidationError;
use crate::schema::TypeValidationErrorKind;
use crate::span::Span;

// Verifies CircularInputFieldChain display joins the path with
// " -> " separators.
// Written by Claude Code, reviewed by a human.
#[test]
fn circular_input_field_chain_display() {
    let error = TypeValidationError::new(
        TypeValidationErrorKind::CircularInputFieldChain {
            circular_field_path: vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
            ],
        },
        Span::builtin(),
        vec![],
    );
    let msg = error.to_string();
    assert!(msg.contains("`A` -> `B` -> `C`"), "got: {msg}");
    assert!(msg.contains("circular"), "got: {msg}");
}

// Verifies MissingRecursiveInterfaceImplementation display
// builds a natural-language inheritance chain.
// Written by Claude Code, reviewed by a human.
#[test]
fn missing_recursive_interface_display() {
    let error = TypeValidationError::new(
        TypeValidationErrorKind::MissingRecursiveInterfaceImplementation {
            inheritance_path: vec![
                "Resource".to_string(),
                "Node".to_string(),
            ],
            missing_recursive_interface_name: "Entity".to_string(),
            type_name: "Image".to_string(),
        },
        Span::builtin(),
        vec![],
    );
    let msg = error.to_string();
    assert!(msg.contains("Image"), "got: {msg}");
    assert!(msg.contains("Resource"), "got: {msg}");
    assert!(msg.contains("Node"), "got: {msg}");
    assert!(msg.contains("Entity"), "got: {msg}");
    assert!(
        msg.contains("which implements"),
        "got: {msg}",
    );
}

// Verifies InvalidParameterWithOutputOnlyType includes type
// and field context in the message.
// Written by Claude Code, reviewed by a human.
#[test]
fn invalid_parameter_output_type_display() {
    let error = TypeValidationError::new(
        TypeValidationErrorKind::InvalidParameterWithOutputOnlyType {
            field_name: "users".to_string(),
            invalid_type_name: "User".to_string(),
            parameter_name: "filter".to_string(),
            type_name: "Query".to_string(),
        },
        Span::builtin(),
        vec![],
    );
    let msg = error.to_string();
    assert!(msg.contains("Query"), "got: {msg}");
    assert!(msg.contains("users"), "got: {msg}");
    assert!(msg.contains("filter"), "got: {msg}");
    assert!(msg.contains("User"), "got: {msg}");
    assert!(msg.contains("not an input type"), "got: {msg}");
}
