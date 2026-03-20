use crate::ast::AstNode;
use crate::ast::BooleanValue;
use crate::ast::EnumValue;
use crate::ast::FloatValue;
use crate::ast::IntValue;
use crate::ast::ListValue;
use crate::ast::NullValue;
use crate::ast::ObjectValue;
use crate::ast::StringValue;
use crate::ast::VariableReference;
use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;
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
    Variable(VariableReference<'src>),
}

#[inherent]
impl AstNode for Value<'_> {
    /// See [`AstNode::append_source()`](crate::ast::AstNode::append_source).
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

    /// Returns this value's byte-offset span within the source
    /// text.
    ///
    /// The returned [`ByteSpan`] can be resolved to line/column
    /// positions via [`source_span()`](Self::source_span) or
    /// [`ByteSpan::resolve()`].
    pub fn byte_span(&self) -> ByteSpan {
        match self {
            Self::Boolean(v) => v.span,
            Self::Enum(v) => v.span,
            Self::Float(v) => v.span,
            Self::Int(v) => v.span,
            Self::List(v) => v.span,
            Self::Null(v) => v.span,
            Self::Object(v) => v.span,
            Self::String(v) => v.span,
            Self::Variable(v) => v.span,
        }
    }

    /// Resolves this value's position to line/column
    /// coordinates using the given [`SourceMap`].
    ///
    /// Returns [`None`] if the byte offsets cannot be resolved
    /// (e.g. the span was synthetically constructed without
    /// valid position data).
    pub fn source_span(
        &self,
        source_map: &SourceMap,
    ) -> Option<SourceSpan> {
        self.byte_span().resolve(source_map)
    }
}
