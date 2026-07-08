use crate::directive_annotation::DirectiveAnnotation;
use crate::error_note::ErrorNoteKind;
use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::schema::TypeValidationErrorKind;
use crate::span::SourceMapId;
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
use crate::value::Value;
use indexmap::IndexMap;
use libgraphql_parser::ByteSpan;

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
// InvalidDirectiveParameterType error.
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
        TypeValidationErrorKind::InvalidDirectiveParameterType {
            directive_name,
            invalid_type_name,
            parameter_name,
        } if directive_name == "myDirective"
            && invalid_type_name == "Result"
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

// Verifies that the InvalidDirectiveParameterType error
// variant produces a sensible Display message that includes
// the directive name (with @), the parameter name, and the
// invalid type name.
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

    // The message should clearly reference @myDirective,
    // the parameter name, the invalid type, and say it's
    // not an input type.
    assert_eq!(
        msg,
        "parameter `input` on directive `@myDirective` \
        has type `Result` which is not an input type",
    );
}

fn deprecated_annotation(span: Span) -> DirectiveAnnotation {
    DirectiveAnnotation {
        arguments: IndexMap::new(),
        name: DirectiveName::new("deprecated"),
        span,
    }
}

fn make_param_with(
    name: &str,
    type_annot: TypeAnnotation,
    default_value: Option<Value>,
    directives: Vec<DirectiveAnnotation>,
) -> ParameterDefinition {
    ParameterDefinition {
        default_value,
        description: None,
        directives,
        name: FieldName::new(name),
        span: Span::dummy(),
        type_annotation: type_annot,
    }
}

// Verifies that `@deprecated` on a required (non-null, no
// default value) parameter of a custom directive definition
// produces a DeprecatedRequiredDirectiveParameter error whose
// span points at the `@deprecated` annotation, with help + spec
// notes.
// https://spec.graphql.org/September2025/#sec--deprecated
// Written by Claude Code, reviewed by a human.
#[test]
fn deprecated_required_directive_parameter() {
    let deprecated_span = Span::new(
        ByteSpan::new(40, 51),
        SourceMapId(1),
    );
    let mut params = IndexMap::new();
    params.insert(
        FieldName::new("oldArg"),
        make_param_with(
            "oldArg",
            TypeAnnotation::named("String", /* nullable = */ false),
            None,
            vec![deprecated_annotation(deprecated_span)],
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
    assert_eq!(errors.len(), 1, "unexpected errors: {errors:?}");
    assert!(matches!(
        errors[0].kind(),
        TypeValidationErrorKind::DeprecatedRequiredDirectiveParameter {
            directive_name,
            parameter_name,
        } if directive_name == "myDirective"
            && parameter_name == "oldArg"
    ));
    assert_eq!(errors[0].span(), deprecated_span);
    assert!(
        errors[0].notes().iter().any(|note| {
            note.kind == ErrorNoteKind::Spec
                && note.message.contains("sec--deprecated")
        }),
        "expected a spec note, got: {:?}",
        errors[0].notes(),
    );
    assert!(
        errors[0].notes().iter().any(|note| {
            note.kind == ErrorNoteKind::Help
        }),
        "expected a help note, got: {:?}",
        errors[0].notes(),
    );
}

// Verifies that `@deprecated` on an OPTIONAL parameter of a
// custom directive definition is valid: both a nullable
// parameter and a non-null parameter with a default value may
// be deprecated.
// https://spec.graphql.org/September2025/#sec--deprecated
// Written by Claude Code, reviewed by a human.
#[test]
fn deprecated_optional_directive_parameters_ok() {
    let mut params = IndexMap::new();
    params.insert(
        FieldName::new("nullableArg"),
        make_param_with(
            "nullableArg",
            TypeAnnotation::named("String", /* nullable = */ true),
            None,
            vec![deprecated_annotation(Span::dummy())],
        ),
    );
    params.insert(
        FieldName::new("defaultedArg"),
        make_param_with(
            "defaultedArg",
            TypeAnnotation::named("String", /* nullable = */ false),
            Some(Value::String("default".to_string())),
            vec![deprecated_annotation(Span::dummy())],
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
