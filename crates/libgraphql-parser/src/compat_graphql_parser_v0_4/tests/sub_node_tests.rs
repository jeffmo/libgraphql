use std::borrow::Cow;

use crate::ast;
use crate::ast::tests::ast_test_utils::make_name;
use crate::ast::tests::ast_test_utils::make_span;
use crate::ast::tests::ast_test_utils::zero_span;
use crate::compat_graphql_parser_v0_4::description_to_gp;
use crate::compat_graphql_parser_v0_4::directive_to_gp;
use crate::compat_graphql_parser_v0_4::enum_value_def_to_gp;
use crate::compat_graphql_parser_v0_4::field_def_to_gp;
use crate::compat_graphql_parser_v0_4::input_value_def_to_gp;

/// Shorthand for constructing a 1-based
/// `graphql_parser::Pos`.
fn pos(
    line: usize,
    column: usize,
) -> graphql_parser::Pos {
    graphql_parser::Pos { line, column }
}

/// Verifies that a `DirectiveAnnotation` with arguments
/// converts to a `graphql_parser::Directive` with
/// `(String, Value)` argument tuples and correct
/// position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_directive_to_gp_with_args() {
    let lg_dir = ast::DirectiveAnnotation {
        name: make_name("deprecated", 0, 10),
        span: make_span(5, 2),
        syntax: None,
        arguments: vec![ast::Argument {
            name: make_name("reason", 0, 6),
            span: zero_span(),
            syntax: None,
            value: ast::Value::String(
                ast::StringValue {
                    is_block: false,
                    span: zero_span(),
                    syntax: None,
                    value: Cow::Borrowed("Use newField"),
                },
            ),
        }],
    };
    let gp_dir = directive_to_gp(&lg_dir);
    assert_eq!(gp_dir.position, pos(6, 3));
    assert_eq!(gp_dir.name, "deprecated");
    assert_eq!(gp_dir.arguments.len(), 1);
    assert_eq!(gp_dir.arguments[0].0, "reason");
    assert_eq!(
        gp_dir.arguments[0].1,
        graphql_parser::query::Value::String(
            "Use newField".to_string(),
        ),
    );
}

/// Verifies that a `DirectiveAnnotation` with no arguments
/// converts to a `graphql_parser::Directive` with an empty
/// arguments vec and correct position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_directive_to_gp_no_args() {
    let lg_dir = ast::DirectiveAnnotation {
        name: make_name("skip", 0, 4),
        span: make_span(1, 4),
        syntax: None,
        arguments: vec![],
    };
    let gp_dir = directive_to_gp(&lg_dir);
    assert_eq!(gp_dir.position, pos(2, 5));
    assert_eq!(gp_dir.name, "skip");
    assert!(gp_dir.arguments.is_empty());
}

/// Verifies `description_to_gp` converts `Some(StringValue)`
/// to `Some(String)` and `None` to `None`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_description_to_gp() {
    let some_desc = Some(ast::StringValue {
        is_block: false,
        span: zero_span(),
        syntax: None,
        value: Cow::Borrowed("A description"),
    });
    assert_eq!(
        description_to_gp(&some_desc),
        Some("A description".to_string()),
    );
    assert_eq!(description_to_gp(&None), None);
}

/// Verifies that an `InputValueDefinition` with a default
/// value converts correctly, including the default value,
/// type annotation, and position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_input_value_def_to_gp() {
    let lg_ivd = ast::InputValueDefinition {
        name: make_name("limit", 0, 5),
        description: None,
        directives: vec![],
        default_value: Some(ast::Value::Int(
            ast::IntValue {
                span: zero_span(),
                syntax: None,
                value: 10,
            },
        )),
        span: make_span(3, 4),
        syntax: None,
        value_type: ast::TypeAnnotation::Named(
            ast::NamedTypeAnnotation {
                name: make_name("Int", 0, 3),
                nullability: ast::Nullability::Nullable,
                span: zero_span(),
            },
        ),
    };
    let gp_iv = input_value_def_to_gp(&lg_ivd);
    assert_eq!(gp_iv.position, pos(4, 5));
    assert_eq!(gp_iv.name, "limit");
    assert_eq!(
        gp_iv.value_type,
        graphql_parser::schema::Type::NamedType(
            "Int".to_string(),
        ),
    );
    assert_eq!(
        gp_iv.default_value,
        Some(graphql_parser::query::Value::Int(
            10i32.into(),
        )),
    );
}

/// Verifies that a `FieldDefinition` with arguments and
/// directives converts correctly, including position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_field_def_to_gp() {
    let lg_fd = ast::FieldDefinition {
        name: make_name("users", 0, 5),
        description: Some(ast::StringValue {
            is_block: false,
            span: zero_span(),
            syntax: None,
            value: Cow::Borrowed("List users"),
        }),
        arguments: vec![ast::InputValueDefinition {
            name: make_name("first", 0, 5),
            description: None,
            directives: vec![],
            default_value: None,
            span: zero_span(),
            syntax: None,
            value_type: ast::TypeAnnotation::Named(
                ast::NamedTypeAnnotation {
                    name: make_name("Int", 0, 3),
                    nullability:
                        ast::Nullability::Nullable,
                    span: zero_span(),
                },
            ),
        }],
        directives: vec![],
        field_type: ast::TypeAnnotation::Named(
            ast::NamedTypeAnnotation {
                name: make_name("User", 0, 4),
                nullability: ast::Nullability::NonNull {
                    syntax: None,
                },
                span: zero_span(),
            },
        ),
        span: make_span(7, 2),
        syntax: None,
    };
    let gp_field = field_def_to_gp(&lg_fd);
    assert_eq!(gp_field.position, pos(8, 3));
    assert_eq!(gp_field.name, "users");
    assert_eq!(
        gp_field.description,
        Some("List users".to_string()),
    );
    assert_eq!(gp_field.arguments.len(), 1);
    assert_eq!(gp_field.arguments[0].name, "first");
    assert_eq!(
        gp_field.field_type,
        graphql_parser::schema::Type::NonNullType(
            Box::new(
                graphql_parser::schema::Type::NamedType(
                    "User".to_string(),
                ),
            ),
        ),
    );
}

/// Verifies that an `EnumValueDefinition` with a
/// description and directive converts correctly,
/// including position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_enum_value_def_to_gp() {
    let lg_evd = ast::EnumValueDefinition {
        name: make_name("ACTIVE", 0, 6),
        description: Some(ast::StringValue {
            is_block: false,
            span: zero_span(),
            syntax: None,
            value: Cow::Borrowed("Active status"),
        }),
        directives: vec![ast::DirectiveAnnotation {
            name: make_name("deprecated", 0, 10),
            span: zero_span(),
            syntax: None,
            arguments: vec![],
        }],
        span: make_span(9, 4),
    };
    let gp_ev = enum_value_def_to_gp(&lg_evd);
    assert_eq!(gp_ev.position, pos(10, 5));
    assert_eq!(gp_ev.name, "ACTIVE");
    assert_eq!(
        gp_ev.description,
        Some("Active status".to_string()),
    );
    assert_eq!(gp_ev.directives.len(), 1);
    assert_eq!(
        gp_ev.directives[0].name,
        "deprecated",
    );
}
