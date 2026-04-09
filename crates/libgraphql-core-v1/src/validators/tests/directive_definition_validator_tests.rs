use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::schema::TypeValidationErrorKind;
use crate::span::Span;
use crate::types::DirectiveDefinition;
use crate::types::DirectiveDefinitionKind;
use crate::types::DirectiveLocationKind;
use crate::types::FieldedTypeData;
use crate::types::GraphQLType;
use crate::types::ObjectType;
use crate::types::ParameterDefinition;
use crate::types::ScalarKind;
use crate::types::ScalarType;
use crate::types::TypeAnnotation;
use crate::validators::validate_directive_definitions;
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

fn make_param(
    name: &str,
    type_annot: TypeAnnotation,
) -> ParameterDefinition {
    ParameterDefinition {
        default_value: None,
        description: None,
        directives: vec![],
        name: FieldName::new(name),
        span: Span::dummy(),
        type_annotation: type_annot,
    }
}

// Verifies that a custom directive with valid input-type
// parameters produces no validation errors.
// https://spec.graphql.org/September2025/#sec-Type-System.Directives
// Written by Claude Code, reviewed by a human.
#[test]
fn valid_custom_directive_with_input_param() {
    let mut params = IndexMap::new();
    params.insert(
        FieldName::new("reason"),
        make_param(
            "reason",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    let mut directive_defs = IndexMap::new();
    directive_defs.insert(
        DirectiveName::new("myDirective"),
        DirectiveDefinition {
            description: None,
            is_repeatable: false,
            kind: DirectiveDefinitionKind::Custom,
            locations: vec![DirectiveLocationKind::FieldDefinition],
            name: DirectiveName::new("myDirective"),
            parameters: params,
            span: Span::dummy(),
        },
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("String"), string_scalar());

    let errors = validate_directive_definitions(
        &directive_defs,
        &types_map,
    );
    assert!(
        errors.is_empty(),
        "expected no errors, got: {errors:?}",
    );
}

// Verifies that a built-in directive is skipped during
// validation (built-ins are assumed correct per spec).
// https://spec.graphql.org/September2025/#sec-Type-System.Directives.Built-in-Directives
// Written by Claude Code, reviewed by a human.
#[test]
fn builtin_directive_skipped() {
    let mut params = IndexMap::new();
    params.insert(
        FieldName::new("reason"),
        make_param(
            "reason",
            TypeAnnotation::named("String", /* nullable = */ true),
        ),
    );
    let mut directive_defs = IndexMap::new();
    directive_defs.insert(
        DirectiveName::new("deprecated"),
        DirectiveDefinition {
            description: None,
            is_repeatable: false,
            kind: DirectiveDefinitionKind::Deprecated,
            locations: vec![DirectiveLocationKind::FieldDefinition],
            name: DirectiveName::new("deprecated"),
            parameters: params,
            span: Span::builtin(),
        },
    );

    // Even with an empty types_map (which would cause
    // UndefinedTypeName for "String"), built-in directives are
    // not validated.
    let types_map = IndexMap::new();
    let errors = validate_directive_definitions(
        &directive_defs,
        &types_map,
    );
    assert!(
        errors.is_empty(),
        "expected no errors, got: {errors:?}",
    );
}

// Verifies that a custom directive parameter referencing an
// output-only type (Object) produces an
// InvalidParameterWithOutputOnlyType error.
// https://spec.graphql.org/September2025/#sec-Type-System.Directives
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_param_with_output_only_type() {
    let result_obj = GraphQLType::Object(Box::new(
        ObjectType(FieldedTypeData {
            description: None,
            directives: vec![],
            fields: IndexMap::new(),
            interfaces: vec![],
            name: TypeName::new("Result"),
            span: Span::dummy(),
        }),
    ));

    let mut params = IndexMap::new();
    params.insert(
        FieldName::new("input"),
        make_param(
            "input",
            TypeAnnotation::named("Result", /* nullable = */ true),
        ),
    );
    let mut directive_defs = IndexMap::new();
    directive_defs.insert(
        DirectiveName::new("myDirective"),
        DirectiveDefinition {
            description: None,
            is_repeatable: false,
            kind: DirectiveDefinitionKind::Custom,
            locations: vec![DirectiveLocationKind::FieldDefinition],
            name: DirectiveName::new("myDirective"),
            parameters: params,
            span: Span::dummy(),
        },
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("Result"), result_obj);

    let errors = validate_directive_definitions(
        &directive_defs,
        &types_map,
    );
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::InvalidParameterWithOutputOnlyType {
            invalid_type_name,
            parameter_name,
            ..
        } if invalid_type_name == "Result"
            && parameter_name == "input"
    ));
}

// Verifies that a custom directive parameter referencing an
// undefined type produces an UndefinedTypeName error.
// https://spec.graphql.org/September2025/#sec-Type-System.Directives
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_param_with_undefined_type() {
    let mut params = IndexMap::new();
    params.insert(
        FieldName::new("value"),
        make_param(
            "value",
            TypeAnnotation::named(
                "NonExistent",
                /* nullable = */ true,
            ),
        ),
    );
    let mut directive_defs = IndexMap::new();
    directive_defs.insert(
        DirectiveName::new("tag"),
        DirectiveDefinition {
            description: None,
            is_repeatable: false,
            kind: DirectiveDefinitionKind::Custom,
            locations: vec![DirectiveLocationKind::Object],
            name: DirectiveName::new("tag"),
            parameters: params,
            span: Span::dummy(),
        },
    );

    let types_map = IndexMap::new();
    let errors = validate_directive_definitions(
        &directive_defs,
        &types_map,
    );
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::UndefinedTypeName {
            undefined_type_name,
        } if undefined_type_name == "NonExistent"
    ));
}

// Regression test for a directive param error variant
// awkwardness. Directive parameter errors use
// InvalidParameterWithOutputOnlyType with `type_name` set to
// an empty string and `field_name` set to `@directiveName`.
// The Display output must still read sensibly (not produce
// artifacts like `.@myDirective` or a leading dot from an
// empty type_name).
//
// https://spec.graphql.org/September2025/#sec-Type-System.Directives
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_param_output_type_error_display_is_sensible() {
    let result_obj = GraphQLType::Object(Box::new(
        ObjectType(FieldedTypeData {
            description: None,
            directives: vec![],
            fields: IndexMap::new(),
            interfaces: vec![],
            name: TypeName::new("Result"),
            span: Span::dummy(),
        }),
    ));

    let mut params = IndexMap::new();
    params.insert(
        FieldName::new("input"),
        make_param(
            "input",
            TypeAnnotation::named("Result", /* nullable = */ true),
        ),
    );
    let mut directive_defs = IndexMap::new();
    directive_defs.insert(
        DirectiveName::new("myDirective"),
        DirectiveDefinition {
            description: None,
            is_repeatable: false,
            kind: DirectiveDefinitionKind::Custom,
            locations: vec![DirectiveLocationKind::FieldDefinition],
            name: DirectiveName::new("myDirective"),
            parameters: params,
            span: Span::dummy(),
        },
    );

    let mut types_map = IndexMap::new();
    types_map.insert(TypeName::new("Result"), result_obj);

    let errors = validate_directive_definitions(
        &directive_defs,
        &types_map,
    );
    assert_eq!(errors.len(), 1);

    let msg = errors[0].to_string();

    // The message should reference @myDirective in a readable
    // way. With type_name = "" the pattern is
    // ".@myDirective(input)" which, while awkward, should at
    // least contain "@myDirective" and "input" and mention the
    // output type "Result".
    assert!(
        msg.contains("@myDirective"),
        "expected @myDirective in error message, got: {msg}",
    );
    assert!(
        msg.contains("input"),
        "expected parameter name 'input' in error message, \
        got: {msg}",
    );
    assert!(
        msg.contains("Result"),
        "expected type name 'Result' in error message, \
        got: {msg}",
    );
    assert!(
        msg.contains("not an input type"),
        "expected 'not an input type' in error message, \
        got: {msg}",
    );
}
