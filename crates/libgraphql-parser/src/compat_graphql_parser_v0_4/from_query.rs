//! Reverse query conversion: `graphql_parser` v0.4
//! query `Document` → libgraphql AST.

use crate::ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_directives_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_type_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::gp_value_to_ast;
use crate::compat_graphql_parser_v0_4::helpers::span_from_pos;
use crate::compat_graphql_parser_v0_4::helpers::string_to_name;
use crate::compat_graphql_parser_v0_4::helpers::string_to_name_at;
use crate::compat_graphql_parser_v0_4::helpers::zero_span_at_origin;

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
pub fn from_graphql_parser_query_ast(
    doc: &graphql_parser::query::Document<
        'static,
        String,
    >,
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
                        gp_operation_to_ast(op),
                    )
                },
                GpDef::Fragment(frag) => {
                    ast::Definition::FragmentDefinition(
                        gp_fragment_def_to_ast(frag),
                    )
                },
            }
        })
        .collect();

    ast::Document {
        definitions,
        span: zero_span_at_origin(),
        syntax: None,
    }
}

fn gp_selection_set_to_ast(
    ss: &graphql_parser::query::SelectionSet<
        'static,
        String,
    >,
) -> ast::SelectionSet<'static> {
    ast::SelectionSet {
        selections: ss
            .items
            .iter()
            .map(gp_selection_to_ast)
            .collect(),
        span: span_from_pos(ss.span.0),
        syntax: None,
    }
}

fn gp_selection_to_ast(
    sel: &graphql_parser::query::Selection<
        'static,
        String,
    >,
) -> ast::Selection<'static> {
    use graphql_parser::query::Selection as GpSel;
    match sel {
        GpSel::Field(field) => {
            ast::Selection::Field(
                gp_query_field_to_ast(field),
            )
        },
        GpSel::FragmentSpread(spread) => {
            ast::Selection::FragmentSpread(
                gp_fragment_spread_to_ast(spread),
            )
        },
        GpSel::InlineFragment(inline) => {
            ast::Selection::InlineFragment(
                gp_inline_fragment_to_ast(inline),
            )
        },
    }
}

fn gp_query_field_to_ast(
    field: &graphql_parser::query::Field<
        'static,
        String,
    >,
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
        ))
    };

    ast::Field {
        alias: field
            .alias
            .as_ref()
            .map(|a| string_to_name(a)),
        arguments: field
            .arguments
            .iter()
            .map(|(name, val)| ast::Argument {
                name: string_to_name(name),
                span: zero_span_at_origin(),
                syntax: None,
                value: gp_value_to_ast(val),
            })
            .collect(),
        directives: gp_directives_to_ast(
            &field.directives,
        ),
        name: string_to_name_at(
            &field.name,
            field.position,
        ),
        selection_set,
        span: span_from_pos(field.position),
        syntax: None,
    }
}

fn gp_fragment_spread_to_ast(
    spread: &graphql_parser::query::FragmentSpread<
        'static,
        String,
    >,
) -> ast::FragmentSpread<'static> {
    ast::FragmentSpread {
        directives: gp_directives_to_ast(
            &spread.directives,
        ),
        name: string_to_name_at(
            &spread.fragment_name,
            spread.position,
        ),
        span: span_from_pos(spread.position),
        syntax: None,
    }
}

fn gp_inline_fragment_to_ast(
    inline: &graphql_parser::query::InlineFragment<
        'static,
        String,
    >,
) -> ast::InlineFragment<'static> {
    ast::InlineFragment {
        directives: gp_directives_to_ast(
            &inline.directives,
        ),
        selection_set: gp_selection_set_to_ast(
            &inline.selection_set,
        ),
        span: span_from_pos(inline.position),
        syntax: None,
        type_condition: inline
            .type_condition
            .as_ref()
            .map(gp_type_condition_to_ast),
    }
}

fn gp_type_condition_to_ast(
    tc: &graphql_parser::query::TypeCondition<
        'static,
        String,
    >,
) -> ast::TypeCondition<'static> {
    let graphql_parser::query::TypeCondition::On(
        name,
    ) = tc;
    ast::TypeCondition {
        named_type: string_to_name(name),
        span: zero_span_at_origin(),
        syntax: None,
    }
}

fn gp_variable_def_to_ast(
    var_def: &graphql_parser::query::VariableDefinition<
        'static,
        String,
    >,
) -> ast::VariableDefinition<'static> {
    ast::VariableDefinition {
        default_value: var_def
            .default_value
            .as_ref()
            .map(gp_value_to_ast),
        description: None,
        directives: vec![],
        span: span_from_pos(var_def.position),
        syntax: None,
        var_type: gp_type_to_ast(&var_def.var_type),
        variable: string_to_name_at(
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
) -> ast::FragmentDefinition<'static> {
    ast::FragmentDefinition {
        description: None,
        directives: gp_directives_to_ast(
            &frag.directives,
        ),
        name: string_to_name_at(
            &frag.name,
            frag.position,
        ),
        selection_set: gp_selection_set_to_ast(
            &frag.selection_set,
        ),
        span: span_from_pos(frag.position),
        syntax: None,
        type_condition: gp_type_condition_to_ast(
            &frag.type_condition,
        ),
    }
}

fn gp_operation_to_ast(
    op: &graphql_parser::query::OperationDefinition<
        'static,
        String,
    >,
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
                    gp_selection_set_to_ast(ss),
                span: span_from_pos(ss.span.0),
                syntax: None,
                variable_definitions: vec![],
            }
        },
        GpOp::Query(query) => {
            ast::OperationDefinition {
                description: None,
                directives: gp_directives_to_ast(
                    &query.directives,
                ),
                name: query.name.as_ref().map(|n| {
                    string_to_name_at(
                        n,
                        query.position,
                    )
                }),
                operation_kind:
                    ast::OperationKind::Query,
                selection_set:
                    gp_selection_set_to_ast(
                        &query.selection_set,
                    ),
                span: span_from_pos(query.position),
                syntax: None,
                variable_definitions: query
                    .variable_definitions
                    .iter()
                    .map(gp_variable_def_to_ast)
                    .collect(),
            }
        },
        GpOp::Mutation(mutation) => {
            ast::OperationDefinition {
                description: None,
                directives: gp_directives_to_ast(
                    &mutation.directives,
                ),
                name: mutation.name.as_ref().map(|n| {
                    string_to_name_at(
                        n,
                        mutation.position,
                    )
                }),
                operation_kind:
                    ast::OperationKind::Mutation,
                selection_set:
                    gp_selection_set_to_ast(
                        &mutation.selection_set,
                    ),
                span: span_from_pos(
                    mutation.position,
                ),
                syntax: None,
                variable_definitions: mutation
                    .variable_definitions
                    .iter()
                    .map(gp_variable_def_to_ast)
                    .collect(),
            }
        },
        GpOp::Subscription(sub) => {
            ast::OperationDefinition {
                description: None,
                directives: gp_directives_to_ast(
                    &sub.directives,
                ),
                name: sub.name.as_ref().map(|n| {
                    string_to_name_at(
                        n,
                        sub.position,
                    )
                }),
                operation_kind:
                    ast::OperationKind::Subscription,
                selection_set:
                    gp_selection_set_to_ast(
                        &sub.selection_set,
                    ),
                span: span_from_pos(sub.position),
                syntax: None,
                variable_definitions: sub
                    .variable_definitions
                    .iter()
                    .map(gp_variable_def_to_ast)
                    .collect(),
            }
        },
    }
}
