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
