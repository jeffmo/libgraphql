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

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::*;
    use std::path::PathBuf;

    pub fn mk_enum(name: &str, variant_names: &[&str]) -> EnumType {
        let mut variants = BTreeMap::new();
        for name in variant_names.iter() {
            variants.insert(name.to_string(), mk_enum_variant(name));
        }

        EnumType {
            def_location: loc::FilePosition {
                col: 1,
                file: PathBuf::from("str://0"),
                line: 2,
            },
            directives: vec![],
            name: name.to_string(),
            variants,
        }
    }

    pub fn mk_enum_variant(name: &str) -> EnumVariant {
        let file_path = PathBuf::from("str://0");
        EnumVariant {
            def_location: loc::FilePosition {
                col: 2,
                file: file_path.to_path_buf(),
                line: 2,
            },
            directives: DirectiveAnnotation::from_ast(
                file_path.as_path(),
                &[],
            ),
            name: name.to_string(),
        }
    }
}

pub type NamedEnumVariantRef = NamedRef<EnumType, EnumVariant>;
