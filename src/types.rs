use crate::ast;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::schema::Schema;
use std::collections::HashMap;

/// Represents a defined directive.
#[derive(Clone, Debug)]
pub enum Directive {
    Custom {
        def_location: ast::FileLocation,
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
    fn deref_name<'a>(
        schema: &'a Schema,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        schema.directives.get(name).ok_or(DerefByNameError::DanglingReference)
    }
}

/// Represents a defined value for some [GraphQLType::Enum].
#[derive(Clone, Debug)]
pub struct EnumValue {
    pub def_location: ast::FileLocation,
}

/// Represents
#[derive(Clone, Debug)]
pub struct ObjectFieldDef {
    pub def_location: ast::FileLocation,
    pub type_ref: GraphQLTypeRef,
}

#[derive(Clone, Debug)]
pub struct InputFieldDef {
    pub def_location: ast::FileLocation,
}

/// Represents a defined type
#[derive(Clone, Debug)]
pub enum GraphQLType {
    Enum {
        def_location: ast::FileLocation,
        directives: Vec<NamedRef<Directive>>,
        values: HashMap<String, EnumValue>,
    },

    InputObject {
        def_location: ast::FileLocation,
        directives: Vec<NamedRef<Directive>>,
        fields: HashMap<String, InputFieldDef>,
    },

    Interface {
        def_location: ast::FileLocation,
        directives: Vec<NamedRef<Directive>>,
        fields: HashMap<String, ObjectFieldDef>,
    },

    Object {
        def_location: ast::FileLocation,
        directives: Vec<NamedRef<Directive>>,
        fields: HashMap<String, ObjectFieldDef>,
    },

    Scalar {
        def_location: ast::FileLocation,
        directives: Vec<NamedRef<Directive>>,
    },

    Union {
        def_location: ast::FileLocation,
        directives: Vec<NamedRef<Directive>>,
        types: HashMap<String, GraphQLTypeRef>
    }
}
impl GraphQLType {
    pub fn get_def_location(&self) -> &ast::FileLocation {
        match self {
            GraphQLType::Enum { def_location, .. } => def_location,
            GraphQLType::InputObject { def_location, .. } => def_location,
            GraphQLType::Interface { def_location, .. } => def_location,
            GraphQLType::Object { def_location, .. } => def_location,
            GraphQLType::Scalar { def_location, .. } => def_location,
            GraphQLType::Union { def_location, .. } => def_location,
        }
    }
}
impl DerefByName for GraphQLType {
    fn deref_name<'a>(
        schema: &'a Schema,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> {
        schema.types.get(name).ok_or(DerefByNameError::DanglingReference)
    }
}

/// Represents a reference to a type (e.g. a "type annotation").
///
/// The most common example of a GraphQLTypeRef is the type specification on
/// an Object field. These type specifications "reference" another defined type.
#[derive(Clone, Debug)]
pub enum GraphQLTypeRef {
    List {
        inner_type_ref: Box<GraphQLTypeRef>,
        nullable: bool,
        ref_location: ast::FileLocation,
    },
    Named {
        nullable: bool,
        type_ref: NamedRef<GraphQLType>,
    }
}
impl GraphQLTypeRef {
    pub fn get_ref_location(&self) -> &ast::FileLocation {
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
