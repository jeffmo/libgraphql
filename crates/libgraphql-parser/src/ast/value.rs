use crate::ast::AstNode;
use crate::ast::BooleanValue;
use crate::ast::EnumValue;
use crate::ast::FloatValue;
use crate::ast::IntValue;
use crate::ast::ListValue;
use crate::ast::NullValue;
use crate::ast::ObjectValue;
use crate::ast::StringValue;
use crate::ast::VariableValue;
use inherent::inherent;

/// A GraphQL input value.
///
/// Represents all possible GraphQL value literals as defined
/// in the
/// [Input Values](https://spec.graphql.org/September2025/#sec-Input-Values)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub enum Value<'src> {
    Boolean(BooleanValue<'src>),
    Enum(EnumValue<'src>),
    Float(FloatValue<'src>),
    Int(IntValue<'src>),
    List(ListValue<'src>),
    Null(NullValue<'src>),
    Object(ObjectValue<'src>),
    String(StringValue<'src>),
    Variable(VariableValue<'src>),
}

#[inherent]
impl AstNode for Value<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        match self {
            Value::Boolean(v) => {
                v.append_source(sink, source)
            },
            Value::Enum(v) => {
                v.append_source(sink, source)
            },
            Value::Float(v) => {
                v.append_source(sink, source)
            },
            Value::Int(v) => {
                v.append_source(sink, source)
            },
            Value::List(v) => {
                v.append_source(sink, source)
            },
            Value::Null(v) => {
                v.append_source(sink, source)
            },
            Value::Object(v) => {
                v.append_source(sink, source)
            },
            Value::String(v) => {
                v.append_source(sink, source)
            },
            Value::Variable(v) => {
                v.append_source(sink, source)
            },
        }
    }
}
