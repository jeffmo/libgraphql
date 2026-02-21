//! Compatibility layer for converting between the
//! libgraphql AST (`crate::ast`) and `graphql_parser`
//! v0.4 types.
//!
//! See [Section 9.2 of the AST design plan](
//! ../../custom-ast-plan.md) for the full conversion
//! specification.

use std::borrow::Cow;

use crate::ast;
use crate::GraphQLParseError;
use crate::GraphQLParseErrorKind;
use crate::GraphQLSourceSpan;
use crate::ParseResult;
use crate::SourcePosition;

/// Create a zero-width `GraphQLSourceSpan` from a
/// `graphql_parser` `Pos` (1-based line/col to 0-based).
fn span_from_pos(
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
fn pos_from_span(
    span: &GraphQLSourceSpan,
) -> graphql_parser::Pos {
    span.start_inclusive.to_ast_pos()
}

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
pub(crate) fn directives_to_gp(
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
fn directive_location_to_gp(
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

fn schema_def_to_gp(
    sd: &ast::SchemaDefinition<'_>,
) -> graphql_parser::schema::Definition<'static, String>
{
    use graphql_parser::schema::Definition as GpDef;
    use graphql_parser::schema::SchemaDefinition
        as GpSchema;

    let mut query = None;
    let mut mutation = None;
    let mut subscription = None;

    for op in &sd.root_operations {
        let name = op.named_type.value.to_string();
        match op.operation_kind {
            ast::OperationKind::Query => {
                query = Some(name)
            },
            ast::OperationKind::Mutation => {
                mutation = Some(name)
            },
            ast::OperationKind::Subscription => {
                subscription = Some(name)
            },
        }
    }

    GpDef::SchemaDefinition(GpSchema {
        position: pos_from_span(&sd.span),
        directives: directives_to_gp(&sd.directives),
        query,
        mutation,
        subscription,
    })
}

fn type_def_to_gp(
    td: &ast::TypeDefinition<'_>,
) -> graphql_parser::schema::Definition<'static, String>
{
    use graphql_parser::schema::Definition as GpDef;
    use graphql_parser::schema::TypeDefinition as GpTd;
    GpDef::TypeDefinition(match td {
        ast::TypeDefinition::Enum(e) => {
            GpTd::Enum(graphql_parser::schema::EnumType {
                position: pos_from_span(&e.span),
                description: description_to_gp(
                    &e.description,
                ),
                name: e.name.value.to_string(),
                directives: directives_to_gp(
                    &e.directives,
                ),
                values: e
                    .values
                    .iter()
                    .map(enum_value_def_to_gp)
                    .collect(),
            })
        },
        ast::TypeDefinition::InputObject(io) => {
            GpTd::InputObject(
                graphql_parser::schema::InputObjectType {
                    position: pos_from_span(&io.span),
                    description: description_to_gp(
                        &io.description,
                    ),
                    name: io.name.value.to_string(),
                    directives: directives_to_gp(
                        &io.directives,
                    ),
                    fields: io
                        .fields
                        .iter()
                        .map(input_value_def_to_gp)
                        .collect(),
                },
            )
        },
        ast::TypeDefinition::Interface(i) => {
            GpTd::Interface(
                graphql_parser::schema::InterfaceType {
                    position: pos_from_span(&i.span),
                    description: description_to_gp(
                        &i.description,
                    ),
                    name: i.name.value.to_string(),
                    implements_interfaces: i
                        .implements
                        .iter()
                        .map(|n| n.value.to_string())
                        .collect(),
                    directives: directives_to_gp(
                        &i.directives,
                    ),
                    fields: i
                        .fields
                        .iter()
                        .map(field_def_to_gp)
                        .collect(),
                },
            )
        },
        ast::TypeDefinition::Object(o) => {
            GpTd::Object(
                graphql_parser::schema::ObjectType {
                    position: pos_from_span(&o.span),
                    description: description_to_gp(
                        &o.description,
                    ),
                    name: o.name.value.to_string(),
                    implements_interfaces: o
                        .implements
                        .iter()
                        .map(|n| n.value.to_string())
                        .collect(),
                    directives: directives_to_gp(
                        &o.directives,
                    ),
                    fields: o
                        .fields
                        .iter()
                        .map(field_def_to_gp)
                        .collect(),
                },
            )
        },
        ast::TypeDefinition::Scalar(s) => {
            GpTd::Scalar(
                graphql_parser::schema::ScalarType {
                    position: pos_from_span(&s.span),
                    description: description_to_gp(
                        &s.description,
                    ),
                    name: s.name.value.to_string(),
                    directives: directives_to_gp(
                        &s.directives,
                    ),
                },
            )
        },
        ast::TypeDefinition::Union(u) => {
            GpTd::Union(
                graphql_parser::schema::UnionType {
                    position: pos_from_span(&u.span),
                    description: description_to_gp(
                        &u.description,
                    ),
                    name: u.name.value.to_string(),
                    directives: directives_to_gp(
                        &u.directives,
                    ),
                    types: u
                        .members
                        .iter()
                        .map(|n| n.value.to_string())
                        .collect(),
                },
            )
        },
    })
}

fn type_ext_to_gp(
    te: &ast::TypeExtension<'_>,
) -> graphql_parser::schema::Definition<'static, String>
{
    use graphql_parser::schema::Definition as GpDef;
    use graphql_parser::schema::TypeExtension as GpTe;
    GpDef::TypeExtension(match te {
        ast::TypeExtension::Enum(e) => GpTe::Enum(
            graphql_parser::schema::EnumTypeExtension {
                position: pos_from_span(&e.span),
                name: e.name.value.to_string(),
                directives: directives_to_gp(
                    &e.directives,
                ),
                values: e
                    .values
                    .iter()
                    .map(enum_value_def_to_gp)
                    .collect(),
            },
        ),
        ast::TypeExtension::InputObject(io) => {
            GpTe::InputObject(
                graphql_parser::schema::InputObjectTypeExtension {
                    position: pos_from_span(&io.span),
                    name: io.name.value.to_string(),
                    directives: directives_to_gp(
                        &io.directives,
                    ),
                    fields: io
                        .fields
                        .iter()
                        .map(input_value_def_to_gp)
                        .collect(),
                },
            )
        },
        ast::TypeExtension::Interface(i) => {
            GpTe::Interface(
                graphql_parser::schema::InterfaceTypeExtension {
                    position: pos_from_span(&i.span),
                    name: i.name.value.to_string(),
                    implements_interfaces: i
                        .implements
                        .iter()
                        .map(|n| n.value.to_string())
                        .collect(),
                    directives: directives_to_gp(
                        &i.directives,
                    ),
                    fields: i
                        .fields
                        .iter()
                        .map(field_def_to_gp)
                        .collect(),
                },
            )
        },
        ast::TypeExtension::Object(o) => {
            GpTe::Object(
                graphql_parser::schema::ObjectTypeExtension {
                    position: pos_from_span(&o.span),
                    name: o.name.value.to_string(),
                    implements_interfaces: o
                        .implements
                        .iter()
                        .map(|n| n.value.to_string())
                        .collect(),
                    directives: directives_to_gp(
                        &o.directives,
                    ),
                    fields: o
                        .fields
                        .iter()
                        .map(field_def_to_gp)
                        .collect(),
                },
            )
        },
        ast::TypeExtension::Scalar(s) => GpTe::Scalar(
            graphql_parser::schema::ScalarTypeExtension {
                position: pos_from_span(&s.span),
                name: s.name.value.to_string(),
                directives: directives_to_gp(
                    &s.directives,
                ),
            },
        ),
        ast::TypeExtension::Union(u) => GpTe::Union(
            graphql_parser::schema::UnionTypeExtension {
                position: pos_from_span(&u.span),
                name: u.name.value.to_string(),
                directives: directives_to_gp(
                    &u.directives,
                ),
                types: u
                    .members
                    .iter()
                    .map(|n| n.value.to_string())
                    .collect(),
            },
        ),
    })
}

fn directive_def_to_gp(
    dd: &ast::DirectiveDefinition<'_>,
) -> graphql_parser::schema::Definition<'static, String>
{
    use graphql_parser::schema::Definition as GpDef;
    GpDef::DirectiveDefinition(
        graphql_parser::schema::DirectiveDefinition {
            position: pos_from_span(&dd.span),
            description: description_to_gp(
                &dd.description,
            ),
            name: dd.name.value.to_string(),
            arguments: dd
                .arguments
                .iter()
                .map(input_value_def_to_gp)
                .collect(),
            repeatable: dd.repeatable,
            locations: dd
                .locations
                .iter()
                .map(|loc| {
                    directive_location_to_gp(&loc.kind)
                })
                .collect(),
        },
    )
}

/// Convert a libgraphql AST `Document` to a
/// `graphql_parser` schema `Document`.
///
/// Returns `ParseResult` with errors for any features
/// that `graphql_parser` cannot represent:
/// - `Definition::SchemaExtension` (dropped entirely)
///
/// Executable definitions (operations, fragments) are
/// silently skipped since they belong in
/// `to_graphql_parser_query_ast`.
pub fn to_graphql_parser_schema_ast(
    doc: &ast::Document<'_>,
) -> ParseResult<
    graphql_parser::schema::Document<'static, String>,
> {
    let mut errors: Vec<GraphQLParseError> = Vec::new();
    let mut definitions = Vec::new();

    for def in &doc.definitions {
        match def {
            ast::Definition::DirectiveDefinition(
                dd,
            ) => {
                definitions.push(
                    directive_def_to_gp(dd),
                );
            },
            ast::Definition::SchemaDefinition(sd) => {
                definitions.push(
                    schema_def_to_gp(sd),
                );
            },
            ast::Definition::SchemaExtension(se) => {
                errors.push(GraphQLParseError::new(
                    "Schema extensions cannot be \
                     represented in graphql_parser \
                     v0.4 AST",
                    se.span.clone(),
                    GraphQLParseErrorKind
                        ::UnsupportedFeature {
                        feature:
                            "schema extension"
                                .to_string(),
                    },
                ));
            },
            ast::Definition::TypeDefinition(td) => {
                definitions.push(
                    type_def_to_gp(td),
                );
            },
            ast::Definition::TypeExtension(te) => {
                definitions.push(
                    type_ext_to_gp(te),
                );
            },
            ast::Definition::FragmentDefinition(_)
            | ast::Definition::OperationDefinition(
                _,
            ) => {},
        }
    }

    let gp_doc =
        graphql_parser::schema::Document { definitions };

    if errors.is_empty() {
        ParseResult::ok(gp_doc)
    } else {
        ParseResult::recovered(gp_doc, errors)
    }
}

fn selection_set_to_gp(
    sel_set: &ast::SelectionSet<'_>,
    errors: &mut Vec<GraphQLParseError>,
) -> graphql_parser::query::SelectionSet<'static, String>
{
    graphql_parser::query::SelectionSet {
        span: (
            pos_from_span(&sel_set.span),
            pos_from_span(&sel_set.span),
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

// ───────────────────────────────────────────────────
// from_* helpers: graphql_parser → libgraphql AST
// ───────────────────────────────────────────────────

/// Create a zero-width span at the origin (line 0,
/// col 0, byte 0). Used for synthetic nodes that have
/// no position in graphql_parser (e.g. descriptions,
/// argument sub-nodes).
fn zero_span_at_origin() -> GraphQLSourceSpan {
    let sp = SourcePosition::new(0, 0, None, 0);
    GraphQLSourceSpan::new(sp.clone(), sp)
}

/// Convert a `String` to an owned `Name<'static>` with
/// a zero-width span at the origin.
fn string_to_name(value: &str) -> ast::Name<'static> {
    ast::Name {
        span: zero_span_at_origin(),
        syntax: None,
        value: Cow::Owned(value.to_owned()),
    }
}

/// Convert a `String` to an owned `Name<'static>` with
/// a zero-width span derived from `pos`.
fn string_to_name_at(
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
fn gp_description_to_ast(
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
fn gp_value_to_ast(
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
fn gp_type_to_ast(
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
fn gp_directives_to_ast(
    dirs: &[graphql_parser::query::Directive<
        'static,
        String,
    >],
) -> Vec<ast::DirectiveAnnotation<'static>> {
    dirs.iter().map(gp_directive_to_ast).collect()
}

/// Convert a `graphql_parser::schema::InputValue` to an
/// `ast::InputValueDefinition<'static>`.
fn gp_input_value_to_ast(
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
fn gp_field_def_to_ast(
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
fn gp_enum_value_to_ast(
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
fn gp_directive_location_to_ast(
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

// ───────────────────────────────────────────────────
// from_graphql_parser_schema_ast
// ───────────────────────────────────────────────────

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
pub fn from_graphql_parser_schema_ast(
    doc: &graphql_parser::schema::Document<
        'static,
        String,
    >,
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
                        gp_schema_def_to_ast(sd),
                    )
                },
                GpDef::TypeDefinition(td) => {
                    ast::Definition::TypeDefinition(
                        gp_type_def_to_ast(td),
                    )
                },
                GpDef::TypeExtension(te) => {
                    ast::Definition::TypeExtension(
                        gp_type_ext_to_ast(te),
                    )
                },
                GpDef::DirectiveDefinition(dd) => {
                    ast::Definition::DirectiveDefinition(
                        gp_directive_def_to_ast(dd),
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

fn gp_schema_def_to_ast(
    sd: &graphql_parser::schema::SchemaDefinition<
        'static,
        String,
    >,
) -> ast::SchemaDefinition<'static> {
    let mut root_ops = Vec::new();
    if let Some(ref name) = sd.query {
        root_ops.push(
            ast::RootOperationTypeDefinition {
                named_type: string_to_name(name),
                operation_kind:
                    ast::OperationKind::Query,
                span: span_from_pos(sd.position),
                syntax: None,
            },
        );
    }
    if let Some(ref name) = sd.mutation {
        root_ops.push(
            ast::RootOperationTypeDefinition {
                named_type: string_to_name(name),
                operation_kind:
                    ast::OperationKind::Mutation,
                span: span_from_pos(sd.position),
                syntax: None,
            },
        );
    }
    if let Some(ref name) = sd.subscription {
        root_ops.push(
            ast::RootOperationTypeDefinition {
                named_type: string_to_name(name),
                operation_kind:
                    ast::OperationKind::Subscription,
                span: span_from_pos(sd.position),
                syntax: None,
            },
        );
    }

    ast::SchemaDefinition {
        description: None,
        directives: gp_directives_to_ast(
            &sd.directives,
        ),
        root_operations: root_ops,
        span: span_from_pos(sd.position),
        syntax: None,
    }
}

fn gp_type_def_to_ast(
    td: &graphql_parser::schema::TypeDefinition<
        'static,
        String,
    >,
) -> ast::TypeDefinition<'static> {
    use graphql_parser::schema::TypeDefinition as GpTd;
    match td {
        GpTd::Scalar(s) => {
            ast::TypeDefinition::Scalar(
                ast::ScalarTypeDefinition {
                    description: gp_description_to_ast(
                        &s.description,
                    ),
                    directives: gp_directives_to_ast(
                        &s.directives,
                    ),
                    name: string_to_name_at(
                        &s.name,
                        s.position,
                    ),
                    span: span_from_pos(s.position),
                    syntax: None,
                },
            )
        },
        GpTd::Object(obj) => {
            ast::TypeDefinition::Object(
                ast::ObjectTypeDefinition {
                    description: gp_description_to_ast(
                        &obj.description,
                    ),
                    directives: gp_directives_to_ast(
                        &obj.directives,
                    ),
                    fields: obj
                        .fields
                        .iter()
                        .map(gp_field_def_to_ast)
                        .collect(),
                    implements: obj
                        .implements_interfaces
                        .iter()
                        .map(|n| string_to_name(n))
                        .collect(),
                    name: string_to_name_at(
                        &obj.name,
                        obj.position,
                    ),
                    span: span_from_pos(obj.position),
                    syntax: None,
                },
            )
        },
        GpTd::Interface(iface) => {
            ast::TypeDefinition::Interface(
                ast::InterfaceTypeDefinition {
                    description: gp_description_to_ast(
                        &iface.description,
                    ),
                    directives: gp_directives_to_ast(
                        &iface.directives,
                    ),
                    fields: iface
                        .fields
                        .iter()
                        .map(gp_field_def_to_ast)
                        .collect(),
                    implements: iface
                        .implements_interfaces
                        .iter()
                        .map(|n| string_to_name(n))
                        .collect(),
                    name: string_to_name_at(
                        &iface.name,
                        iface.position,
                    ),
                    span: span_from_pos(
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
                    ),
                    directives: gp_directives_to_ast(
                        &u.directives,
                    ),
                    members: u
                        .types
                        .iter()
                        .map(|n| string_to_name(n))
                        .collect(),
                    name: string_to_name_at(
                        &u.name,
                        u.position,
                    ),
                    span: span_from_pos(u.position),
                    syntax: None,
                },
            )
        },
        GpTd::Enum(e) => {
            ast::TypeDefinition::Enum(
                ast::EnumTypeDefinition {
                    description: gp_description_to_ast(
                        &e.description,
                    ),
                    directives: gp_directives_to_ast(
                        &e.directives,
                    ),
                    name: string_to_name_at(
                        &e.name,
                        e.position,
                    ),
                    span: span_from_pos(e.position),
                    syntax: None,
                    values: e
                        .values
                        .iter()
                        .map(gp_enum_value_to_ast)
                        .collect(),
                },
            )
        },
        GpTd::InputObject(io) => {
            ast::TypeDefinition::InputObject(
                ast::InputObjectTypeDefinition {
                    description: gp_description_to_ast(
                        &io.description,
                    ),
                    directives: gp_directives_to_ast(
                        &io.directives,
                    ),
                    fields: io
                        .fields
                        .iter()
                        .map(gp_input_value_to_ast)
                        .collect(),
                    name: string_to_name_at(
                        &io.name,
                        io.position,
                    ),
                    span: span_from_pos(io.position),
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
) -> ast::TypeExtension<'static> {
    use graphql_parser::schema::TypeExtension as GpTe;
    match te {
        GpTe::Scalar(s) => {
            ast::TypeExtension::Scalar(
                ast::ScalarTypeExtension {
                    directives: gp_directives_to_ast(
                        &s.directives,
                    ),
                    name: string_to_name_at(
                        &s.name,
                        s.position,
                    ),
                    span: span_from_pos(s.position),
                    syntax: None,
                },
            )
        },
        GpTe::Object(obj) => {
            ast::TypeExtension::Object(
                ast::ObjectTypeExtension {
                    directives: gp_directives_to_ast(
                        &obj.directives,
                    ),
                    fields: obj
                        .fields
                        .iter()
                        .map(gp_field_def_to_ast)
                        .collect(),
                    implements: obj
                        .implements_interfaces
                        .iter()
                        .map(|n| string_to_name(n))
                        .collect(),
                    name: string_to_name_at(
                        &obj.name,
                        obj.position,
                    ),
                    span: span_from_pos(obj.position),
                    syntax: None,
                },
            )
        },
        GpTe::Interface(iface) => {
            ast::TypeExtension::Interface(
                ast::InterfaceTypeExtension {
                    directives: gp_directives_to_ast(
                        &iface.directives,
                    ),
                    fields: iface
                        .fields
                        .iter()
                        .map(gp_field_def_to_ast)
                        .collect(),
                    implements: iface
                        .implements_interfaces
                        .iter()
                        .map(|n| string_to_name(n))
                        .collect(),
                    name: string_to_name_at(
                        &iface.name,
                        iface.position,
                    ),
                    span: span_from_pos(
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
                    ),
                    members: u
                        .types
                        .iter()
                        .map(|n| string_to_name(n))
                        .collect(),
                    name: string_to_name_at(
                        &u.name,
                        u.position,
                    ),
                    span: span_from_pos(u.position),
                    syntax: None,
                },
            )
        },
        GpTe::Enum(e) => {
            ast::TypeExtension::Enum(
                ast::EnumTypeExtension {
                    directives: gp_directives_to_ast(
                        &e.directives,
                    ),
                    name: string_to_name_at(
                        &e.name,
                        e.position,
                    ),
                    span: span_from_pos(e.position),
                    syntax: None,
                    values: e
                        .values
                        .iter()
                        .map(gp_enum_value_to_ast)
                        .collect(),
                },
            )
        },
        GpTe::InputObject(io) => {
            ast::TypeExtension::InputObject(
                ast::InputObjectTypeExtension {
                    directives: gp_directives_to_ast(
                        &io.directives,
                    ),
                    fields: io
                        .fields
                        .iter()
                        .map(gp_input_value_to_ast)
                        .collect(),
                    name: string_to_name_at(
                        &io.name,
                        io.position,
                    ),
                    span: span_from_pos(io.position),
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
) -> ast::DirectiveDefinition<'static> {
    ast::DirectiveDefinition {
        arguments: dd
            .arguments
            .iter()
            .map(gp_input_value_to_ast)
            .collect(),
        description: gp_description_to_ast(
            &dd.description,
        ),
        locations: dd
            .locations
            .iter()
            .map(|loc| ast::DirectiveLocation {
                kind: gp_directive_location_to_ast(loc),
                span: span_from_pos(dd.position),
                syntax: None,
            })
            .collect(),
        name: string_to_name_at(
            &dd.name,
            dd.position,
        ),
        repeatable: dd.repeatable,
        span: span_from_pos(dd.position),
        syntax: None,
    }
}

// ───────────────────────────────────────────────────
// from_graphql_parser_query_ast
// ───────────────────────────────────────────────────

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
