use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::types::DirectiveAnnotation;
use std::collections::BTreeMap;

/// Information associated with [GraphQLType::Enum]
#[derive(Clone, Debug, PartialEq)]
pub struct EnumType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<DirectiveAnnotation>,
    pub name: String,
    pub variants: BTreeMap<String, EnumVariant>,
}

/// Represents a defined variant for some [GraphQLType::Enum].
#[derive(Clone, Debug, PartialEq)]
pub struct EnumVariant {
    pub def_location: loc::FilePosition,
    pub directives: Vec<DirectiveAnnotation>,
    pub name: String,
}
impl DerefByName for EnumVariant {
    type Source = EnumType;

    fn deref_name<'a>(
        enum_type: &'a Self::Source,
        variant_name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        enum_type.variants.get(variant_name).ok_or_else(
            || DerefByNameError::DanglingReference(variant_name.to_string())
        )
    }
}

pub type NamedEnumVariantRef = NamedRef<EnumType, EnumVariant>;
