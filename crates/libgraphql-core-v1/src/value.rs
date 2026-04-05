use crate::names::EnumValueName;
use crate::names::VariableName;
use indexmap::IndexMap;

/// A GraphQL input value.
///
/// Represents all possible value literals as defined in the
/// [Input Values](https://spec.graphql.org/September2025/#sec-Input-Values)
/// section of the spec. Used for argument values, default values,
/// and variable values.
///
/// Variable references use [`VariableName`] and enum values use
/// [`EnumValueName`] — preventing accidental mixing with other
/// name domains.
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Value {
    Boolean(bool),
    Enum(EnumValueName),
    Float(f64),
    Int(i64),
    List(Vec<Value>),
    Null,
    Object(IndexMap<String, Value>),
    String(String),
    VarRef(VariableName),
}
