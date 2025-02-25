use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::types::GraphQLTypeRef;
use crate::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub(crate) def_location: loc::FilePosition,
    pub(crate) default_value: Option<Value>,
    pub(crate) name: String,
    pub(crate) type_: GraphQLTypeRef,
}
impl DerefByName for Variable {
    type Source = HashMap<String, Variable>;

    fn deref_name<'a>(
        vardef_map: &'a Self::Source,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        vardef_map.get(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string()),
        )
    }
}

pub type NamedVariableRef = NamedRef<HashMap<String, Variable>, Variable>;
