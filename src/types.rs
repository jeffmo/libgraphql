use crate::ast;
use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::schema::Schema;
use std::collections::HashMap;

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

/// Information associated with GraphQLType::Enum
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLEnumType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<NamedDirectiveRef>,
    pub name: String,
    pub variants: HashMap<String, EnumVariant>,
}

/// Represents a defined variant for some [GraphQLType::Enum].
#[derive(Clone, Debug, PartialEq)]
pub struct EnumVariant {
    pub def_location: loc::FilePosition,
}
impl DerefByName for EnumVariant {
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

/// Represents
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectFieldDef {
    pub def_location: loc::SchemaDefLocation,
    pub type_ref: GraphQLTypeRef,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputFieldDef {
    pub def_location: loc::SchemaDefLocation,
}

/// Information associated with GraphQLType::Object
#[derive(Clone, Debug, PartialEq)]
pub struct GraphQLObjectType {
    pub def_location: loc::FilePosition,
    pub directives: Vec<NamedDirectiveRef>,
    pub fields: HashMap<String, ObjectFieldDef>,
    pub name: String,
}

/// Represents a defined type
#[derive(Clone, Debug, PartialEq)]
pub enum GraphQLType {
    Bool,

    Enum(GraphQLEnumType),

    Float,

    ID,

    InputObject {
        def_location: loc::FilePosition,
        directives: Vec<NamedDirectiveRef>,
        fields: HashMap<String, InputFieldDef>,
        name: String,
    },

    Int,

    Interface {
        def_location: loc::FilePosition,
        directives: Vec<NamedDirectiveRef>,
        fields: HashMap<String, ObjectFieldDef>,
        name: String,
    },

    Object(GraphQLObjectType),

    Scalar {
        def_location: loc::SchemaDefLocation,
        directives: Vec<NamedDirectiveRef>,
        name: String,
    },

    String,

    Union {
        def_location: loc::FilePosition,
        directives: Vec<NamedDirectiveRef>,
        name: String,
        types: HashMap<String, GraphQLTypeRef>
    }
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
            GraphQLType::Enum(GraphQLEnumType { def_location, .. }) =>
                loc::SchemaDefLocation::Schema(def_location.clone()),
            GraphQLType::InputObject { def_location, .. } =>
                loc::SchemaDefLocation::Schema(def_location.clone()),
            GraphQLType::Interface { def_location, .. } =>
                loc::SchemaDefLocation::Schema(def_location.clone()),
            GraphQLType::Object(t) =>
                loc::SchemaDefLocation::Schema(t.def_location.clone()),
            GraphQLType::Scalar { def_location, .. } =>
                def_location.clone(),
            GraphQLType::Union { def_location, .. } =>
                loc::SchemaDefLocation::Schema(def_location.clone()),
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        match self {
            GraphQLType::Bool
                | GraphQLType::Float
                | GraphQLType::ID
                | GraphQLType::Int
                | GraphQLType::String => None,
            GraphQLType::Enum(GraphQLEnumType { name, .. }) => Some(name.as_str()),
            GraphQLType::InputObject { name, .. } => Some(name.as_str()),
            GraphQLType::Interface { name, .. } => Some(name.as_str()),
            GraphQLType::Object(t) => Some(t.name.as_str()),
            GraphQLType::Scalar { name, .. } => Some(name.as_str()),
            GraphQLType::Union { name, .. } => Some(name.as_str()),
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
/// The most common example of a GraphQLTypeRef is the type specification on
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
                        name.to_string(),
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

pub type NamedDirectiveRef = NamedRef<Schema, Directive>;
pub type NamedEnumVariantRef = NamedRef<GraphQLEnumType, EnumVariant>;
pub type NamedGraphQLTypeRef = NamedRef<Schema, GraphQLType>;
