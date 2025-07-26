use crate::DirectiveAnnotation;
use crate::loc;
use crate::schema::Schema;
use crate::types::NamedGraphQLTypeRef;
use crate::types::GraphQLType;
use std::collections::BTreeMap;

/// Represents a
/// [union type](https://spec.graphql.org/October2021/#sec-Unions) defined
/// within some [`Schema`](crate::schema::Schema).
#[derive(Clone, Debug, PartialEq)]
pub struct UnionType {
    pub(crate) def_location: loc::SchemaDefLocation,
    pub(super) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: String,
    pub(crate) members: BTreeMap<String, NamedGraphQLTypeRef>,
}
impl UnionType {
    /// The [`SchemaDefLocation`](loc::SchemaDefLocation) indicating where this
    /// [`UnionType`] was defined within the schema.
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`UnionType`].
    ///
    /// This list is guaranteed to be ordered the same as the order of
    /// annotations specified on the object type definition in the schema. Note
    /// that annotations added from a type extension will appear sequentially in
    /// the order they were applied on the type extension, but there is no
    /// guarantee about where in this list a given type extension's annotations
    /// are added.
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    /// The description of this [`UnionType`] as defined in the schema
    /// (e.g. in a """-string immediately before the type definition).
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// An ordered list of the names of each [`GraphQLType`] defined as a member
    /// of this union.
    ///
    /// The order of this `Vec` retains the same ordering as the order of
    /// members defined on the union type in the schema. Note that members added
    /// from type extensions will appear in the order they were specified on the
    /// type extension, but there is no guarantee about where in this list a
    /// given type extension's members will be added.
    pub fn member_type_names(&self) -> Vec<&str> {
        self.members.keys()
            .map(|type_name| type_name.as_str())
            .collect()
    }

    /// An ordered list of [`GraphQLType`]s defined as a member of this union.
    ///
    /// The order of this `Vec` retains the same ordering as the order of
    /// members defined on the union type in the schema. Note that members added
    /// from type extensions will appear in the order they were specified on the
    /// type extension, but there is no guarantee about where in this list a
    /// given type extension's members will be added.
    pub fn member_types<'schema>(
        &self,
        schema: &'schema Schema,
    ) -> Vec<&'schema GraphQLType> {
        self.members.values()
            .map(|type_ref| type_ref.deref(schema).unwrap())
            .collect()
    }

    /// The name of this [`UnionType`].
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
