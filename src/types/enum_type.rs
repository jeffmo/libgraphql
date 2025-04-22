use crate::DirectiveAnnotation;
use crate::loc;
use crate::types::EnumValue;
use std::collections::BTreeMap;

/// Represents a
/// [enum type](https://spec.graphql.org/October2021/#sec-Enums) defined within
/// some [`Schema`](crate::Schema).
#[derive(Clone, Debug, PartialEq)]
pub struct EnumType {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) name: String,
    pub(super) values: BTreeMap<String, EnumValue>,
}

impl EnumType {
    /// The [loc::SchemaDefLocation] indicating where this [EnumType] was
    /// defined within the schema.
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    /// The list of [DirectiveAnnotation]s applied to this [EnumType].
    ///
    /// This list of [DirectiveAnnotation]s is guaranteed to be ordered the same
    /// as the order of annotations specified on the [EnumType] definition in
    /// the schema. Note that [DirectiveAnnotation]s added from a type extension
    /// will appear sequentially in the order they were applied on the type
    /// extension, but there is no guarantee about where in this list a given
    /// type extension's annotations are added.
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    /// The name of this [EnumType].
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// A map from ValueName -> [EnumValue] for all [EnumValue]s defined for
    /// this [EnumType].
    ///
    /// This returns a [BTreeMap] to guarantee that map entries retain the same
    /// ordering as the order of field definitions on the [EnumType] in the
    /// schema.
    pub fn values(&self) -> &BTreeMap<String, EnumValue> {
        &self.values
    }
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use crate::types::NamedGraphQLTypeRef;
    use super::*;
    use std::path::PathBuf;

    pub fn mk_enum(type_name: &str, value_names: &[&str]) -> EnumType {
        let mut type_ = EnumType {
            def_location: loc::FilePosition {
                col: 1,
                file: PathBuf::from("str://0"),
                line: 2,
            }.into(),
            directives: vec![],
            name: type_name.to_string(),
            values: BTreeMap::new(),
        };

        for value_name in value_names.iter() {
            type_.values.insert(
                value_name.to_string(),
                mk_enum_value(&type_, value_name),
            );
        }

        type_
    }

    pub fn mk_enum_value(type_: &EnumType, value_name: &str) -> EnumValue {
        let file_path = PathBuf::from("str://0");
        let def_location = loc::SchemaDefLocation::Schema(
            loc::FilePosition {
                col: 2,
                file: file_path.to_path_buf(),
                line: 2,
            }
        );
        EnumValue {
            def_location: def_location.to_owned(),
            directives: DirectiveAnnotation::from_ast(
                file_path.as_path(),
                &[],
            ),
            name: value_name.to_string(),
            type_ref: NamedGraphQLTypeRef::new(
                type_.name.to_owned(),
                def_location
            ),
        }
    }
}
