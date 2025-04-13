use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::schema::Schema;
use crate::types::GraphQLTypeRef;
use crate::types::NamedGraphQLTypeRef;
use crate::types::Parameter;
use crate::Value;
use std::collections::BTreeMap;

lazy_static::lazy_static! {
    static ref DEPRECATED_PARAMS: BTreeMap<String, Parameter> = {
        BTreeMap::from([
            ("reason".to_string(), Parameter {
                def_location: loc::SchemaDefLocation::GraphQLBuiltIn,
                default_value: Some(Value::String("No longer supported".to_string())),
                name: "reason".to_string(),
                type_ref: GraphQLTypeRef::Named {
                    nullable: true,
                    type_ref: NamedGraphQLTypeRef::new(
                        "deprecated",
                        loc::SchemaDefLocation::GraphQLBuiltIn,
                    ),
                },
            }),
        ])
    };

    static ref INCLUDE_PARAMS: BTreeMap<String, Parameter> = {
        BTreeMap::from([
            ("if".to_string(), Parameter {
                def_location: loc::SchemaDefLocation::GraphQLBuiltIn,
                default_value: None,
                name: "if".to_string(),
                type_ref: GraphQLTypeRef::Named {
                    nullable: false,
                    type_ref: NamedGraphQLTypeRef::new(
                        "Boolean",
                        loc::SchemaDefLocation::GraphQLBuiltIn,
                    ),
                },
            }),
        ])
    };

    static ref SKIP_PARAMS: BTreeMap<String, Parameter> = {
        BTreeMap::from([
            ("if".to_string(), Parameter {
                def_location: loc::SchemaDefLocation::GraphQLBuiltIn,
                default_value: None,
                name: "if".to_string(),
                type_ref: GraphQLTypeRef::Named {
                    nullable: false,
                    type_ref: NamedGraphQLTypeRef::new(
                        "Boolean",
                        loc::SchemaDefLocation::GraphQLBuiltIn,
                    ),
                },
            }),
        ])
    };

    static ref SPECIFIED_BY_PARAMS: BTreeMap<String, Parameter> = {
        BTreeMap::from([
            ("url".to_string(), Parameter {
                def_location: loc::SchemaDefLocation::GraphQLBuiltIn,
                default_value: None,
                name: "url".to_string(),
                type_ref: GraphQLTypeRef::Named {
                    nullable: false,
                    type_ref: NamedGraphQLTypeRef::new(
                        "String",
                        loc::SchemaDefLocation::GraphQLBuiltIn,
                    ),
                },
            }),
        ])
    };
}

/// Represents a defined directive.
#[derive(Clone, Debug, PartialEq)]
pub enum Directive {
    Custom {
        def_location: loc::FilePosition,
        name: String,
        params: BTreeMap<String, Parameter>,
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

    pub fn parameters(&self) -> &BTreeMap<String, Parameter> {
        match self {
            Directive::Custom { params, .. } => params,
            Directive::Deprecated => &DEPRECATED_PARAMS,
            Directive::Include => &INCLUDE_PARAMS,
            Directive::Skip => &SKIP_PARAMS,
            Directive::SpecifiedBy => &SPECIFIED_BY_PARAMS,
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
