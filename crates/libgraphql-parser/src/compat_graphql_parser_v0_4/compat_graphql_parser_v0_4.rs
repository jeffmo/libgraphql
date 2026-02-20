//! Compatibility layer for converting between the
//! libgraphql AST (`crate::ast`) and `graphql_parser`
//! v0.4 types.
//!
//! See [Section 9.2 of the AST design plan](
//! ../../custom-ast-plan.md) for the full conversion
//! specification.

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
