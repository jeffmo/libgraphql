use crate::ast;
use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::schema::Schema;
use std::collections::HashMap;
use std::path::Path;

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
        schema.directives.get(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string())
        )
    }
}


/// Represents a Directive annotation. Essentially a wrapper around
/// NamedGraphQLDirectiveRef, but includes an argument list.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLDirectiveAnnotation {
    pub args: HashMap<String, ast::Value>,
    pub directive_ref: NamedDirectiveRef,
}
impl GraphQLDirectiveAnnotation {
    pub fn from_ast<P: AsRef<Path>>(
        file_path: P,
        ast_annots: &[ast::operation::Directive],
    ) -> Vec<Self> {
        ast_annots.iter().map(|d| {
            GraphQLDirectiveAnnotation {
                args: d.arguments.clone().into_iter().collect(),
                directive_ref: NamedDirectiveRef::new(
                    &d.name,
                    loc::FilePosition::from_pos(
                        file_path.as_ref(),
                        d.position,
                    ),
                ),
            }
        }).collect()
    }
}

/// Information associated with [GraphQLType::Enum]
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLEnumType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<GraphQLDirectiveAnnotation>,
    pub name: String,
    pub variants: HashMap<String, GraphQLEnumVariant>,
}

/// Represents a defined field on a [GraphQLObjectType] or
/// [GraphQLInterfaceType].
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLFieldDef {
    pub def_location: loc::SchemaDefLocation,
    pub type_ref: GraphQLTypeRef,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputFieldDef {
    pub def_location: loc::SchemaDefLocation,
    // TODO: There's more to input fields...
}

/// Represents a defined variant for some [GraphQLType::Enum].
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLEnumVariant {
    pub def_location: loc::FilePosition,
    pub directives: Vec<GraphQLDirectiveAnnotation>,
    pub name: String,
}
impl DerefByName for GraphQLEnumVariant {
    type Source = GraphQLEnumType;

    fn deref_name<'a>(
        enum_type: &'a Self::Source,
        variant_name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        enum_type.variants.get(variant_name).ok_or_else(
            || DerefByNameError::DanglingReference(variant_name.to_string())
        )
    }
}

/// Information associated with [GraphQLType::InputObject]
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLInputObjectType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<GraphQLDirectiveAnnotation>,
    pub fields: HashMap<String, InputFieldDef>,
    pub name: String,
}

/// Information associated with [GraphQLType::Interface]
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLInterfaceType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<GraphQLDirectiveAnnotation>,
    pub fields: HashMap<String, GraphQLFieldDef>,
    pub name: String,
}

/// Information associated with [GraphQLType::Object]
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLObjectType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<GraphQLDirectiveAnnotation>,
    pub fields: HashMap<String, GraphQLFieldDef>,
    pub name: String,
}

/// Information associated with [GraphQLType::Scalar]
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLScalarType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<GraphQLDirectiveAnnotation>,
    pub name: String,
}

/// Represents a defined GraphQL type
#[derive(Clone, Debug, PartialEq)]
pub enum GraphQLType {
    Bool,
    Enum(GraphQLEnumType),
    Float,
    ID,
    InputObject(GraphQLInputObjectType),
    Int,
    Interface(GraphQLInterfaceType),
    Object(GraphQLObjectType),
    Scalar(GraphQLScalarType),
    String,
    Union(GraphQLUnionType),
}
impl GraphQLType {
    pub fn get_def_location(&self) -> loc::SchemaDefLocation {
        match self {
            GraphQLType::Bool
                | GraphQLType::Float
                | GraphQLType::ID
                | GraphQLType::Int
                | GraphQLType::String =>
                loc::SchemaDefLocation::GraphQLBuiltIn,
            GraphQLType::Enum(t) =>
                loc::SchemaDefLocation::Schema(t.def_location.clone()),
            GraphQLType::InputObject(t) =>
                loc::SchemaDefLocation::Schema(t.def_location.clone()),
            GraphQLType::Interface(t) =>
                loc::SchemaDefLocation::Schema(t.def_location.clone()),
            GraphQLType::Object(t) =>
                loc::SchemaDefLocation::Schema(t.def_location.clone()),
            GraphQLType::Scalar(t) =>
                loc::SchemaDefLocation::Schema(t.def_location.clone()),
            GraphQLType::Union(t) =>
                loc::SchemaDefLocation::Schema(t.def_location.clone()),
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        match self {
            GraphQLType::Bool
                | GraphQLType::Float
                | GraphQLType::ID
                | GraphQLType::Int
                | GraphQLType::String => None,
            GraphQLType::Enum(t) => Some(t.name.as_str()),
            GraphQLType::InputObject(t) => Some(t.name.as_str()),
            GraphQLType::Interface(t) => Some(t.name.as_str()),
            GraphQLType::Object(t) => Some(t.name.as_str()),
            GraphQLType::Scalar(t) => Some(t.name.as_str()),
            GraphQLType::Union(t) => Some(t.name.as_str()),
        }
    }

    pub fn unwrap_object(&self) -> &GraphQLObjectType {
        match self {
            GraphQLType::Object(obj_type) => obj_type,
            _ => panic!("Not a GraphQLType::Object: {:#?}", self),
        }
    }
}
impl DerefByName for GraphQLType {
    type Source = Schema;

    fn deref_name<'a>(
        schema: &'a Schema,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        schema.types.get(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string()),
        )
    }
}

/// Represents a reference to a type (e.g. a "type annotation").
///
/// The most common example of a [GraphQLTypeRef] is the type specification on
/// an Object field. These type specifications "reference" another defined type.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphQLTypeRef {
    List {
        inner_type_ref: Box<GraphQLTypeRef>,
        nullable: bool,
        ref_location: loc::FilePosition,
    },
    Named {
        nullable: bool,
        type_ref: NamedGraphQLTypeRef,
    }
}
impl GraphQLTypeRef {
    pub(crate) fn extract_inner_named_ref(&self) -> &NamedGraphQLTypeRef {
        match self {
            GraphQLTypeRef::List { inner_type_ref, .. }
                => inner_type_ref.extract_inner_named_ref(),
            GraphQLTypeRef::Named { type_ref, .. }
                => type_ref,
        }
    }

    pub(crate) fn from_ast_type(
        ref_location: &loc::FilePosition,
        ast_type: &ast::operation::Type,
    ) -> Self {
        Self::from_ast_type_impl(ref_location, ast_type, /* nullable = */ true)
    }

    fn from_ast_type_impl(
        ref_location: &loc::FilePosition,
        ast_type: &ast::operation::Type,
        nullable: bool,
    ) -> Self {
        match ast_type {
            ast::operation::Type::ListType(inner) =>
                Self::List {
                    inner_type_ref: Box::new(Self::from_ast_type_impl(
                        ref_location,
                        inner,
                        true,
                    )),
                    nullable,
                    ref_location: ref_location.clone(),
                },

            ast::operation::Type::NamedType(name) =>
                Self::Named {
                    nullable,
                    type_ref: NamedGraphQLTypeRef::new(
                        name,
                        ref_location.clone(),
                    ),
                },

            ast::operation::Type::NonNullType(inner) =>
                Self::from_ast_type_impl(ref_location, inner, false),
        }
    }

    pub fn extract_named_type_ref(&self) -> &NamedGraphQLTypeRef {
        match self {
            GraphQLTypeRef::List { inner_type_ref, .. } =>
                inner_type_ref.extract_named_type_ref(),

            GraphQLTypeRef::Named { type_ref, .. } =>
                type_ref,
        }
    }

    pub fn get_ref_location(&self) -> &loc::FilePosition {
        match self {
            GraphQLTypeRef::List { ref_location, .. } => ref_location,
            GraphQLTypeRef::Named { type_ref, .. } => type_ref.get_ref_location(),
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            GraphQLTypeRef::List { nullable, .. } => *nullable,
            GraphQLTypeRef::Named { nullable, .. } => *nullable,
        }
    }
}

/// Information associated with [GraphQLType::Union]
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLUnionType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<GraphQLDirectiveAnnotation>,
    pub name: String,
    pub members: HashMap<String, GraphQLTypeRef>,
}

pub type NamedDirectiveRef = NamedRef<Schema, Directive>;
pub type NamedGraphQLEnumVariantRef = NamedRef<GraphQLEnumType, GraphQLEnumVariant>;
pub type NamedGraphQLTypeRef = NamedRef<Schema, GraphQLType>;
