//! Reverse schema conversion: `graphql_parser` v0.4
//! schema `Document` → libgraphql AST.

use crate::ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_description_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_directive_location_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_directives_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_enum_value_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_field_def_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_input_value_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::FromGpContext;

/// Convert a `graphql_parser` schema `Document` to a
/// libgraphql AST `Document`.
///
/// This is the reverse of
/// `to_graphql_parser_schema_ast`. The conversion is
/// lossy:
/// - All `syntax` fields are `None`
/// - Spans are zero-width, derived from `Pos` only
/// - Strings become `Cow::Owned`
/// - `ObjectValue` field ordering is alphabetical
///   (from `BTreeMap`)
/// - Byte offsets are 0 (use
///   `from_graphql_parser_schema_ast_with_source` for
///   accurate byte offsets)
pub fn from_graphql_parser_schema_ast(
    doc: &graphql_parser::schema::Document<
        'static,
        String,
    >,
) -> ast::Document<'static> {
    let ctx = FromGpContext::without_source();
    from_gp_schema_impl(doc, &ctx)
}

/// Like `from_graphql_parser_schema_ast`, but computes
/// byte offsets from the source text for accurate
/// `SourcePosition.byte_offset` values.
pub fn from_graphql_parser_schema_ast_with_source(
    doc: &graphql_parser::schema::Document<
        'static,
        String,
    >,
    source: &str,
) -> ast::Document<'static> {
    let ctx = FromGpContext::with_source(source);
    from_gp_schema_impl(doc, &ctx)
}

fn from_gp_schema_impl(
    doc: &graphql_parser::schema::Document<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::Document<'static> {
    let definitions = doc
        .definitions
        .iter()
        .map(|def| {
            use graphql_parser::schema::Definition
                as GpDef;
            match def {
                GpDef::SchemaDefinition(sd) => {
                    ast::Definition::SchemaDefinition(
                        gp_schema_def_to_ast(sd, ctx),
                    )
                },
                GpDef::TypeDefinition(td) => {
                    ast::Definition::TypeDefinition(
                        gp_type_def_to_ast(td, ctx),
                    )
                },
                GpDef::TypeExtension(te) => {
                    ast::Definition::TypeExtension(
                        gp_type_ext_to_ast(te, ctx),
                    )
                },
                GpDef::DirectiveDefinition(dd) => {
                    ast::Definition::DirectiveDefinition(
                        gp_directive_def_to_ast(
                            dd, ctx,
                        ),
                    )
                },
            }
        })
        .collect();

    ast::Document {
        definitions,
        span: ctx.zero_span(),
        syntax: None,
    }
}

fn gp_schema_def_to_ast(
    sd: &graphql_parser::schema::SchemaDefinition<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::SchemaDefinition<'static> {
    let mut root_ops = Vec::new();
    if let Some(ref name) = sd.query {
        root_ops.push(
            ast::RootOperationTypeDefinition {
                named_type: ctx.string_to_name(name),
                operation_kind:
                    ast::OperationKind::Query,
                span: ctx.span_from_pos(sd.position),
                syntax: None,
            },
        );
    }
    if let Some(ref name) = sd.mutation {
        root_ops.push(
            ast::RootOperationTypeDefinition {
                named_type: ctx.string_to_name(name),
                operation_kind:
                    ast::OperationKind::Mutation,
                span: ctx.span_from_pos(sd.position),
                syntax: None,
            },
        );
    }
    if let Some(ref name) = sd.subscription {
        root_ops.push(
            ast::RootOperationTypeDefinition {
                named_type: ctx.string_to_name(name),
                operation_kind:
                    ast::OperationKind::Subscription,
                span: ctx.span_from_pos(sd.position),
                syntax: None,
            },
        );
    }

    ast::SchemaDefinition {
        description: None,
        directives: gp_directives_to_ast(
            &sd.directives,
            ctx,
        ),
        root_operations: root_ops,
        span: ctx.span_from_pos(sd.position),
        syntax: None,
    }
}

fn gp_type_def_to_ast(
    td: &graphql_parser::schema::TypeDefinition<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::TypeDefinition<'static> {
    use graphql_parser::schema::TypeDefinition as GpTd;
    match td {
        GpTd::Scalar(s) => {
            ast::TypeDefinition::Scalar(
                ast::ScalarTypeDefinition {
                    description: gp_description_to_ast(
                        &s.description,
                        ctx,
                    ),
                    directives: gp_directives_to_ast(
                        &s.directives,
                        ctx,
                    ),
                    name: ctx.string_to_name_at(
                        &s.name,
                        s.position,
                    ),
                    span: ctx
                        .span_from_pos(s.position),
                    syntax: None,
                },
            )
        },
        GpTd::Object(obj) => {
            ast::TypeDefinition::Object(
                ast::ObjectTypeDefinition {
                    description: gp_description_to_ast(
                        &obj.description,
                        ctx,
                    ),
                    directives: gp_directives_to_ast(
                        &obj.directives,
                        ctx,
                    ),
                    fields: obj
                        .fields
                        .iter()
                        .map(|f| {
                            gp_field_def_to_ast(f, ctx)
                        })
                        .collect(),
                    implements: obj
                        .implements_interfaces
                        .iter()
                        .map(|n| {
                            ctx.string_to_name(n)
                        })
                        .collect(),
                    name: ctx.string_to_name_at(
                        &obj.name,
                        obj.position,
                    ),
                    span: ctx
                        .span_from_pos(obj.position),
                    syntax: None,
                },
            )
        },
        GpTd::Interface(iface) => {
            ast::TypeDefinition::Interface(
                ast::InterfaceTypeDefinition {
                    description: gp_description_to_ast(
                        &iface.description,
                        ctx,
                    ),
                    directives: gp_directives_to_ast(
                        &iface.directives,
                        ctx,
                    ),
                    fields: iface
                        .fields
                        .iter()
                        .map(|f| {
                            gp_field_def_to_ast(f, ctx)
                        })
                        .collect(),
                    implements: iface
                        .implements_interfaces
                        .iter()
                        .map(|n| {
                            ctx.string_to_name(n)
                        })
                        .collect(),
                    name: ctx.string_to_name_at(
                        &iface.name,
                        iface.position,
                    ),
                    span: ctx.span_from_pos(
                        iface.position,
                    ),
                    syntax: None,
                },
            )
        },
        GpTd::Union(u) => {
            ast::TypeDefinition::Union(
                ast::UnionTypeDefinition {
                    description: gp_description_to_ast(
                        &u.description,
                        ctx,
                    ),
                    directives: gp_directives_to_ast(
                        &u.directives,
                        ctx,
                    ),
                    members: u
                        .types
                        .iter()
                        .map(|n| {
                            ctx.string_to_name(n)
                        })
                        .collect(),
                    name: ctx.string_to_name_at(
                        &u.name,
                        u.position,
                    ),
                    span: ctx
                        .span_from_pos(u.position),
                    syntax: None,
                },
            )
        },
        GpTd::Enum(e) => {
            ast::TypeDefinition::Enum(
                ast::EnumTypeDefinition {
                    description: gp_description_to_ast(
                        &e.description,
                        ctx,
                    ),
                    directives: gp_directives_to_ast(
                        &e.directives,
                        ctx,
                    ),
                    name: ctx.string_to_name_at(
                        &e.name,
                        e.position,
                    ),
                    span: ctx
                        .span_from_pos(e.position),
                    syntax: None,
                    values: e
                        .values
                        .iter()
                        .map(|ev| {
                            gp_enum_value_to_ast(
                                ev, ctx,
                            )
                        })
                        .collect(),
                },
            )
        },
        GpTd::InputObject(io) => {
            ast::TypeDefinition::InputObject(
                ast::InputObjectTypeDefinition {
                    description: gp_description_to_ast(
                        &io.description,
                        ctx,
                    ),
                    directives: gp_directives_to_ast(
                        &io.directives,
                        ctx,
                    ),
                    fields: io
                        .fields
                        .iter()
                        .map(|iv| {
                            gp_input_value_to_ast(
                                iv, ctx,
                            )
                        })
                        .collect(),
                    name: ctx.string_to_name_at(
                        &io.name,
                        io.position,
                    ),
                    span: ctx
                        .span_from_pos(io.position),
                    syntax: None,
                },
            )
        },
    }
}

fn gp_type_ext_to_ast(
    te: &graphql_parser::schema::TypeExtension<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::TypeExtension<'static> {
    use graphql_parser::schema::TypeExtension as GpTe;
    match te {
        GpTe::Scalar(s) => {
            ast::TypeExtension::Scalar(
                ast::ScalarTypeExtension {
                    directives: gp_directives_to_ast(
                        &s.directives,
                        ctx,
                    ),
                    name: ctx.string_to_name_at(
                        &s.name,
                        s.position,
                    ),
                    span: ctx
                        .span_from_pos(s.position),
                    syntax: None,
                },
            )
        },
        GpTe::Object(obj) => {
            ast::TypeExtension::Object(
                ast::ObjectTypeExtension {
                    directives: gp_directives_to_ast(
                        &obj.directives,
                        ctx,
                    ),
                    fields: obj
                        .fields
                        .iter()
                        .map(|f| {
                            gp_field_def_to_ast(f, ctx)
                        })
                        .collect(),
                    implements: obj
                        .implements_interfaces
                        .iter()
                        .map(|n| {
                            ctx.string_to_name(n)
                        })
                        .collect(),
                    name: ctx.string_to_name_at(
                        &obj.name,
                        obj.position,
                    ),
                    span: ctx
                        .span_from_pos(obj.position),
                    syntax: None,
                },
            )
        },
        GpTe::Interface(iface) => {
            ast::TypeExtension::Interface(
                ast::InterfaceTypeExtension {
                    directives: gp_directives_to_ast(
                        &iface.directives,
                        ctx,
                    ),
                    fields: iface
                        .fields
                        .iter()
                        .map(|f| {
                            gp_field_def_to_ast(f, ctx)
                        })
                        .collect(),
                    implements: iface
                        .implements_interfaces
                        .iter()
                        .map(|n| {
                            ctx.string_to_name(n)
                        })
                        .collect(),
                    name: ctx.string_to_name_at(
                        &iface.name,
                        iface.position,
                    ),
                    span: ctx.span_from_pos(
                        iface.position,
                    ),
                    syntax: None,
                },
            )
        },
        GpTe::Union(u) => {
            ast::TypeExtension::Union(
                ast::UnionTypeExtension {
                    directives: gp_directives_to_ast(
                        &u.directives,
                        ctx,
                    ),
                    members: u
                        .types
                        .iter()
                        .map(|n| {
                            ctx.string_to_name(n)
                        })
                        .collect(),
                    name: ctx.string_to_name_at(
                        &u.name,
                        u.position,
                    ),
                    span: ctx
                        .span_from_pos(u.position),
                    syntax: None,
                },
            )
        },
        GpTe::Enum(e) => {
            ast::TypeExtension::Enum(
                ast::EnumTypeExtension {
                    directives: gp_directives_to_ast(
                        &e.directives,
                        ctx,
                    ),
                    name: ctx.string_to_name_at(
                        &e.name,
                        e.position,
                    ),
                    span: ctx
                        .span_from_pos(e.position),
                    syntax: None,
                    values: e
                        .values
                        .iter()
                        .map(|ev| {
                            gp_enum_value_to_ast(
                                ev, ctx,
                            )
                        })
                        .collect(),
                },
            )
        },
        GpTe::InputObject(io) => {
            ast::TypeExtension::InputObject(
                ast::InputObjectTypeExtension {
                    directives: gp_directives_to_ast(
                        &io.directives,
                        ctx,
                    ),
                    fields: io
                        .fields
                        .iter()
                        .map(|iv| {
                            gp_input_value_to_ast(
                                iv, ctx,
                            )
                        })
                        .collect(),
                    name: ctx.string_to_name_at(
                        &io.name,
                        io.position,
                    ),
                    span: ctx
                        .span_from_pos(io.position),
                    syntax: None,
                },
            )
        },
    }
}

fn gp_directive_def_to_ast(
    dd: &graphql_parser::schema::DirectiveDefinition<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::DirectiveDefinition<'static> {
    ast::DirectiveDefinition {
        arguments: dd
            .arguments
            .iter()
            .map(|iv| gp_input_value_to_ast(iv, ctx))
            .collect(),
        description: gp_description_to_ast(
            &dd.description,
            ctx,
        ),
        locations: dd
            .locations
            .iter()
            .map(|loc| ast::DirectiveLocation {
                kind: gp_directive_location_to_ast(loc),
                span: ctx.span_from_pos(dd.position),
                syntax: None,
            })
            .collect(),
        name: ctx.string_to_name_at(
            &dd.name,
            dd.position,
        ),
        repeatable: dd.repeatable,
        span: ctx.span_from_pos(dd.position),
        syntax: None,
    }
}
