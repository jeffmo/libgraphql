use crate::ast;
use crate::loc;
use crate::named_ref::DerefByName;
use crate::operation::NamedVariableRef;
use crate::operation::Variable;
use crate::types::EnumValue;
use crate::types::NamedEnumValueRef;
use indexmap::IndexMap;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Value {
    Bool(bool),
    EnumValue(NamedEnumValueRef),
    Float(f64),
    Int(i32),
    List(Vec<Value>),
    Null,
    Object(IndexMap<String, Value>),
    String(String),
    VarRef(NamedVariableRef),
}
impl Value {
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(str) = self {
            Some(str.as_str())
        } else {
            None
        }
    }

    pub(crate) fn from_ast(
        ast_value: &ast::Value<'_>,
        position: &loc::SourceLocation,
    ) -> Self {
        match ast_value {
            ast::Value::Variable(var_ref) =>
                Value::VarRef(
                    Variable::named_ref(
                        var_ref.name.value.as_ref(),
                        position.to_owned(),
                    ),
                ),

            ast::Value::Int(int_val) =>
                Value::Int(int_val.value),

            ast::Value::Float(float_val) =>
                Value::Float(float_val.value),

            ast::Value::String(string_val) =>
                Value::String(string_val.value.to_string()),

            ast::Value::Boolean(bool_val) =>
                Value::Bool(bool_val.value),

            ast::Value::Null(_) =>
                Value::Null,

            ast::Value::Enum(enum_val) =>
                Value::EnumValue(
                    EnumValue::named_ref(
                        enum_val.value.as_ref(),
                        position.to_owned(),
                    ),
                ),

            ast::Value::List(list_val) =>
                Value::List(list_val.values.iter().map(|ast_value|
                    Value::from_ast(ast_value, position)
                ).collect()),

            ast::Value::Object(obj_val) =>
                Value::Object(obj_val.fields.iter().map(|field| (
                    field.name.value.to_string(),
                    Value::from_ast(&field.value, position),
                )).collect()),
        }
    }
}
