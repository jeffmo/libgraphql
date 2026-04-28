use crate::names::DirectiveName;
use crate::names::TypeName;
use crate::schema_source_map::SchemaSourceMap;
use crate::types::DirectiveDefinition;
use crate::types::EnumType;
use crate::types::GraphQLType;
use crate::types::InputObjectType;
use crate::types::InterfaceType;
use crate::types::ObjectType;
use crate::types::ScalarType;
use crate::types::UnionType;
use indexmap::IndexMap;

/// A fully validated, immutable GraphQL schema.
///
/// Produced by
/// [`SchemaBuilder::build()`](crate::schema::SchemaBuilder::build)
/// after all type definitions, directive definitions, and schema
/// metadata have been loaded and validated against the
/// [GraphQL specification](https://spec.graphql.org/September2025/#sec-Schema).
///
/// `Schema` provides typed accessors for looking up types by name
/// and category, querying root operation types, and resolving
/// source locations via stored source maps.
///
/// # Example
///
/// ```rust
/// # use libgraphql_core_v1 as libgraphql_core;
/// use libgraphql_core::schema::SchemaBuilder;
///
/// let schema = SchemaBuilder::build_from_str(
///     "type Query { hello: String }",
/// ).unwrap();
///
/// assert_eq!(schema.query_type_name().as_str(), "Query");
/// assert!(schema.query_type().is_some());
/// assert!(schema.object_type("Query").is_some());
/// ```
///
/// See [Schema](https://spec.graphql.org/September2025/#sec-Schema).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Schema {
    pub(crate) directive_defs: IndexMap<DirectiveName, DirectiveDefinition>,
    pub(crate) mutation_type_name: Option<TypeName>,
    pub(crate) query_type_name: TypeName,
    pub(crate) source_maps: Vec<SchemaSourceMap>,
    pub(crate) subscription_type_name: Option<TypeName>,
    pub(crate) types: IndexMap<TypeName, GraphQLType>,
}

impl Schema {
    // ---------------------------------------------------------
    // Generic lookups
    // ---------------------------------------------------------

    /// Returns the type with the given name, or `None` if no such
    /// type is defined.
    ///
    /// See [Types](https://spec.graphql.org/September2025/#sec-Types).
    pub fn get_type(&self, name: &str) -> Option<&GraphQLType> {
        self.types.get(name)
    }

    /// Returns the directive definition with the given name, or
    /// `None` if no such directive is defined.
    ///
    /// See [Type System Directives](https://spec.graphql.org/September2025/#sec-Type-System.Directives).
    pub fn get_directive(&self, name: &str) -> Option<&DirectiveDefinition> {
        self.directive_defs.get(name)
    }

    // ---------------------------------------------------------
    // Typed lookups
    // ---------------------------------------------------------

    /// Returns the named type as an [`ObjectType`], or `None` if
    /// not found or not an object type.
    ///
    /// See [Objects](https://spec.graphql.org/September2025/#sec-Objects).
    pub fn object_type(&self, name: &str) -> Option<&ObjectType> {
        self.types.get(name).and_then(|t| t.as_object())
    }

    /// Returns the named type as an [`InterfaceType`], or `None`
    /// if not found or not an interface type.
    ///
    /// See [Interfaces](https://spec.graphql.org/September2025/#sec-Interfaces).
    pub fn interface_type(&self, name: &str) -> Option<&InterfaceType> {
        self.types.get(name).and_then(|t| t.as_interface())
    }

    /// Returns the named type as an [`EnumType`], or `None` if
    /// not found or not an enum type.
    ///
    /// See [Enums](https://spec.graphql.org/September2025/#sec-Enums).
    pub fn enum_type(&self, name: &str) -> Option<&EnumType> {
        self.types.get(name).and_then(|t| t.as_enum())
    }

    /// Returns the named type as a [`ScalarType`], or `None` if
    /// not found or not a scalar type.
    ///
    /// See [Scalars](https://spec.graphql.org/September2025/#sec-Scalars).
    pub fn scalar_type(&self, name: &str) -> Option<&ScalarType> {
        self.types.get(name).and_then(|t| t.as_scalar())
    }

    /// Returns the named type as a [`UnionType`], or `None` if
    /// not found or not a union type.
    ///
    /// See [Unions](https://spec.graphql.org/September2025/#sec-Unions).
    pub fn union_type(&self, name: &str) -> Option<&UnionType> {
        self.types.get(name).and_then(|t| t.as_union())
    }

    /// Returns the named type as an [`InputObjectType`], or
    /// `None` if not found or not an input object type.
    ///
    /// See [Input Objects](https://spec.graphql.org/September2025/#sec-Input-Objects).
    pub fn input_object_type(&self, name: &str) -> Option<&InputObjectType> {
        self.types.get(name).and_then(|t| t.as_input_object())
    }

    // ---------------------------------------------------------
    // Typed iterators
    // ---------------------------------------------------------

    /// Returns an iterator over all object types in the schema.
    ///
    /// See [Objects](https://spec.graphql.org/September2025/#sec-Objects).
    pub fn object_types(&self) -> impl Iterator<Item = &ObjectType> {
        self.types.values().filter_map(|t| t.as_object())
    }

    /// Returns an iterator over all interface types in the schema.
    ///
    /// See [Interfaces](https://spec.graphql.org/September2025/#sec-Interfaces).
    pub fn interface_types(&self) -> impl Iterator<Item = &InterfaceType> {
        self.types.values().filter_map(|t| t.as_interface())
    }

    /// Returns an iterator over all enum types in the schema.
    ///
    /// See [Enums](https://spec.graphql.org/September2025/#sec-Enums).
    pub fn enum_types(&self) -> impl Iterator<Item = &EnumType> {
        self.types.values().filter_map(|t| t.as_enum())
    }

    /// Returns all types (objects and interfaces) that declare
    /// they implement the given interface name.
    ///
    /// This performs a linear scan of all types. For schemas with
    /// a large number of types where this is called frequently,
    /// consider building an index.
    ///
    /// See [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation()).
    pub fn types_implementing(
        &self,
        interface_name: &str,
    ) -> Vec<&GraphQLType> {
        self.types.values().filter(|t| {
            match t {
                GraphQLType::Object(obj) => {
                    obj.interfaces().iter().any(|l| {
                        l.value.as_str() == interface_name
                    })
                },
                GraphQLType::Interface(iface) => {
                    iface.interfaces().iter().any(|l| {
                        l.value.as_str() == interface_name
                    })
                },
                _ => false,
            }
        }).collect()
    }

    // ---------------------------------------------------------
    // Root operation types
    // ---------------------------------------------------------

    /// Returns the query root operation object type.
    ///
    /// Always `Some` for a valid schema -- every schema must
    /// define a query root type.
    ///
    /// See [Root Operation Types](https://spec.graphql.org/September2025/#sec-Root-Operation-Types).
    pub fn query_type(&self) -> Option<&ObjectType> {
        self.object_type(self.query_type_name.as_str())
    }

    /// Returns the name of the query root operation type.
    ///
    /// See [Root Operation Types](https://spec.graphql.org/September2025/#sec-Root-Operation-Types).
    pub fn query_type_name(&self) -> &TypeName {
        &self.query_type_name
    }

    /// Returns the mutation root operation object type, or `None`
    /// if the schema does not define a mutation type.
    ///
    /// See [Root Operation Types](https://spec.graphql.org/September2025/#sec-Root-Operation-Types).
    pub fn mutation_type(&self) -> Option<&ObjectType> {
        self.mutation_type_name
            .as_ref()
            .and_then(|name| self.object_type(name.as_str()))
    }

    /// Returns the name of the mutation root operation type, or
    /// `None` if not defined.
    ///
    /// See [Root Operation Types](https://spec.graphql.org/September2025/#sec-Root-Operation-Types).
    pub fn mutation_type_name(&self) -> Option<&TypeName> {
        self.mutation_type_name.as_ref()
    }

    /// Returns the subscription root operation object type, or
    /// `None` if the schema does not define a subscription type.
    ///
    /// See [Root Operation Types](https://spec.graphql.org/September2025/#sec-Root-Operation-Types).
    pub fn subscription_type(&self) -> Option<&ObjectType> {
        self.subscription_type_name
            .as_ref()
            .and_then(|name| self.object_type(name.as_str()))
    }

    /// Returns the name of the subscription root operation type,
    /// or `None` if not defined.
    ///
    /// See [Root Operation Types](https://spec.graphql.org/September2025/#sec-Root-Operation-Types).
    pub fn subscription_type_name(&self) -> Option<&TypeName> {
        self.subscription_type_name.as_ref()
    }

    // ---------------------------------------------------------
    // Collection accessors
    // ---------------------------------------------------------

    /// Returns all types registered in the schema, keyed by name.
    ///
    /// Includes both user-defined types and the five built-in
    /// scalar types.
    pub fn types(&self) -> &IndexMap<TypeName, GraphQLType> {
        &self.types
    }

    /// Returns all directive definitions registered in the schema,
    /// keyed by name.
    ///
    /// Includes both user-defined directives and the five built-in
    /// directives (`@skip`, `@include`, `@deprecated`,
    /// `@specifiedBy`, `@oneOf`).
    pub fn directive_defs(
        &self,
    ) -> &IndexMap<DirectiveName, DirectiveDefinition> {
        &self.directive_defs
    }

    /// Returns the source maps stored in this schema.
    ///
    /// Source maps allow resolving byte-offset spans to
    /// line/column positions within the original source text.
    pub fn source_maps(&self) -> &[SchemaSourceMap] {
        &self.source_maps
    }
}
