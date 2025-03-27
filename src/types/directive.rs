use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::schema::Schema;

/// Represents a defined directive.
#[derive(Clone, Debug, PartialEq)]
pub enum Directive {
    Custom {
        def_location: loc::FilePosition,
        name: String,
        // TODO: parameters
    },
    Deprecated,
    Include,
    Skip,
    SpecifiedBy,
}
impl Directive {
    pub fn name(&self) -> &str {
        match self {
            Directive::Custom { name, .. } => name.as_str(),
            Directive::Deprecated => "deprecated",
            Directive::Include => "include",
            Directive::Skip => "skip",
            Directive::SpecifiedBy => "specifiedBy",
        }
    }
}
impl DerefByName for Directive {
    type Source=Schema;

    fn deref_name<'a>(
        schema: &'a Schema,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        schema.directive_defs.get(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string())
        )
    }
}

pub type NamedDirectiveRef = NamedRef<Schema, Directive>;
