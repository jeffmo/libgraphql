use std::sync::OnceLock;

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

type DirectiveParamsMap = IndexMap<String, Parameter>;

fn deprecated_directive_params() -> &'static DirectiveParamsMap {
    static PARAMS: OnceLock<DirectiveParamsMap> = OnceLock::new();
    PARAMS.get_or_init(|| {
        IndexMap::from([
            ("reason".to_string(), Parameter {
                def_location: loc::SourceLocation::GraphQLBuiltIn,
                default_value: Some(Value::String("No longer supported".to_string())),
                name: "reason".to_string(),
                type_annotation: NamedTypeAnnotation {
                    nullable: true,
                    type_ref: NamedGraphQLTypeRef::new(
                        "deprecated",
                        loc::SourceLocation::GraphQLBuiltIn,
                    ),
                }.into(),
            }),
        ])
    })
}

fn include_directive_params() -> &'static DirectiveParamsMap {
    static PARAMS: OnceLock<DirectiveParamsMap> = OnceLock::new();
    PARAMS.get_or_init(|| {
        IndexMap::from([
            ("if".to_string(), Parameter {
                def_location: loc::SourceLocation::GraphQLBuiltIn,
                default_value: None,
                name: "if".to_string(),
                type_annotation: NamedTypeAnnotation {
                    nullable: false,
                    type_ref: NamedGraphQLTypeRef::new(
                        "Boolean",
                        loc::SourceLocation::GraphQLBuiltIn,
                    ),
                }.into(),
            }),
        ])
    })
}

fn skip_directive_params() -> &'static DirectiveParamsMap {
    static PARAMS: OnceLock<DirectiveParamsMap> = OnceLock::new();
    PARAMS.get_or_init(|| {
        IndexMap::from([
            ("if".to_string(), Parameter {
                def_location: loc::SourceLocation::GraphQLBuiltIn,
                default_value: None,
                name: "if".to_string(),
                type_annotation: NamedTypeAnnotation {
                    nullable: false,
                    type_ref: NamedGraphQLTypeRef::new(
                        "Boolean",
                        loc::SourceLocation::GraphQLBuiltIn,
                    ),
                }.into(),
            }),
        ])
    })
}

fn specified_by_directive_params() -> &'static DirectiveParamsMap {
    static PARAMS: OnceLock<DirectiveParamsMap> = OnceLock::new();
    PARAMS.get_or_init(|| {
        IndexMap::from([
            ("url".to_string(), Parameter {
                def_location: loc::SourceLocation::GraphQLBuiltIn,
                default_value: None,
                name: "url".to_string(),
                type_annotation: NamedTypeAnnotation {
                    nullable: false,
                    type_ref: NamedGraphQLTypeRef::new(
                        "String",
                        loc::SourceLocation::GraphQLBuiltIn,
                    ),
                }.into(),
            }),
        ])
    })
}

/// Represents a defined directive.
#[derive(Clone, Debug, PartialEq)]
pub enum Directive {
    Custom {
        def_location: loc::SourceLocation,
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
    pub fn def_location(&self) -> &loc::SourceLocation {
        match self {
            Self::Custom { def_location, .. } => def_location,
            Self::Deprecated
                | Self::Include
                | Self::Skip
                | Self::SpecifiedBy =>
                &loc::SourceLocation::GraphQLBuiltIn
        }
    }

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

    pub fn is_builtin(&self) -> bool {
        matches!(self.def_location(), loc::SourceLocation::GraphQLBuiltIn)
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
            Directive::Deprecated => deprecated_directive_params(),
            Directive::Include => include_directive_params(),
            Directive::Skip => skip_directive_params(),
            Directive::SpecifiedBy => specified_by_directive_params(),
        }
    }
}
impl DerefByName for Directive {
    type Source = Schema;
    type RefLocation = loc::SourceLocation;

    fn deref_name<'a>(
        schema: &'a Schema,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        schema.directive_defs.get(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string())
        )
    }
}

pub type NamedDirectiveRef = NamedRef<Schema, loc::SourceLocation, Directive>;
