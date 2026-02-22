use crate::ast;
use crate::ast::tests::ast_test_utils::make_name;
use crate::ast::tests::ast_test_utils::zero_span;
use crate::compat_graphql_parser_v0_4::type_annotation_to_gp;

use graphql_parser::schema::Type as GpType;

/// Verifies that a nullable named type (e.g. `String`)
/// converts to `NamedType("String")` without NonNull
/// wrapping.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_type_annotation_to_gp_nullable_named() {
    let lg_type = ast::TypeAnnotation::Named(
        ast::NamedTypeAnnotation {
            name: make_name("String", 0, 6),
            nullability: ast::Nullability::Nullable,
            span: zero_span(),
        },
    );
    assert_eq!(
        type_annotation_to_gp(&lg_type),
        GpType::NamedType("String".to_string()),
    );
}

/// Verifies that a non-null named type (e.g. `String!`)
/// converts to `NonNullType(NamedType("String"))`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_type_annotation_to_gp_non_null_named() {
    let lg_type = ast::TypeAnnotation::Named(
        ast::NamedTypeAnnotation {
            name: make_name("String", 0, 6),
            nullability: ast::Nullability::NonNull {
                syntax: None,
            },
            span: zero_span(),
        },
    );
    assert_eq!(
        type_annotation_to_gp(&lg_type),
        GpType::NonNullType(Box::new(
            GpType::NamedType("String".to_string()),
        )),
    );
}

/// Verifies that a nullable list of non-null named type
/// (e.g. `[String!]`) converts to
/// `ListType(NonNullType(NamedType("String")))`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_type_annotation_to_gp_nullable_list() {
    let lg_type = ast::TypeAnnotation::List(
        ast::ListTypeAnnotation {
            element_type: Box::new(
                ast::TypeAnnotation::Named(
                    ast::NamedTypeAnnotation {
                        name: make_name(
                            "String", 0, 6,
                        ),
                        nullability:
                            ast::Nullability::NonNull {
                                syntax: None,
                            },
                        span: zero_span(),
                    },
                ),
            ),
            nullability: ast::Nullability::Nullable,
            span: zero_span(),
            syntax: None,
        },
    );
    assert_eq!(
        type_annotation_to_gp(&lg_type),
        GpType::ListType(Box::new(
            GpType::NonNullType(Box::new(
                GpType::NamedType("String".to_string()),
            )),
        )),
    );
}

/// Verifies full `[String!]!` nesting: non-null list
/// of non-null named type converts to
/// `NonNullType(ListType(NonNullType(NamedType("String"))))`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_type_annotation_to_gp_non_null_list() {
    let lg_type = ast::TypeAnnotation::List(
        ast::ListTypeAnnotation {
            element_type: Box::new(
                ast::TypeAnnotation::Named(
                    ast::NamedTypeAnnotation {
                        name: make_name(
                            "String", 0, 6,
                        ),
                        nullability:
                            ast::Nullability::NonNull {
                                syntax: None,
                            },
                        span: zero_span(),
                    },
                ),
            ),
            nullability: ast::Nullability::NonNull {
                syntax: None,
            },
            span: zero_span(),
            syntax: None,
        },
    );
    assert_eq!(
        type_annotation_to_gp(&lg_type),
        GpType::NonNullType(Box::new(
            GpType::ListType(Box::new(
                GpType::NonNullType(Box::new(
                    GpType::NamedType(
                        "String".to_string(),
                    ),
                )),
            )),
        )),
    );
}
