use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::schema::Schema;
use std::collections::HashMap;

/// Represents a defined directive.
#[derive(Clone, Debug)]
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
    pub def_location: loc::FilePosition,
}

/// Represents
#[derive(Clone, Debug)]
pub struct ObjectFieldDef {
    // Some field definitions are built-in (e.g. `__typename`).
    pub def_location: loc::SchemaDefLocation,
    pub type_ref: GraphQLTypeRef,
}

#[derive(Clone, Debug)]
pub struct InputFieldDef {
    pub def_location: loc::SchemaDefLocation,
}

/// Represents a defined type
#[derive(Clone, Debug)]
pub enum GraphQLType {
    Enum {
        def_location: loc::FilePosition,
        directives: Vec<NamedRef<Directive>>,
        values: HashMap<String, EnumValue>,
    },

    InputObject {
        def_location: loc::FilePosition,
        directives: Vec<NamedRef<Directive>>,
        fields: HashMap<String, InputFieldDef>,
    },

    Interface {
        def_location: loc::FilePosition,
        directives: Vec<NamedRef<Directive>>,
        fields: HashMap<String, ObjectFieldDef>,
    },

    Object {
        def_location: loc::FilePosition,
        directives: Vec<NamedRef<Directive>>,
        fields: HashMap<String, ObjectFieldDef>,
    },

    Scalar {
        def_location: loc::SchemaDefLocation,
        directives: Vec<NamedRef<Directive>>,
    },

    Union {
        def_location: loc::FilePosition,
        directives: Vec<NamedRef<Directive>>,
        types: HashMap<String, GraphQLTypeRef>
    }
}
impl GraphQLType {
    pub fn get_def_location(&self) -> loc::SchemaDefLocation {
        match self {
            GraphQLType::Enum { def_location, .. } =>
                loc::SchemaDefLocation::SchemaFile(def_location.clone()),
            GraphQLType::InputObject { def_location, .. } =>
                loc::SchemaDefLocation::SchemaFile(def_location.clone()),
            GraphQLType::Interface { def_location, .. } =>
                loc::SchemaDefLocation::SchemaFile(def_location.clone()),
            GraphQLType::Object { def_location, .. } =>
                loc::SchemaDefLocation::SchemaFile(def_location.clone()),
            GraphQLType::Scalar { def_location, .. } =>
                def_location.clone(),
            GraphQLType::Union { def_location, .. } =>
                loc::SchemaDefLocation::SchemaFile(def_location.clone()),
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
        ref_location: loc::FilePosition,
    },
    Named {
        nullable: bool,
        type_ref: NamedRef<GraphQLType>,
    }
}
impl GraphQLTypeRef {
    pub fn get_ref_location(&self) -> &loc::FilePosition {
        match self {
            GraphQLTypeRef::List { ref_location, .. } => ref_location,
            GraphQLTypeRef::Named { type_ref, .. } => type_ref.get_ref_location(),
        }
    }

    /*
    pub fn get_ref_position(&self) -> &loc::FilePosition {
        match self {
            GraphQLTypeRef::List { ref_location, .. } => ref_location,
            GraphQLTypeRef::Named { type_ref, .. } => type_ref.get_ref_location(),
        }
    }
    */

    pub fn is_nullable(&self) -> bool {
        match self {
            GraphQLTypeRef::List { nullable, .. } => *nullable,
            GraphQLTypeRef::Named { nullable, .. } => *nullable,
        }
    }
}
