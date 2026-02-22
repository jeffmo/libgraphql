//! Forward query conversion: libgraphql AST →
//! `graphql_parser` v0.4 query `Document`.

use crate::ast;
use crate::compat_graphql_parser_v0_4::helpers::directives_to_gp;
use crate::compat_graphql_parser_v0_4::helpers::pos_from_span;
use crate::compat_graphql_parser_v0_4::helpers::type_annotation_to_gp;
use crate::compat_graphql_parser_v0_4::helpers::value_to_gp;
use crate::GraphQLParseError;
use crate::GraphQLParseErrorKind;
use crate::ParseResult;

fn selection_set_to_gp(
    sel_set: &ast::SelectionSet<'_>,
    errors: &mut Vec<GraphQLParseError>,
) -> graphql_parser::query::SelectionSet<'static, String>
{
    graphql_parser::query::SelectionSet {
        span: (
            pos_from_span(&sel_set.span),
            sel_set
                .span
                .end_exclusive
                .to_ast_pos(),
        ),
        items: sel_set
            .selections
            .iter()
            .map(|s| selection_to_gp(s, errors))
            .collect(),
    }
}

fn selection_to_gp(
    sel: &ast::Selection<'_>,
    errors: &mut Vec<GraphQLParseError>,
) -> graphql_parser::query::Selection<'static, String> {
    use graphql_parser::query::Selection as GpSel;
    match sel {
        ast::Selection::Field(field) => {
            GpSel::Field(
                query_field_to_gp(field, errors),
            )
        },
        ast::Selection::FragmentSpread(frag_spread) => {
            GpSel::FragmentSpread(
                fragment_spread_to_gp(frag_spread),
            )
        },
        ast::Selection::InlineFragment(inline_frag) => {
            GpSel::InlineFragment(
                inline_fragment_to_gp(
                    inline_frag,
                    errors,
                ),
            )
        },
    }
}

fn query_field_to_gp(
    field: &ast::Field<'_>,
    errors: &mut Vec<GraphQLParseError>,
) -> graphql_parser::query::Field<'static, String> {
    graphql_parser::query::Field {
        position: pos_from_span(&field.span),
        alias: field
            .alias
            .as_ref()
            .map(|a| a.value.to_string()),
        name: field.name.value.to_string(),
        arguments: field
            .arguments
            .iter()
            .map(|arg| {
                (
                    arg.name.value.to_string(),
                    value_to_gp(&arg.value),
                )
            })
            .collect(),
        directives: directives_to_gp(
            &field.directives,
        ),
        selection_set: match &field.selection_set {
            Some(sel_set) => {
                selection_set_to_gp(sel_set, errors)
            },
            None => graphql_parser::query::SelectionSet {
                span: (
                    pos_from_span(&field.span),
                    pos_from_span(&field.span),
                ),
                items: vec![],
            },
        },
    }
}

fn fragment_spread_to_gp(
    frag_spread: &ast::FragmentSpread<'_>,
) -> graphql_parser::query::FragmentSpread<
    'static,
    String,
> {
    graphql_parser::query::FragmentSpread {
        position: pos_from_span(&frag_spread.span),
        fragment_name: frag_spread
            .name
            .value
            .to_string(),
        directives: directives_to_gp(
            &frag_spread.directives,
        ),
    }
}

fn inline_fragment_to_gp(
    inline_frag: &ast::InlineFragment<'_>,
    errors: &mut Vec<GraphQLParseError>,
) -> graphql_parser::query::InlineFragment<
    'static,
    String,
> {
    graphql_parser::query::InlineFragment {
        position: pos_from_span(&inline_frag.span),
        type_condition: inline_frag
            .type_condition
            .as_ref()
            .map(type_condition_to_gp),
        directives: directives_to_gp(
            &inline_frag.directives,
        ),
        selection_set: selection_set_to_gp(
            &inline_frag.selection_set,
            errors,
        ),
    }
}

fn type_condition_to_gp(
    type_cond: &ast::TypeCondition<'_>,
) -> graphql_parser::query::TypeCondition<
    'static,
    String,
> {
    graphql_parser::query::TypeCondition::On(
        type_cond.named_type.value.to_string(),
    )
}

fn variable_def_to_gp(
    var_def: &ast::VariableDefinition<'_>,
    errors: &mut Vec<GraphQLParseError>,
) -> graphql_parser::query::VariableDefinition<
    'static,
    String,
> {
    if !var_def.directives.is_empty() {
        errors.push(GraphQLParseError::new(
            "Variable directives cannot be \
             represented in graphql_parser v0.4 AST",
            var_def.span.clone(),
            GraphQLParseErrorKind::UnsupportedFeature {
                feature: "variable directives"
                    .to_string(),
            },
        ));
    }
    graphql_parser::query::VariableDefinition {
        position: pos_from_span(&var_def.span),
        name: var_def.variable.value.to_string(),
        var_type: type_annotation_to_gp(
            &var_def.var_type,
        ),
        default_value: var_def
            .default_value
            .as_ref()
            .map(value_to_gp),
    }
}

fn fragment_def_to_gp(
    frag_def: &ast::FragmentDefinition<'_>,
    errors: &mut Vec<GraphQLParseError>,
) -> graphql_parser::query::FragmentDefinition<
    'static,
    String,
> {
    graphql_parser::query::FragmentDefinition {
        position: pos_from_span(&frag_def.span),
        name: frag_def.name.value.to_string(),
        type_condition: type_condition_to_gp(
            &frag_def.type_condition,
        ),
        directives: directives_to_gp(
            &frag_def.directives,
        ),
        selection_set: selection_set_to_gp(
            &frag_def.selection_set,
            errors,
        ),
    }
}

fn operation_def_to_gp(
    op_def: &ast::OperationDefinition<'_>,
    errors: &mut Vec<GraphQLParseError>,
) -> graphql_parser::query::OperationDefinition<
    'static,
    String,
> {
    use graphql_parser::query::OperationDefinition
        as GpOp;

    let var_defs: Vec<_> = op_def
        .variable_definitions
        .iter()
        .map(|var_def| {
            variable_def_to_gp(var_def, errors)
        })
        .collect();
    let sel_set = selection_set_to_gp(
        &op_def.selection_set,
        errors,
    );
    let dirs = directives_to_gp(&op_def.directives);
    let pos = pos_from_span(&op_def.span);
    let name = op_def
        .name
        .as_ref()
        .map(|n| n.value.to_string());

    // Shorthand query: Query kind with no name, no
    // variable definitions, and no directives maps to
    // the SelectionSet variant.
    let is_shorthand = matches!(
        op_def.operation_kind,
        ast::OperationKind::Query
    ) && name.is_none()
        && op_def.variable_definitions.is_empty()
        && op_def.directives.is_empty();

    if is_shorthand {
        return GpOp::SelectionSet(sel_set);
    }

    match op_def.operation_kind {
        ast::OperationKind::Query => {
            GpOp::Query(graphql_parser::query::Query {
                position: pos,
                name,
                variable_definitions: var_defs,
                directives: dirs,
                selection_set: sel_set,
            })
        },
        ast::OperationKind::Mutation => {
            GpOp::Mutation(
                graphql_parser::query::Mutation {
                    position: pos,
                    name,
                    variable_definitions: var_defs,
                    directives: dirs,
                    selection_set: sel_set,
                },
            )
        },
        ast::OperationKind::Subscription => {
            GpOp::Subscription(
                graphql_parser::query::Subscription {
                    position: pos,
                    name,
                    variable_definitions: var_defs,
                    directives: dirs,
                    selection_set: sel_set,
                },
            )
        },
    }
}

/// Convert a libgraphql AST `Document` to a
/// `graphql_parser` query `Document`.
///
/// Returns `ParseResult` with errors for any features
/// that `graphql_parser` cannot represent:
/// - `VariableDefinition` with non-empty directives
///   (directives dropped)
///
/// Type-system definitions (schema, types, directives,
/// extensions) are silently skipped since they belong in
/// `to_graphql_parser_schema_ast`.
pub fn to_graphql_parser_query_ast(
    doc: &ast::Document<'_>,
) -> ParseResult<
    graphql_parser::query::Document<'static, String>,
> {
    let mut errors: Vec<GraphQLParseError> = Vec::new();
    let mut definitions = Vec::new();

    for def in &doc.definitions {
        match def {
            ast::Definition::FragmentDefinition(
                frag_def,
            ) => {
                definitions.push(
                    graphql_parser::query::Definition
                        ::Fragment(
                        fragment_def_to_gp(
                            frag_def,
                            &mut errors,
                        ),
                    ),
                );
            },
            ast::Definition::OperationDefinition(
                op_def,
            ) => {
                definitions.push(
                    graphql_parser::query::Definition
                        ::Operation(
                        operation_def_to_gp(
                            op_def,
                            &mut errors,
                        ),
                    ),
                );
            },
            // Type-system defs are skipped in query
            // conversion
            ast::Definition::DirectiveDefinition(_)
            | ast::Definition::SchemaDefinition(_)
            | ast::Definition::SchemaExtension(_)
            | ast::Definition::TypeDefinition(_)
            | ast::Definition::TypeExtension(_) => {},
        }
    }

    let gp_doc =
        graphql_parser::query::Document { definitions };

    if errors.is_empty() {
        ParseResult::ok(gp_doc)
    } else {
        ParseResult::recovered(gp_doc, errors)
    }
}
