use std::borrow::Cow;

use crate::ast::ast_node::append_span_source_slice;
use crate::ast::AstNode;
use crate::ast::DelimiterPair;
use crate::ast::Name;
use crate::token::GraphQLToken;
use crate::GraphQLSourceSpan;
use inherent::inherent;

// =========================================================
// Value enum
// =========================================================

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

// =========================================================
// Scalar value types
// =========================================================

/// A GraphQL integer value.
///
/// Per the
/// [Int Value](https://spec.graphql.org/September2025/#sec-Int-Value)
/// section of the spec, Int is a signed 32-bit integer. On
/// overflow/underflow the parser emits a diagnostic and clamps
/// to `i32::MAX` / `i32::MIN`.
#[derive(Clone, Debug, PartialEq)]
pub struct IntValue<'src> {
    /// The parsed 32-bit integer value. On overflow/underflow
    /// the parser emits a diagnostic and clamps to
    /// `i32::MAX` / `i32::MIN`.
    pub value: i32,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<IntValueSyntax<'src>>,
}

impl IntValue<'_> {
    /// Widen to `i64` (infallible).
    pub fn as_i64(&self) -> i64 {
        self.value as i64
    }
}

/// A GraphQL float value.
///
/// Per the
/// [Float Value](https://spec.graphql.org/September2025/#sec-Float-Value)
/// section of the spec, Float is a double-precision
/// floating-point value (IEEE 754). On overflow the parser
/// emits a diagnostic and stores
/// `f64::INFINITY` / `f64::NEG_INFINITY`.
#[derive(Clone, Debug, PartialEq)]
pub struct FloatValue<'src> {
    /// The parsed `f64` value. On overflow the parser emits a
    /// diagnostic and stores
    /// `f64::INFINITY` / `f64::NEG_INFINITY`.
    pub value: f64,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<FloatValueSyntax<'src>>,
}

/// A GraphQL string value.
///
/// Per the
/// [String Value](https://spec.graphql.org/September2025/#sec-String-Value)
/// section of the spec, string values may be quoted strings
/// or block strings. This struct contains the processed
/// string after escape-sequence resolution and block-string
/// indentation stripping. Borrows from source when no
/// transformation was needed; owned when escapes were resolved
/// or block-string stripping produced a non-contiguous result.
#[derive(Clone, Debug, PartialEq)]
pub struct StringValue<'src> {
    /// The processed string value after escape-sequence
    /// resolution and block-string indentation stripping.
    pub value: Cow<'src, str>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<StringValueSyntax<'src>>,
}

/// A GraphQL boolean value (`true` or `false`).
///
/// See the
/// [Boolean Value](https://spec.graphql.org/September2025/#sec-Boolean-Value)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct BooleanValue<'src> {
    pub value: bool,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<BooleanValueSyntax<'src>>,
}

/// A GraphQL null literal.
///
/// See the
/// [Null Value](https://spec.graphql.org/September2025/#sec-Null-Value)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct NullValue<'src> {
    pub span: GraphQLSourceSpan,
    pub syntax: Option<NullValueSyntax<'src>>,
}

/// A GraphQL enum value (an unquoted name that is not
/// `true`, `false`, or `null`).
///
/// See the
/// [Enum Value](https://spec.graphql.org/September2025/#sec-Enum-Value)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumValue<'src> {
    pub value: Cow<'src, str>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<EnumValueSyntax<'src>>,
}

// =========================================================
// Variable value
// =========================================================

/// A variable reference in a GraphQL value position
/// (e.g., `$id`).
///
/// See the
/// [Variables](https://spec.graphql.org/September2025/#sec-Language.Variables)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableValue<'src> {
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<VariableValueSyntax<'src>>,
}

// =========================================================
// Composite value types
// =========================================================

/// A GraphQL list value (e.g., `[1, 2, 3]`).
///
/// See the
/// [List Value](https://spec.graphql.org/September2025/#sec-List-Value)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ListValue<'src> {
    pub values: Vec<Value<'src>>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<ListValueSyntax<'src>>,
}

/// A GraphQL input object value (e.g., `{x: 1, y: 2}`).
///
/// See the
/// [Input Object Values](https://spec.graphql.org/September2025/#sec-Input-Object-Values)
/// section of the spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectValue<'src> {
    pub fields: Vec<ObjectField<'src>>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<ObjectValueSyntax<'src>>,
}

/// A single field within a GraphQL
/// [input object value](https://spec.graphql.org/September2025/#sec-Input-Object-Values).
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectField<'src> {
    pub name: Name<'src>,
    pub value: Value<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<ObjectFieldSyntax<'src>>,
}

// =========================================================
// Value syntax structs
// =========================================================

/// Syntax detail for an [`IntValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct IntValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

/// Syntax detail for a [`FloatValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct FloatValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

/// Syntax detail for a [`StringValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct StringValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

/// Syntax detail for a [`BooleanValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct BooleanValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

/// Syntax detail for a [`NullValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct NullValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

/// Syntax detail for an [`EnumValue`] (the enum value
/// literal, not the enum value definition).
#[derive(Clone, Debug, PartialEq)]
pub struct EnumValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

/// Syntax detail for a [`VariableValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct VariableValueSyntax<'src> {
    pub dollar: GraphQLToken<'src>,
}

/// Syntax detail for a [`ListValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct ListValueSyntax<'src> {
    pub brackets: DelimiterPair<'src>,
}

/// Syntax detail for an [`ObjectValue`].
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectValueSyntax<'src> {
    pub braces: DelimiterPair<'src>,
}

/// Syntax detail for an [`ObjectField`].
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectFieldSyntax<'src> {
    pub colon: GraphQLToken<'src>,
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

#[inherent]
impl AstNode for IntValue<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for FloatValue<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for StringValue<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for BooleanValue<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for NullValue<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for EnumValue<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for VariableValue<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for ListValue<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for ObjectValue<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}

#[inherent]
impl AstNode for ObjectField<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        if let Some(src) = source {
            append_span_source_slice(
                &self.span, sink, src,
            );
        }
    }
}
