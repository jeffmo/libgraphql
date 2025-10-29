use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::types::TypeAnnotation;
use crate::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub def_location: loc::SourceLocation,
    pub default_value: Option<Value>,
    pub name: String,
    pub type_: TypeAnnotation,
}
impl Variable {
    pub fn def_location(&self) -> &loc::SourceLocation {
        &self.def_location
    }
}
impl DerefByName for Variable {
    type Source = HashMap<String, Variable>;
    type RefLocation = loc::SourceLocation;

    fn deref_name<'a>(
        vardef_map: &'a Self::Source,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        vardef_map.get(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string()),
        )
    }
}

pub type NamedVariableRef = NamedRef<
    /* TSource = */ HashMap<String, Variable>,
    /* TRefLocation = */ loc::SourceLocation,
    /* TResource = */ Variable,
>;
