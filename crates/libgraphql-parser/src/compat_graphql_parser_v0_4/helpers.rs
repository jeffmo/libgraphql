//! Shared helper functions for converting between
//! libgraphql AST types and `graphql_parser` v0.4 types.
//!
//! This module contains position converters, value
//! mappers, and sub-node converters used by both the
//! `to_*` (libgraphql → graphql_parser) and `from_*`
//! (graphql_parser → libgraphql) directions.

use std::borrow::Cow;

use crate::ast;
use crate::GraphQLSourceSpan;
use crate::SourcePosition;

// ───────────────────────────────────────────────────
// Position converters
// ───────────────────────────────────────────────────

/// Create a zero-width `GraphQLSourceSpan` from a
/// `graphql_parser` `Pos` (1-based line/col to 0-based).
pub(super) fn span_from_pos(
    pos: graphql_parser::Pos,
) -> GraphQLSourceSpan {
    let sp = SourcePosition::new(
        pos.line.saturating_sub(1),
        pos.column.saturating_sub(1),
        None,
        0,
    );
    GraphQLSourceSpan::new(sp.clone(), sp)
}

/// Convert a `GraphQLSourceSpan` to a `graphql_parser`
/// `Pos` (0-based to 1-based).
pub(super) fn pos_from_span(
    span: &GraphQLSourceSpan,
) -> graphql_parser::Pos {
    span.start_inclusive.to_ast_pos()
}

// ───────────────────────────────────────────────────
// to_gp helpers: libgraphql AST → graphql_parser
// ───────────────────────────────────────────────────

/// Convert an `ast::Value` to a
/// `graphql_parser::query::Value`.
///
/// All semantic content is preserved. Ownership changes
/// from `Cow<str>` to `String` (Loss Inventory item 6).
/// `ObjectValue` field ordering becomes alphabetical via
/// `BTreeMap` (Loss Inventory item 4).
pub(crate) fn value_to_gp(
    val: &ast::Value<'_>,
) -> graphql_parser::query::Value<'static, String> {
    use graphql_parser::query::Value as GpValue;
    match val {
        ast::Value::Boolean(b) => {
            GpValue::Boolean(b.value)
        },
        ast::Value::Enum(e) => {
            GpValue::Enum(e.value.to_string())
        },
        ast::Value::Float(f) => {
            GpValue::Float(f.value)
        },
        ast::Value::Int(i) => {
            GpValue::Int(i.value.into())
        },
        ast::Value::List(l) => GpValue::List(
            l.values.iter().map(value_to_gp).collect(),
        ),
        ast::Value::Null(_) => GpValue::Null,
        ast::Value::Object(o) => {
            let mut map =
                std::collections::BTreeMap::new();
            for field in &o.fields {
                map.insert(
                    field.name.value.to_string(),
                    value_to_gp(&field.value),
                );
            }
            GpValue::Object(map)
        },
        ast::Value::String(s) => {
            GpValue::String(s.value.to_string())
        },
        ast::Value::Variable(var) => {
            GpValue::Variable(
                var.name.value.to_string(),
            )
        },
    }
}

/// Convert an `ast::TypeAnnotation` to a
/// `graphql_parser::schema::Type`.
///
/// Our AST flattens non-null into a `Nullability` field
/// on each annotation node, while `graphql_parser` wraps
/// with a recursive `NonNullType` variant. This function
/// reconstructs the recursive form.
pub(crate) fn type_annotation_to_gp(
    ta: &ast::TypeAnnotation<'_>,
) -> graphql_parser::schema::Type<'static, String> {
    use graphql_parser::schema::Type as GpType;
    match ta {
        ast::TypeAnnotation::Named(n) => {
            let inner = GpType::NamedType(
                n.name.value.to_string(),
            );
            match &n.nullability {
                ast::Nullability::NonNull { .. } => {
                    GpType::NonNullType(Box::new(inner))
                },
                ast::Nullability::Nullable => inner,
            }
        },
        ast::TypeAnnotation::List(l) => {
            let inner = GpType::ListType(Box::new(
                type_annotation_to_gp(&l.element_type),
            ));
            match &l.nullability {
                ast::Nullability::NonNull { .. } => {
                    GpType::NonNullType(Box::new(inner))
                },
                ast::Nullability::Nullable => inner,
            }
        },
    }
}

/// Convert an `ast::DirectiveAnnotation` to a
/// `graphql_parser` `Directive`.
///
/// Arguments become `(String, Value)` tuples.
pub(crate) fn directive_to_gp(
    dir: &ast::DirectiveAnnotation<'_>,
) -> graphql_parser::query::Directive<'static, String> {
    graphql_parser::query::Directive {
        position: pos_from_span(&dir.span),
        name: dir.name.value.to_string(),
        arguments: dir
            .arguments
            .iter()
            .map(|arg| {
                (
                    arg.name.value.to_string(),
                    value_to_gp(&arg.value),
                )
            })
            .collect(),
    }
}

/// Convert a slice of `DirectiveAnnotation` to a vec
/// of `graphql_parser` `Directive`.
pub(super) fn directives_to_gp(
    dirs: &[ast::DirectiveAnnotation<'_>],
) -> Vec<
    graphql_parser::query::Directive<'static, String>,
> {
    dirs.iter().map(directive_to_gp).collect()
}

/// Convert an optional `StringValue` description to an
/// `Option<String>`.
pub(crate) fn description_to_gp(
    desc: &Option<ast::StringValue<'_>>,
) -> Option<String> {
    desc.as_ref().map(|s| s.value.to_string())
}

/// Convert an `ast::InputValueDefinition` to a
/// `graphql_parser::schema::InputValue`.
pub(crate) fn input_value_def_to_gp(
    ivd: &ast::InputValueDefinition<'_>,
) -> graphql_parser::schema::InputValue<'static, String>
{
    graphql_parser::schema::InputValue {
        position: pos_from_span(&ivd.span),
        description: description_to_gp(
            &ivd.description,
        ),
        name: ivd.name.value.to_string(),
        value_type: type_annotation_to_gp(
            &ivd.value_type,
        ),
        default_value: ivd
            .default_value
            .as_ref()
            .map(value_to_gp),
        directives: directives_to_gp(&ivd.directives),
    }
}

/// Convert an `ast::FieldDefinition` to a
/// `graphql_parser::schema::Field`.
pub(crate) fn field_def_to_gp(
    fd: &ast::FieldDefinition<'_>,
) -> graphql_parser::schema::Field<'static, String> {
    graphql_parser::schema::Field {
        position: pos_from_span(&fd.span),
        description: description_to_gp(&fd.description),
        name: fd.name.value.to_string(),
        arguments: fd
            .arguments
            .iter()
            .map(input_value_def_to_gp)
            .collect(),
        field_type: type_annotation_to_gp(&fd.field_type),
        directives: directives_to_gp(&fd.directives),
    }
}

/// Convert an `ast::EnumValueDefinition` to a
/// `graphql_parser::schema::EnumValue`.
pub(crate) fn enum_value_def_to_gp(
    evd: &ast::EnumValueDefinition<'_>,
) -> graphql_parser::schema::EnumValue<'static, String> {
    graphql_parser::schema::EnumValue {
        position: pos_from_span(&evd.span),
        description: description_to_gp(
            &evd.description,
        ),
        name: evd.name.value.to_string(),
        directives: directives_to_gp(&evd.directives),
    }
}

/// Convert an `ast::DirectiveLocationKind` to a
/// `graphql_parser::schema::DirectiveLocation`.
pub(super) fn directive_location_to_gp(
    kind: &ast::DirectiveLocationKind,
) -> graphql_parser::schema::DirectiveLocation {
    use graphql_parser::schema::DirectiveLocation as Gp;
    match kind {
        ast::DirectiveLocationKind::ArgumentDefinition => {
            Gp::ArgumentDefinition
        },
        ast::DirectiveLocationKind::Enum => Gp::Enum,
        ast::DirectiveLocationKind::EnumValue => {
            Gp::EnumValue
        },
        ast::DirectiveLocationKind::Field => Gp::Field,
        ast::DirectiveLocationKind::FieldDefinition => {
            Gp::FieldDefinition
        },
        ast::DirectiveLocationKind::FragmentDefinition => {
            Gp::FragmentDefinition
        },
        ast::DirectiveLocationKind::FragmentSpread => {
            Gp::FragmentSpread
        },
        ast::DirectiveLocationKind::InlineFragment => {
            Gp::InlineFragment
        },
        ast::DirectiveLocationKind::InputFieldDefinition => {
            Gp::InputFieldDefinition
        },
        ast::DirectiveLocationKind::InputObject => {
            Gp::InputObject
        },
        ast::DirectiveLocationKind::Interface => {
            Gp::Interface
        },
        ast::DirectiveLocationKind::Mutation => {
            Gp::Mutation
        },
        ast::DirectiveLocationKind::Object => Gp::Object,
        ast::DirectiveLocationKind::Query => Gp::Query,
        ast::DirectiveLocationKind::Scalar => Gp::Scalar,
        ast::DirectiveLocationKind::Schema => Gp::Schema,
        ast::DirectiveLocationKind::Subscription => {
            Gp::Subscription
        },
        ast::DirectiveLocationKind::Union => Gp::Union,
        ast::DirectiveLocationKind::VariableDefinition => {
            Gp::VariableDefinition
        },
    }
}

// ───────────────────────────────────────────────────
// from_* helpers: graphql_parser → libgraphql AST
// ───────────────────────────────────────────────────

/// Create a zero-width span at the origin (line 0,
/// col 0, byte 0). Used for synthetic nodes that have
/// no position in graphql_parser (e.g. descriptions,
/// argument sub-nodes).
pub(super) fn zero_span_at_origin() -> GraphQLSourceSpan {
    let sp = SourcePosition::new(0, 0, None, 0);
    GraphQLSourceSpan::new(sp.clone(), sp)
}

/// Convert a `String` to an owned `Name<'static>` with
/// a zero-width span at the origin.
pub(super) fn string_to_name(
    value: &str,
) -> ast::Name<'static> {
    ast::Name {
        span: zero_span_at_origin(),
        syntax: None,
        value: Cow::Owned(value.to_owned()),
    }
}

/// Convert a `String` to an owned `Name<'static>` with
/// a zero-width span derived from `pos`.
pub(super) fn string_to_name_at(
    value: &str,
    pos: graphql_parser::Pos,
) -> ast::Name<'static> {
    ast::Name {
        span: span_from_pos(pos),
        syntax: None,
        value: Cow::Owned(value.to_owned()),
    }
}

/// Convert an `Option<String>` description to an
/// `Option<StringValue<'static>>`.
pub(super) fn gp_description_to_ast(
    desc: &Option<String>,
) -> Option<ast::StringValue<'static>> {
    desc.as_ref().map(|s| ast::StringValue {
        is_block: false,
        span: zero_span_at_origin(),
        syntax: None,
        value: Cow::Owned(s.clone()),
    })
}

/// Convert a `graphql_parser::query::Value` to an
/// `ast::Value<'static>`.
pub(super) fn gp_value_to_ast(
    val: &graphql_parser::query::Value<'static, String>,
) -> ast::Value<'static> {
    use graphql_parser::query::Value as GpValue;
    let zs = zero_span_at_origin();
    match val {
        GpValue::Boolean(b) => {
            ast::Value::Boolean(ast::BooleanValue {
                span: zs,
                syntax: None,
                value: *b,
            })
        },
        GpValue::Enum(e) => {
            ast::Value::Enum(ast::EnumValue {
                span: zs,
                syntax: None,
                value: Cow::Owned(e.clone()),
            })
        },
        GpValue::Float(f) => {
            ast::Value::Float(ast::FloatValue {
                span: zs,
                syntax: None,
                value: *f,
            })
        },
        GpValue::Int(n) => {
            let i64_val =
                n.as_i64().unwrap_or(0);
            ast::Value::Int(ast::IntValue {
                span: zs,
                syntax: None,
                value: i64_val as i32,
            })
        },
        GpValue::List(items) => {
            ast::Value::List(ast::ListValue {
                span: zs,
                syntax: None,
                values: items
                    .iter()
                    .map(gp_value_to_ast)
                    .collect(),
            })
        },
        GpValue::Null => {
            ast::Value::Null(ast::NullValue {
                span: zs,
                syntax: None,
            })
        },
        GpValue::Object(map) => {
            ast::Value::Object(ast::ObjectValue {
                fields: map
                    .iter()
                    .map(|(key, val)| {
                        ast::ObjectField {
                            name: string_to_name(key),
                            span: zero_span_at_origin(),
                            syntax: None,
                            value: gp_value_to_ast(val),
                        }
                    })
                    .collect(),
                span: zs,
                syntax: None,
            })
        },
        GpValue::String(s) => {
            ast::Value::String(ast::StringValue {
                is_block: false,
                span: zs,
                syntax: None,
                value: Cow::Owned(s.clone()),
            })
        },
        GpValue::Variable(name) => {
            ast::Value::Variable(ast::VariableValue {
                name: string_to_name(name),
                span: zs,
                syntax: None,
            })
        },
    }
}

/// Convert a `graphql_parser::schema::Type` to an
/// `ast::TypeAnnotation<'static>`.
pub(super) fn gp_type_to_ast(
    ty: &graphql_parser::schema::Type<'static, String>,
) -> ast::TypeAnnotation<'static> {
    use graphql_parser::schema::Type as GpType;
    let zs = zero_span_at_origin();
    match ty {
        GpType::NamedType(name) => {
            ast::TypeAnnotation::Named(
                ast::NamedTypeAnnotation {
                    name: string_to_name(name),
                    nullability:
                        ast::Nullability::Nullable,
                    span: zs,
                },
            )
        },
        GpType::ListType(inner) => {
            ast::TypeAnnotation::List(
                ast::ListTypeAnnotation {
                    element_type: Box::new(
                        gp_type_to_ast(inner),
                    ),
                    nullability:
                        ast::Nullability::Nullable,
                    span: zs,
                    syntax: None,
                },
            )
        },
        GpType::NonNullType(inner) => {
            match inner.as_ref() {
                GpType::NamedType(name) => {
                    ast::TypeAnnotation::Named(
                        ast::NamedTypeAnnotation {
                            name: string_to_name(name),
                            nullability:
                                ast::Nullability::NonNull {
                                    syntax: None,
                                },
                            span: zs,
                        },
                    )
                },
                GpType::ListType(elem) => {
                    ast::TypeAnnotation::List(
                        ast::ListTypeAnnotation {
                            element_type: Box::new(
                                gp_type_to_ast(elem),
                            ),
                            nullability:
                                ast::Nullability::NonNull {
                                    syntax: None,
                                },
                            span: zs,
                            syntax: None,
                        },
                    )
                },
                GpType::NonNullType(_) => {
                    // Invalid per GraphQL spec but we
                    // handle it gracefully.
                    gp_type_to_ast(inner)
                },
            }
        },
    }
}

/// Convert a `graphql_parser` `Directive` to an
/// `ast::DirectiveAnnotation<'static>`.
fn gp_directive_to_ast(
    dir: &graphql_parser::query::Directive<
        'static,
        String,
    >,
) -> ast::DirectiveAnnotation<'static> {
    ast::DirectiveAnnotation {
        name: string_to_name_at(&dir.name, dir.position),
        span: span_from_pos(dir.position),
        syntax: None,
        arguments: dir
            .arguments
            .iter()
            .map(|(name, val)| ast::Argument {
                name: string_to_name(name),
                span: zero_span_at_origin(),
                syntax: None,
                value: gp_value_to_ast(val),
            })
            .collect(),
    }
}

/// Convert a slice of `graphql_parser` `Directive` to
/// a `Vec<ast::DirectiveAnnotation<'static>>`.
pub(super) fn gp_directives_to_ast(
    dirs: &[graphql_parser::query::Directive<
        'static,
        String,
    >],
) -> Vec<ast::DirectiveAnnotation<'static>> {
    dirs.iter().map(gp_directive_to_ast).collect()
}

/// Convert a `graphql_parser::schema::InputValue` to an
/// `ast::InputValueDefinition<'static>`.
pub(super) fn gp_input_value_to_ast(
    iv: &graphql_parser::schema::InputValue<
        'static,
        String,
    >,
) -> ast::InputValueDefinition<'static> {
    ast::InputValueDefinition {
        default_value: iv
            .default_value
            .as_ref()
            .map(gp_value_to_ast),
        description: gp_description_to_ast(
            &iv.description,
        ),
        directives: gp_directives_to_ast(
            &iv.directives,
        ),
        name: string_to_name_at(&iv.name, iv.position),
        span: span_from_pos(iv.position),
        syntax: None,
        value_type: gp_type_to_ast(&iv.value_type),
    }
}

/// Convert a `graphql_parser::schema::Field` to an
/// `ast::FieldDefinition<'static>`.
pub(super) fn gp_field_def_to_ast(
    field: &graphql_parser::schema::Field<
        'static,
        String,
    >,
) -> ast::FieldDefinition<'static> {
    ast::FieldDefinition {
        arguments: field
            .arguments
            .iter()
            .map(gp_input_value_to_ast)
            .collect(),
        description: gp_description_to_ast(
            &field.description,
        ),
        directives: gp_directives_to_ast(
            &field.directives,
        ),
        field_type: gp_type_to_ast(&field.field_type),
        name: string_to_name_at(
            &field.name,
            field.position,
        ),
        span: span_from_pos(field.position),
        syntax: None,
    }
}

/// Convert a `graphql_parser::schema::EnumValue` to an
/// `ast::EnumValueDefinition<'static>`.
pub(super) fn gp_enum_value_to_ast(
    ev: &graphql_parser::schema::EnumValue<
        'static,
        String,
    >,
) -> ast::EnumValueDefinition<'static> {
    ast::EnumValueDefinition {
        description: gp_description_to_ast(
            &ev.description,
        ),
        directives: gp_directives_to_ast(
            &ev.directives,
        ),
        name: string_to_name_at(&ev.name, ev.position),
        span: span_from_pos(ev.position),
    }
}

/// Convert a `graphql_parser::schema::DirectiveLocation`
/// to an `ast::DirectiveLocationKind`.
pub(super) fn gp_directive_location_to_ast(
    loc: &graphql_parser::schema::DirectiveLocation,
) -> ast::DirectiveLocationKind {
    use graphql_parser::schema::DirectiveLocation as Gp;
    match loc {
        Gp::ArgumentDefinition => {
            ast::DirectiveLocationKind::ArgumentDefinition
        },
        Gp::Enum => {
            ast::DirectiveLocationKind::Enum
        },
        Gp::EnumValue => {
            ast::DirectiveLocationKind::EnumValue
        },
        Gp::Field => {
            ast::DirectiveLocationKind::Field
        },
        Gp::FieldDefinition => {
            ast::DirectiveLocationKind::FieldDefinition
        },
        Gp::FragmentDefinition => {
            ast::DirectiveLocationKind::FragmentDefinition
        },
        Gp::FragmentSpread => {
            ast::DirectiveLocationKind::FragmentSpread
        },
        Gp::InlineFragment => {
            ast::DirectiveLocationKind::InlineFragment
        },
        Gp::InputFieldDefinition => {
            ast::DirectiveLocationKind::InputFieldDefinition
        },
        Gp::InputObject => {
            ast::DirectiveLocationKind::InputObject
        },
        Gp::Interface => {
            ast::DirectiveLocationKind::Interface
        },
        Gp::Mutation => {
            ast::DirectiveLocationKind::Mutation
        },
        Gp::Object => {
            ast::DirectiveLocationKind::Object
        },
        Gp::Query => {
            ast::DirectiveLocationKind::Query
        },
        Gp::Scalar => {
            ast::DirectiveLocationKind::Scalar
        },
        Gp::Schema => {
            ast::DirectiveLocationKind::Schema
        },
        Gp::Subscription => {
            ast::DirectiveLocationKind::Subscription
        },
        Gp::Union => {
            ast::DirectiveLocationKind::Union
        },
        Gp::VariableDefinition => {
            ast::DirectiveLocationKind::VariableDefinition
        },
    }
}
