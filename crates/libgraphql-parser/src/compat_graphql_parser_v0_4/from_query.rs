//! Reverse query conversion: `graphql_parser` v0.4
//! query `Document` → libgraphql AST.

use crate::ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_directives_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_type_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_value_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::FromGpContext;

/// Convert a `graphql_parser` query `Document` to a
/// libgraphql AST `Document`.
///
/// This is the reverse of
/// `to_graphql_parser_query_ast`. The conversion is
/// lossy in the same ways as
/// `from_graphql_parser_schema_ast`.
///
/// `graphql_parser` does not support directives on
/// `VariableDefinition`, so the resulting
/// `VariableDefinition.directives` will always be
/// empty.
///
/// Byte offsets are 0. Use
/// `from_graphql_parser_query_ast_with_source` for
/// accurate byte offsets.
pub fn from_graphql_parser_query_ast(
    doc: &graphql_parser::query::Document<
        'static,
        String,
    >,
) -> ast::Document<'static> {
    let ctx = FromGpContext::without_source();
    from_gp_query_impl(doc, &ctx)
}

/// Like `from_graphql_parser_query_ast`, but computes
/// byte offsets from the source text for accurate
/// `SourcePosition.byte_offset` values.
pub fn from_graphql_parser_query_ast_with_source(
    doc: &graphql_parser::query::Document<
        'static,
        String,
    >,
    source: &str,
) -> ast::Document<'static> {
    let ctx = FromGpContext::with_source(source);
    from_gp_query_impl(doc, &ctx)
}

fn from_gp_query_impl(
    doc: &graphql_parser::query::Document<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::Document<'static> {
    let definitions = doc
        .definitions
        .iter()
        .map(|def| {
            use graphql_parser::query::Definition
                as GpDef;
            match def {
                GpDef::Operation(op) => {
                    ast::Definition::OperationDefinition(
                        gp_operation_to_ast(op, ctx),
                    )
                },
                GpDef::Fragment(frag) => {
                    ast::Definition::FragmentDefinition(
                        gp_fragment_def_to_ast(
                            frag, ctx,
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

fn gp_selection_set_to_ast(
    ss: &graphql_parser::query::SelectionSet<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::SelectionSet<'static> {
    ast::SelectionSet {
        selections: ss
            .items
            .iter()
            .map(|s| gp_selection_to_ast(s, ctx))
            .collect(),
        span: ctx.span_from_pos_pair(
            ss.span.0, ss.span.1,
        ),
        syntax: None,
    }
}

fn gp_selection_to_ast(
    sel: &graphql_parser::query::Selection<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::Selection<'static> {
    use graphql_parser::query::Selection as GpSel;
    match sel {
        GpSel::Field(field) => {
            ast::Selection::Field(
                gp_query_field_to_ast(field, ctx),
            )
        },
        GpSel::FragmentSpread(spread) => {
            ast::Selection::FragmentSpread(
                gp_fragment_spread_to_ast(spread, ctx),
            )
        },
        GpSel::InlineFragment(inline) => {
            ast::Selection::InlineFragment(
                gp_inline_fragment_to_ast(inline, ctx),
            )
        },
    }
}

fn gp_query_field_to_ast(
    field: &graphql_parser::query::Field<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::Field<'static> {
    let selection_set = if field
        .selection_set
        .items
        .is_empty()
    {
        None
    } else {
        Some(gp_selection_set_to_ast(
            &field.selection_set,
            ctx,
        ))
    };

    ast::Field {
        alias: field
            .alias
            .as_ref()
            .map(|a| ctx.string_to_name(a)),
        arguments: field
            .arguments
            .iter()
            .map(|(name, val)| ast::Argument {
                name: ctx.string_to_name(name),
                span: ctx.zero_span(),
                syntax: None,
                value: gp_value_to_ast(val, ctx),
            })
            .collect(),
        directives: gp_directives_to_ast(
            &field.directives,
            ctx,
        ),
        name: ctx.string_to_name_at(
            &field.name,
            field.position,
        ),
        selection_set,
        span: ctx.span_from_pos(field.position),
        syntax: None,
    }
}

fn gp_fragment_spread_to_ast(
    spread: &graphql_parser::query::FragmentSpread<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::FragmentSpread<'static> {
    ast::FragmentSpread {
        directives: gp_directives_to_ast(
            &spread.directives,
            ctx,
        ),
        name: ctx.string_to_name_at(
            &spread.fragment_name,
            spread.position,
        ),
        span: ctx.span_from_pos(spread.position),
        syntax: None,
    }
}

fn gp_inline_fragment_to_ast(
    inline: &graphql_parser::query::InlineFragment<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::InlineFragment<'static> {
    ast::InlineFragment {
        directives: gp_directives_to_ast(
            &inline.directives,
            ctx,
        ),
        selection_set: gp_selection_set_to_ast(
            &inline.selection_set,
            ctx,
        ),
        span: ctx.span_from_pos(inline.position),
        syntax: None,
        type_condition: inline
            .type_condition
            .as_ref()
            .map(|tc| {
                gp_type_condition_to_ast(tc, ctx)
            }),
    }
}

fn gp_type_condition_to_ast(
    tc: &graphql_parser::query::TypeCondition<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::TypeCondition<'static> {
    let graphql_parser::query::TypeCondition::On(
        name,
    ) = tc;
    ast::TypeCondition {
        named_type: ctx.string_to_name(name),
        span: ctx.zero_span(),
        syntax: None,
    }
}

fn gp_variable_def_to_ast(
    var_def: &graphql_parser::query::VariableDefinition<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::VariableDefinition<'static> {
    ast::VariableDefinition {
        default_value: var_def
            .default_value
            .as_ref()
            .map(|v| gp_value_to_ast(v, ctx)),
        description: None,
        directives: vec![],
        span: ctx.span_from_pos(var_def.position),
        syntax: None,
        var_type: gp_type_to_ast(
            &var_def.var_type,
            ctx,
        ),
        variable: ctx.string_to_name_at(
            &var_def.name,
            var_def.position,
        ),
    }
}

fn gp_fragment_def_to_ast(
    frag: &graphql_parser::query::FragmentDefinition<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::FragmentDefinition<'static> {
    ast::FragmentDefinition {
        description: None,
        directives: gp_directives_to_ast(
            &frag.directives,
            ctx,
        ),
        name: ctx.string_to_name_at(
            &frag.name,
            frag.position,
        ),
        selection_set: gp_selection_set_to_ast(
            &frag.selection_set,
            ctx,
        ),
        span: ctx.span_from_pos(frag.position),
        syntax: None,
        type_condition: gp_type_condition_to_ast(
            &frag.type_condition,
            ctx,
        ),
    }
}

fn gp_operation_to_ast(
    op: &graphql_parser::query::OperationDefinition<
        'static,
        String,
    >,
    ctx: &FromGpContext<'_>,
) -> ast::OperationDefinition<'static> {
    use graphql_parser::query::OperationDefinition
        as GpOp;
    match op {
        GpOp::SelectionSet(ss) => {
            ast::OperationDefinition {
                description: None,
                directives: vec![],
                name: None,
                operation_kind:
                    ast::OperationKind::Query,
                selection_set:
                    gp_selection_set_to_ast(ss, ctx),
                span: ctx.span_from_pos(ss.span.0),
                syntax: None,
                variable_definitions: vec![],
            }
        },
        GpOp::Query(query) => {
            ast::OperationDefinition {
                description: None,
                directives: gp_directives_to_ast(
                    &query.directives,
                    ctx,
                ),
                name: query.name.as_ref().map(|n| {
                    ctx.string_to_name_at(
                        n,
                        query.position,
                    )
                }),
                operation_kind:
                    ast::OperationKind::Query,
                selection_set:
                    gp_selection_set_to_ast(
                        &query.selection_set,
                        ctx,
                    ),
                span: ctx
                    .span_from_pos(query.position),
                syntax: None,
                variable_definitions: query
                    .variable_definitions
                    .iter()
                    .map(|vd| {
                        gp_variable_def_to_ast(vd, ctx)
                    })
                    .collect(),
            }
        },
        GpOp::Mutation(mutation) => {
            ast::OperationDefinition {
                description: None,
                directives: gp_directives_to_ast(
                    &mutation.directives,
                    ctx,
                ),
                name: mutation.name.as_ref().map(|n| {
                    ctx.string_to_name_at(
                        n,
                        mutation.position,
                    )
                }),
                operation_kind:
                    ast::OperationKind::Mutation,
                selection_set:
                    gp_selection_set_to_ast(
                        &mutation.selection_set,
                        ctx,
                    ),
                span: ctx.span_from_pos(
                    mutation.position,
                ),
                syntax: None,
                variable_definitions: mutation
                    .variable_definitions
                    .iter()
                    .map(|vd| {
                        gp_variable_def_to_ast(vd, ctx)
                    })
                    .collect(),
            }
        },
        GpOp::Subscription(sub) => {
            ast::OperationDefinition {
                description: None,
                directives: gp_directives_to_ast(
                    &sub.directives,
                    ctx,
                ),
                name: sub.name.as_ref().map(|n| {
                    ctx.string_to_name_at(
                        n,
                        sub.position,
                    )
                }),
                operation_kind:
                    ast::OperationKind::Subscription,
                selection_set:
                    gp_selection_set_to_ast(
                        &sub.selection_set,
                        ctx,
                    ),
                span: ctx
                    .span_from_pos(sub.position),
                syntax: None,
                variable_definitions: sub
                    .variable_definitions
                    .iter()
                    .map(|vd| {
                        gp_variable_def_to_ast(vd, ctx)
                    })
                    .collect(),
            }
        },
    }
}
