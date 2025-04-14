use crate::ast;
use crate::loc;
use crate::Schema;
use crate::SchemaBuildError;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::types::DirectiveAnnotation;
use std::collections::BTreeMap;
use std::path::Path;

type Result<T> = std::result::Result<T, SchemaBuildError>;

/// Information associated with [GraphQLType::Enum]
#[derive(Clone, Debug, PartialEq)]
pub struct EnumType<'schema> {
    pub def_location: loc::FilePosition,
    pub directives: Vec<DirectiveAnnotation<'schema>>,
    pub name: String,
    pub variants: BTreeMap<String, EnumVariant<'schema>>,
}

/// Represents a defined variant for some [GraphQLType::Enum].
#[derive(Clone, Debug, PartialEq)]
pub struct EnumVariant<'schema> {
    pub def_location: loc::FilePosition,
    pub directives: Vec<DirectiveAnnotation<'schema>>,
    pub name: String,
}
impl<'schema> DerefByName for EnumVariant<'schema> {
    type Source = EnumType<'schema>;

    fn deref_name<'a>(
        enum_type: &'a Self::Source,
        variant_name: &str,
    ) -> std::result::Result<&'a EnumVariant<'schema>, DerefByNameError> {
        enum_type.variants.get(variant_name).ok_or_else(
            || DerefByNameError::DanglingReference(variant_name.to_string())
        )
    }
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::*;
    use std::path::PathBuf;

    pub fn mk_enum<'schema>(
        schema: &'schema Schema,
        name: &str,
        variant_names: &[&str],
    ) -> EnumType<'schema> {
        let mut variants = BTreeMap::new();
        for name in variant_names.iter() {
            variants.insert(name.to_string(), mk_enum_variant(schema, name));
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

    pub fn mk_enum_variant<'schema>(schema: &'schema Schema, name: &str) -> EnumVariant<'schema> {
        let file_path = PathBuf::from("str://0");
        EnumVariant {
            def_location: loc::FilePosition {
                col: 2,
                file: file_path.to_path_buf(),
                line: 2,
            },
            directives: DirectiveAnnotation::from_ast(
                schema,
                file_path.as_path(),
                &[],
            ),
            name: name.to_string(),
        }
    }
}

pub type NamedEnumVariantRef<'schema> = NamedRef<EnumType<'schema>, EnumVariant<'schema>>;
