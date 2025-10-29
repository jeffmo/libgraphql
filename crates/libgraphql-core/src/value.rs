use crate::ast;
use crate::named_ref::DerefByName;
use crate::loc;
use crate::operation::NamedVariableRef;
use crate::operation::Variable;
use crate::types::EnumValue;
use crate::types::NamedEnumValueRef;
use indexmap::IndexMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    VarRef(NamedVariableRef),
    Int(ast::Number),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    EnumValue(NamedEnumValueRef),
    List(Vec<Value>),
    Object(IndexMap<String, Value>),
}
impl Value {
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(str) = self {
            Some(str.as_str())
        } else {
            None
        }
    }

    // TODO: Move this to a private function on OperationsBuilder
    pub(crate) fn from_ast(
        ast_value: &ast::Value,
        position: &loc::SourceLocation,
    ) -> Self {
        match ast_value {
            ast::Value::Variable(var_name) =>
                Value::VarRef(
                    Variable::named_ref(var_name, position.to_owned()),
                ),

            ast::Value::Int(value) =>
                Value::Int(value.clone()),

            ast::Value::Float(value) =>
                Value::Float(*value),

            ast::Value::String(value) =>
                Value::String(value.clone()),

            ast::Value::Boolean(value) =>
                Value::Bool(*value),

            ast::Value::Null =>
                Value::Null,

            ast::Value::Enum(value) =>
                Value::EnumValue(
                    EnumValue::named_ref(value, position.to_owned())
                ),

            ast::Value::List(values) =>
                Value::List(values.iter().map(|ast_value|
                    Value::from_ast(ast_value, position)
                ).collect()),

            ast::Value::Object(entries) =>
                Value::Object(entries.iter().map(|(key, ast_value)|
                    (key.clone(), Value::from_ast(ast_value, position))
                ).collect()),
        }
    }
}

