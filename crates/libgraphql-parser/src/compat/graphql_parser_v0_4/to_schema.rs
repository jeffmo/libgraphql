//! Forward schema conversion: libgraphql AST →
//! `graphql_parser` v0.4 schema `Document`.

use crate::ast;
use crate::compat::graphql_parser_v0_4::helpers::description_to_gp;
use crate::compat::graphql_parser_v0_4::helpers::directive_location_to_gp;
use crate::compat::graphql_parser_v0_4::helpers::directives_to_gp;
use crate::compat::graphql_parser_v0_4::helpers::enum_value_def_to_gp;
use crate::compat::graphql_parser_v0_4::helpers::field_def_to_gp;
use crate::compat::graphql_parser_v0_4::helpers::input_value_def_to_gp;
use crate::compat::graphql_parser_v0_4::helpers::pos_from_span;
use crate::compat::graphql_parser_v0_4::helpers::type_ext_pos_from_span;
use crate::GraphQLParseError;
use crate::GraphQLParseErrorKind;
use crate::ParseResult;
use crate::SourceSpan;

fn schema_def_to_gp(
    sd: &ast::SchemaDefinition<'_>,
    source_map: &crate::SourceMap<'_>,
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
        position: pos_from_span(sd.span, source_map),
        directives: directives_to_gp(
            &sd.directives, source_map,
        ),
        query,
        mutation,
        subscription,
    })
}

fn type_def_to_gp(
    td: &ast::TypeDefinition<'_>,
    source_map: &crate::SourceMap<'_>,
) -> graphql_parser::schema::Definition<'static, String>
{
    use graphql_parser::schema::Definition as GpDef;
    use graphql_parser::schema::TypeDefinition as GpTd;
    GpDef::TypeDefinition(match td {
        ast::TypeDefinition::Enum(e) => {
            GpTd::Enum(graphql_parser::schema::EnumType {
                position: pos_from_span(
                    e.span, source_map,
                ),
                description: description_to_gp(
                    &e.description,
                ),
                name: e.name.value.to_string(),
                directives: directives_to_gp(
                    &e.directives, source_map,
                ),
                values: e
                    .values
                    .iter()
                    .map(|v| {
                        enum_value_def_to_gp(
                            v, source_map,
                        )
                    })
                    .collect(),
            })
        },
        ast::TypeDefinition::InputObject(io) => {
            GpTd::InputObject(
                graphql_parser::schema::InputObjectType {
                    position: pos_from_span(
                        io.span, source_map,
                    ),
                    description: description_to_gp(
                        &io.description,
                    ),
                    name: io.name.value.to_string(),
                    directives: directives_to_gp(
                        &io.directives, source_map,
                    ),
                    fields: io
                        .fields
                        .iter()
                        .map(|ivd| {
                            input_value_def_to_gp(
                                ivd, source_map,
                            )
                        })
                        .collect(),
                },
            )
        },
        ast::TypeDefinition::Interface(i) => {
            GpTd::Interface(
                graphql_parser::schema::InterfaceType {
                    position: pos_from_span(
                        i.span, source_map,
                    ),
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
                        &i.directives, source_map,
                    ),
                    fields: i
                        .fields
                        .iter()
                        .map(|fd| {
                            field_def_to_gp(
                                fd, source_map,
                            )
                        })
                        .collect(),
                },
            )
        },
        ast::TypeDefinition::Object(o) => {
            GpTd::Object(
                graphql_parser::schema::ObjectType {
                    position: pos_from_span(
                        o.span, source_map,
                    ),
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
                        &o.directives, source_map,
                    ),
                    fields: o
                        .fields
                        .iter()
                        .map(|fd| {
                            field_def_to_gp(
                                fd, source_map,
                            )
                        })
                        .collect(),
                },
            )
        },
        ast::TypeDefinition::Scalar(s) => {
            GpTd::Scalar(
                graphql_parser::schema::ScalarType {
                    position: pos_from_span(
                        s.span, source_map,
                    ),
                    description: description_to_gp(
                        &s.description,
                    ),
                    name: s.name.value.to_string(),
                    directives: directives_to_gp(
                        &s.directives, source_map,
                    ),
                },
            )
        },
        ast::TypeDefinition::Union(u) => {
            GpTd::Union(
                graphql_parser::schema::UnionType {
                    position: pos_from_span(
                        u.span, source_map,
                    ),
                    description: description_to_gp(
                        &u.description,
                    ),
                    name: u.name.value.to_string(),
                    directives: directives_to_gp(
                        &u.directives, source_map,
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
    source_map: &crate::SourceMap<'_>,
) -> graphql_parser::schema::Definition<'static, String>
{
    use graphql_parser::schema::Definition as GpDef;
    use graphql_parser::schema::TypeExtension as GpTe;
    GpDef::TypeExtension(match te {
        ast::TypeExtension::Enum(e) => GpTe::Enum(
            graphql_parser::schema::EnumTypeExtension {
                position: type_ext_pos_from_span(
                    e.span, source_map,
                ),
                name: e.name.value.to_string(),
                directives: directives_to_gp(
                    &e.directives, source_map,
                ),
                values: e
                    .values
                    .iter()
                    .map(|v| {
                        enum_value_def_to_gp(
                            v, source_map,
                        )
                    })
                    .collect(),
            },
        ),
        ast::TypeExtension::InputObject(io) => {
            GpTe::InputObject(
                graphql_parser::schema::InputObjectTypeExtension {
                    position: type_ext_pos_from_span(
                        io.span, source_map,
                    ),
                    name: io.name.value.to_string(),
                    directives: directives_to_gp(
                        &io.directives, source_map,
                    ),
                    fields: io
                        .fields
                        .iter()
                        .map(|ivd| {
                            input_value_def_to_gp(
                                ivd, source_map,
                            )
                        })
                        .collect(),
                },
            )
        },
        ast::TypeExtension::Interface(i) => {
            GpTe::Interface(
                graphql_parser::schema::InterfaceTypeExtension {
                    position: type_ext_pos_from_span(
                        i.span, source_map,
                    ),
                    name: i.name.value.to_string(),
                    implements_interfaces: i
                        .implements
                        .iter()
                        .map(|n| n.value.to_string())
                        .collect(),
                    directives: directives_to_gp(
                        &i.directives, source_map,
                    ),
                    fields: i
                        .fields
                        .iter()
                        .map(|fd| {
                            field_def_to_gp(
                                fd, source_map,
                            )
                        })
                        .collect(),
                },
            )
        },
        ast::TypeExtension::Object(o) => {
            GpTe::Object(
                graphql_parser::schema::ObjectTypeExtension {
                    position: type_ext_pos_from_span(
                        o.span, source_map,
                    ),
                    name: o.name.value.to_string(),
                    implements_interfaces: o
                        .implements
                        .iter()
                        .map(|n| n.value.to_string())
                        .collect(),
                    directives: directives_to_gp(
                        &o.directives, source_map,
                    ),
                    fields: o
                        .fields
                        .iter()
                        .map(|fd| {
                            field_def_to_gp(
                                fd, source_map,
                            )
                        })
                        .collect(),
                },
            )
        },
        ast::TypeExtension::Scalar(s) => GpTe::Scalar(
            graphql_parser::schema::ScalarTypeExtension {
                position: type_ext_pos_from_span(
                    s.span, source_map,
                ),
                name: s.name.value.to_string(),
                directives: directives_to_gp(
                    &s.directives, source_map,
                ),
            },
        ),
        ast::TypeExtension::Union(u) => GpTe::Union(
            graphql_parser::schema::UnionTypeExtension {
                position: type_ext_pos_from_span(
                    u.span, source_map,
                ),
                name: u.name.value.to_string(),
                directives: directives_to_gp(
                    &u.directives, source_map,
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
    source_map: &crate::SourceMap<'_>,
) -> graphql_parser::schema::Definition<'static, String>
{
    use graphql_parser::schema::Definition as GpDef;
    GpDef::DirectiveDefinition(
        graphql_parser::schema::DirectiveDefinition {
            position: pos_from_span(
                dd.span, source_map,
            ),
            description: description_to_gp(
                &dd.description,
            ),
            name: dd.name.value.to_string(),
            arguments: dd
                .arguments
                .iter()
                .map(|ivd| {
                    input_value_def_to_gp(
                        ivd, source_map,
                    )
                })
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
pub fn to_graphql_parser_schema_ast<'a>(
    doc: &ast::Document<'_>,
    source_map: &crate::SourceMap<'a>,
) -> ParseResult<
    'a,
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
                    directive_def_to_gp(
                        dd, source_map,
                    ),
                );
            },
            ast::Definition::SchemaDefinition(sd) => {
                definitions.push(
                    schema_def_to_gp(sd, source_map),
                );
            },
            ast::Definition::SchemaExtension(se) => {
                errors.push(GraphQLParseError::new(
                    "Schema extensions cannot be \
                     represented in graphql_parser \
                     v0.4 AST",
                    GraphQLParseErrorKind
                        ::UnsupportedFeature {
                        feature:
                            "schema extension"
                                .to_string(),
                    },
                    source_map.resolve_span(se.span)
                        .unwrap_or_else(SourceSpan::zero),
                ));
            },
            ast::Definition::TypeDefinition(td) => {
                definitions.push(
                    type_def_to_gp(td, source_map),
                );
            },
            ast::Definition::TypeExtension(te) => {
                definitions.push(
                    type_ext_to_gp(te, source_map),
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
        ParseResult::new_ok(gp_doc, source_map.clone())
    } else {
        ParseResult::new_recovered(gp_doc, errors, source_map.clone())
    }
}
