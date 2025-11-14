use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::types::TypeAnnotation;
use crate::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub(crate) def_location: loc::SourceLocation,
    pub(crate) default_value: Option<Value>,
    pub(crate) name: String,
    pub(crate) type_annotation: TypeAnnotation,
}
impl Variable {
    pub fn def_location(&self) -> &loc::SourceLocation {
        &self.def_location
    }

    pub fn default_value(&self) -> Option<&Value> {
        self.default_value.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
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
