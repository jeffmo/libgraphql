use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::schema::Schema;
use crate::types::NamedGraphQLTypeRef;
use crate::types::NamedTypeAnnotation;
use crate::types::Parameter;
use crate::Value;
use indexmap::IndexMap;

lazy_static::lazy_static! {
    static ref DEPRECATED_PARAMS: IndexMap<String, Parameter> = {
        IndexMap::from([
            ("reason".to_string(), Parameter {
                def_location: loc::SchemaDefLocation::GraphQLBuiltIn,
                default_value: Some(Value::String("No longer supported".to_string())),
                name: "reason".to_string(),
                type_annotation: NamedTypeAnnotation {
                    nullable: true,
                    type_ref: NamedGraphQLTypeRef::new(
                        "deprecated",
                        loc::SchemaDefLocation::GraphQLBuiltIn,
                    ),
                }.into(),
            }),
        ])
    };

    static ref INCLUDE_PARAMS: IndexMap<String, Parameter> = {
        IndexMap::from([
            ("if".to_string(), Parameter {
                def_location: loc::SchemaDefLocation::GraphQLBuiltIn,
                default_value: None,
                name: "if".to_string(),
                type_annotation: NamedTypeAnnotation {
                    nullable: false,
                    type_ref: NamedGraphQLTypeRef::new(
                        "Boolean",
                        loc::SchemaDefLocation::GraphQLBuiltIn,
                    ),
                }.into(),
            }),
        ])
    };

    static ref SKIP_PARAMS: IndexMap<String, Parameter> = {
        IndexMap::from([
            ("if".to_string(), Parameter {
                def_location: loc::SchemaDefLocation::GraphQLBuiltIn,
                default_value: None,
                name: "if".to_string(),
                type_annotation: NamedTypeAnnotation {
                    nullable: false,
                    type_ref: NamedGraphQLTypeRef::new(
                        "Boolean",
                        loc::SchemaDefLocation::GraphQLBuiltIn,
                    ),
                }.into(),
            }),
        ])
    };

    static ref SPECIFIED_BY_PARAMS: IndexMap<String, Parameter> = {
        IndexMap::from([
            ("url".to_string(), Parameter {
                def_location: loc::SchemaDefLocation::GraphQLBuiltIn,
                default_value: None,
                name: "url".to_string(),
                type_annotation: NamedTypeAnnotation {
                    nullable: false,
                    type_ref: NamedGraphQLTypeRef::new(
                        "String",
                        loc::SchemaDefLocation::GraphQLBuiltIn,
                    ),
                }.into(),
            }),
        ])
    };
}

/// Represents a defined directive.
#[derive(Clone, Debug, PartialEq)]
pub enum Directive {
    Custom {
        def_location: loc::FilePosition,
        description: Option<String>,
        name: String,
        params: IndexMap<String, Parameter>,
    },
    Deprecated,
    Include,
    Skip,
    SpecifiedBy,
}
impl Directive {
    /// The description of this [`Directive`] as defined in the schema
    /// (e.g. in a """-string immediately before the type definition).
    pub fn description(&self) -> Option<&str> {
        match self {
            Directive::Custom { description, .. } => description.as_deref(),
            Directive::Deprecated => None,
            Directive::Include => None,
            Directive::Skip => None,
            Directive::SpecifiedBy => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Directive::Custom { name, .. } => name.as_str(),
            Directive::Deprecated => "deprecated",
            Directive::Include => "include",
            Directive::Skip => "skip",
            Directive::SpecifiedBy => "specifiedBy",
        }
    }

    pub fn parameters(&self) -> &IndexMap<String, Parameter> {
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
