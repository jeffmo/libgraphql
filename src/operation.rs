use crate::ast;
use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::types::EnumVariant;
use crate::types::GraphQLTypeRef;
use crate::types::NamedDirectiveRef;
use crate::types::NamedEnumVariantRef;
use crate::types::NamedGraphQLTypeRef;
use std::collections::btree_map::BTreeMap;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct NamedFragment {
    directives: Vec<DirectiveAnnotation>,
    name: String,
    selections: Vec<OperationSelection>,
}
impl DerefByName for NamedFragment {
    type Source=HashMap<String, NamedFragment>;

    fn deref_name<'a>(
        fragments: &'a HashMap<String, NamedFragment>,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        fragments.get(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string())
        )
    }
}

#[derive(Clone, Debug)]
pub struct Mutation {
    directives: Vec<DirectiveAnnotation>,
    name: String,
    selections: Vec<OperationSelection>,
    var_defs: HashMap<String, OperationVarDef>,
}

pub type NamedFragmentRef = NamedRef<HashMap<String, NamedFragment>, NamedFragment>;

#[derive(Clone, Debug)]
pub enum OperationArgValue {
    VarRef(NamedOperationVariableRef),
    Int(ast::Number),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    EnumVariant(NamedEnumVariantRef),
    List(Vec<OperationArgValue>),
    Object(BTreeMap<String, OperationArgValue>),
}
impl OperationArgValue {
    // TODO: Move this to a private function on OperationsBuilder
    pub(crate) fn from_ast_value(
        ast_value: &ast::Value,
        position: loc::FilePosition,
    ) -> Self {
        match ast_value {
            ast::Value::Variable(var_name) =>
                OperationArgValue::VarRef(
                    OperationVarDef::named_ref(var_name, position),
                ),

            ast::Value::Int(value) =>
                OperationArgValue::Int(value.clone()),

            ast::Value::Float(value) =>
                OperationArgValue::Float(value.clone()),

            ast::Value::String(value) =>
                OperationArgValue::String(value.clone()),

            ast::Value::Boolean(value) =>
                OperationArgValue::Bool(value.clone()),

            ast::Value::Null =>
                OperationArgValue::Null,

            ast::Value::Enum(value) =>
                OperationArgValue::EnumVariant(
                    EnumVariant::named_ref(value, position),
                ),

            ast::Value::List(values) =>
                OperationArgValue::List(values.iter().map(|ast_value|
                    OperationArgValue::from_ast_value(ast_value, position.clone())
                ).collect()),

            ast::Value::Object(entries) =>
                OperationArgValue::Object(entries.iter().map(|(key, ast_value)|
                    (key.clone(), OperationArgValue::from_ast_value(ast_value, position.clone()))
                ).collect()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DirectiveAnnotation {
    pub arguments: HashMap<String, OperationArgValue>,
    pub directive_ref: NamedDirectiveRef,
    pub position: loc::FilePosition,
}

#[derive(Clone, Debug)]
pub enum OperationSelection {
    Field {
        alias: Option<String>,
        arguments: HashMap<String, OperationArgValue>,
        directives: Vec<DirectiveAnnotation>,
        name: String,
        position: loc::FilePosition,
        selections: Vec<OperationSelection>,
    },

    InlineFragmentSpread {
        directives: Vec<DirectiveAnnotation>,
        position: loc::FilePosition,
        selections: Vec<OperationSelection>,
        type_condition: Option<NamedGraphQLTypeRef>,
    },

    NamedFragmentSpread {
        directives: Vec<DirectiveAnnotation>,
        fragment: NamedFragmentRef,
        position: loc::FilePosition,
    },
}

#[derive(Clone, Debug)]
pub enum Operation {
    NamedFragment(NamedFragment),
    Mutation(Mutation),
    Query(Query),
}

#[derive(Clone, Debug)]
pub struct OperationVarDef {
    pub(crate) def_location: loc::FilePosition,
    pub(crate) default_value: Option<ast::Value>,
    pub(crate) name: String,
    pub(crate) type_: GraphQLTypeRef,
}
impl DerefByName for OperationVarDef {
    type Source = OperationVarDefMap;

    fn deref_name<'a>(
        vardef_map: &'a Self::Source,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        vardef_map.get(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string()),
        )
    }
}

pub type OperationVarDefMap = HashMap<String, OperationVarDef>;
pub type NamedOperationVariableRef = NamedRef<OperationVarDefMap, OperationVarDef>;

#[derive(Clone, Debug)]
pub struct Query {
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: Option<String>,
    pub(crate) selections: Vec<OperationSelection>,
    pub(crate) def_location: Option<loc::FilePosition>,
    pub(crate) var_defs: HashMap<String, OperationVarDef>,
}
