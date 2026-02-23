//! Forward schema conversion: libgraphql AST →
//! `graphql_parser` v0.4 schema `Document`.

use crate::ast;
use crate::compat_graphql_parser_v0_4::helpers::description_to_gp;
use crate::compat_graphql_parser_v0_4::helpers::directive_location_to_gp;
use crate::compat_graphql_parser_v0_4::helpers::directives_to_gp;
use crate::compat_graphql_parser_v0_4::helpers::enum_value_def_to_gp;
use crate::compat_graphql_parser_v0_4::helpers::field_def_to_gp;
use crate::compat_graphql_parser_v0_4::helpers::input_value_def_to_gp;
use crate::compat_graphql_parser_v0_4::helpers::pos_from_span;
use crate::GraphQLParseError;
use crate::GraphQLParseErrorKind;
use crate::ParseResult;

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
