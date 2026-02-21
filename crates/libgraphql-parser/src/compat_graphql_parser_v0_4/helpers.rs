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

/// Conversion context for `graphql_parser` →
/// libgraphql AST conversions.
///
/// When constructed with `with_source`, byte offsets in
/// produced `SourcePosition`s are computed from the
/// original source text. Without source, byte offsets
/// default to 0.
pub(super) struct FromGpContext<'src> {
    source: Option<&'src str>,
    line_starts: Vec<usize>,
}

impl<'src> FromGpContext<'src> {
    /// Create a context without source text. All byte
    /// offsets will be 0.
    pub(super) fn without_source() -> Self {
        Self {
            source: None,
            line_starts: vec![],
        }
    }

    /// Create a context with source text. Byte offsets
    /// will be computed from (line, col) using the
    /// source.
    pub(super) fn with_source(
        source: &'src str,
    ) -> Self {
        let mut line_starts = vec![0usize];
        for (offset, byte) in source.bytes().enumerate()
        {
            if byte == b'\n' {
                line_starts.push(offset + 1);
            }
        }
        Self {
            source: Some(source),
            line_starts,
        }
    }

    /// Compute the byte offset for a 0-based
    /// (line, col_utf8) pair.
    fn byte_offset_for(
        &self,
        line: usize,
        col_utf8: usize,
    ) -> usize {
        match self.source {
            Some(src)
                if line < self.line_starts.len() =>
            {
                let line_start =
                    self.line_starts[line];
                src[line_start..]
                    .char_indices()
                    .nth(col_utf8)
                    .map(|(off, _)| line_start + off)
                    .unwrap_or(line_start)
            },
            _ => 0,
        }
    }

    /// Create a zero-width `GraphQLSourceSpan` from a
    /// `graphql_parser` `Pos` (1-based → 0-based).
    /// When source is available, byte offsets are
    /// computed from (line, col).
    pub(super) fn span_from_pos(
        &self,
        pos: graphql_parser::Pos,
    ) -> GraphQLSourceSpan {
        let sp = self.source_pos_from_gp(pos);
        GraphQLSourceSpan::new(sp.clone(), sp)
    }

    /// Create a `GraphQLSourceSpan` from a start
    /// and end `graphql_parser` `Pos` pair
    /// (1-based → 0-based). Used for nodes like
    /// `SelectionSet` that carry both start and
    /// end positions.
    pub(super) fn span_from_pos_pair(
        &self,
        start: graphql_parser::Pos,
        end: graphql_parser::Pos,
    ) -> GraphQLSourceSpan {
        GraphQLSourceSpan::new(
            self.source_pos_from_gp(start),
            self.source_pos_from_gp(end),
        )
    }

    /// Convert a `graphql_parser` `Pos` to a
    /// `SourcePosition` (1-based → 0-based).
    fn source_pos_from_gp(
        &self,
        pos: graphql_parser::Pos,
    ) -> SourcePosition {
        let line = pos.line.saturating_sub(1);
        let col = pos.column.saturating_sub(1);
        let byte_off =
            self.byte_offset_for(line, col);
        SourcePosition::new(
            line, col, None, byte_off,
        )
    }

    /// Create a zero-width span at the origin (line 0,
    /// col 0, byte 0). Used for synthetic nodes that
    /// have no position in graphql_parser (e.g.
    /// descriptions, argument sub-nodes).
    pub(super) fn zero_span(
        &self,
    ) -> GraphQLSourceSpan {
        let sp = SourcePosition::new(0, 0, None, 0);
        GraphQLSourceSpan::new(sp.clone(), sp)
    }

    /// Convert a string to an owned `Name<'static>`
    /// with a zero-width span at the origin.
    pub(super) fn string_to_name(
        &self,
        value: &str,
    ) -> ast::Name<'static> {
        ast::Name {
            span: self.zero_span(),
            syntax: None,
            value: Cow::Owned(value.to_owned()),
        }
    }

    /// Convert a string to an owned `Name<'static>`
    /// with a zero-width span derived from `pos`.
    pub(super) fn string_to_name_at(
        &self,
        value: &str,
        pos: graphql_parser::Pos,
    ) -> ast::Name<'static> {
        ast::Name {
            span: self.span_from_pos(pos),
            syntax: None,
            value: Cow::Owned(value.to_owned()),
        }
    }
}

/// Convert an `Option<String>` description to an
/// `Option<StringValue<'static>>`.
pub(super) fn gp_description_to_ast(
    desc: &Option<String>,
    ctx: &FromGpContext<'_>,
) -> Option<ast::StringValue<'static>> {
    desc.as_ref().map(|s| ast::StringValue {
        is_block: false,
        span: ctx.zero_span(),
        syntax: None,
        value: Cow::Owned(s.clone()),
    })
}

/// Convert a `graphql_parser::query::Value` to an
/// `ast::Value<'static>`.
pub(super) fn gp_value_to_ast(
    val: &graphql_parser::query::Value<'static, String>,
    ctx: &FromGpContext<'_>,
) -> ast::Value<'static> {
    use graphql_parser::query::Value as GpValue;
    let zs = ctx.zero_span();
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
                    .map(|v| gp_value_to_ast(v, ctx))
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
                            name: ctx
                                .string_to_name(key),
                            span: ctx.zero_span(),
                            syntax: None,
                            value: gp_value_to_ast(
                                val, ctx,
                            ),
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
                name: ctx.string_to_name(name),
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
    ctx: &FromGpContext<'_>,
) -> ast::TypeAnnotation<'static> {
    use graphql_parser::schema::Type as GpType;
    let zs = ctx.zero_span();
    match ty {
        GpType::NamedType(name) => {
            ast::TypeAnnotation::Named(
                ast::NamedTypeAnnotation {
                    name: ctx.string_to_name(name),
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
                        gp_type_to_ast(inner, ctx),
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
                            name: ctx
                                .string_to_name(name),
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
                                gp_type_to_ast(
                                    elem, ctx,
                                ),
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
                    gp_type_to_ast(inner, ctx)
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
    ctx: &FromGpContext<'_>,
) -> ast::DirectiveAnnotation<'static> {
    ast::DirectiveAnnotation {
        name: ctx
            .string_to_name_at(&dir.name, dir.position),
        span: ctx.span_from_pos(dir.position),
        syntax: None,
        arguments: dir
            .arguments
            .iter()
            .map(|(name, val)| ast::Argument {
                name: ctx.string_to_name(name),
                span: ctx.zero_span(),
                syntax: None,
                value: gp_value_to_ast(val, ctx),
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
    ctx: &FromGpContext<'_>,
) -> Vec<ast::DirectiveAnnotation<'static>> {
    dirs.iter()
        .map(|d| gp_directive_to_ast(d, ctx))
        .collect()
}

/// Convert a `graphql_parser::schema::InputValue` to an
/// `ast::InputValueDefinition<'static>`.
pub(super) fn gp_input_value_to_ast(
    iv: &graphql_parser::schema::InputValue<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::InputValueDefinition<'static> {
    ast::InputValueDefinition {
        default_value: iv
            .default_value
            .as_ref()
            .map(|v| gp_value_to_ast(v, ctx)),
        description: gp_description_to_ast(
            &iv.description,
            ctx,
        ),
        directives: gp_directives_to_ast(
            &iv.directives,
            ctx,
        ),
        name: ctx
            .string_to_name_at(&iv.name, iv.position),
        span: ctx.span_from_pos(iv.position),
        syntax: None,
        value_type: gp_type_to_ast(
            &iv.value_type,
            ctx,
        ),
    }
}

/// Convert a `graphql_parser::schema::Field` to an
/// `ast::FieldDefinition<'static>`.
pub(super) fn gp_field_def_to_ast(
    field: &graphql_parser::schema::Field<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::FieldDefinition<'static> {
    ast::FieldDefinition {
        arguments: field
            .arguments
            .iter()
            .map(|iv| gp_input_value_to_ast(iv, ctx))
            .collect(),
        description: gp_description_to_ast(
            &field.description,
            ctx,
        ),
        directives: gp_directives_to_ast(
            &field.directives,
            ctx,
        ),
        field_type: gp_type_to_ast(
            &field.field_type,
            ctx,
        ),
        name: ctx.string_to_name_at(
            &field.name,
            field.position,
        ),
        span: ctx.span_from_pos(field.position),
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
    ctx: &FromGpContext<'_>,
) -> ast::EnumValueDefinition<'static> {
    ast::EnumValueDefinition {
        description: gp_description_to_ast(
            &ev.description,
            ctx,
        ),
        directives: gp_directives_to_ast(
            &ev.directives,
            ctx,
        ),
        name: ctx
            .string_to_name_at(&ev.name, ev.position),
        span: ctx.span_from_pos(ev.position),
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
