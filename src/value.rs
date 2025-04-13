use crate::ast;
use crate::named_ref::DerefByName;
use crate::loc;
use crate::operation::NamedVariableRef;
use crate::operation::Variable;
use crate::types::EnumVariant;
use crate::types::NamedEnumVariantRef;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    VarRef(NamedVariableRef),
    Int(ast::Number),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    EnumVariant(NamedEnumVariantRef),
    List(Vec<Value>),
    Object(BTreeMap<String, Value>),
}
impl Value {
    // TODO: Move this to a private function on OperationsBuilder
    pub(crate) fn from_ast(
        ast_value: &ast::Value,
        position: loc::FilePosition,
    ) -> Self {
        match ast_value {
            ast::Value::Variable(var_name) =>
                Value::VarRef(
                    Variable::named_ref(var_name, position.into()),
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
                Value::EnumVariant(
                    EnumVariant::named_ref(value, position.into()),
                ),

            ast::Value::List(values) =>
                Value::List(values.iter().map(|ast_value|
                    Value::from_ast(ast_value, position.clone())
                ).collect()),

            ast::Value::Object(entries) =>
                Value::Object(entries.iter().map(|(key, ast_value)|
                    (key.clone(), Value::from_ast(ast_value, position.clone()))
                ).collect()),
        }
    }
}

