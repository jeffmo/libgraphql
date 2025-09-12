use crate::ReadOnlyMap;
use crate::schema::SchemaBuilder;
use crate::types::Directive;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use std::collections::HashMap;

/// Represents a fully typechecked and immutable GraphQL schema.
#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    pub(crate) directive_defs: HashMap<String, Directive>,
    pub(crate) query_type: NamedGraphQLTypeRef,
    pub(crate) mutation_type: Option<NamedGraphQLTypeRef>,
    pub(crate) subscription_type: Option<NamedGraphQLTypeRef>,
    pub(crate) types: HashMap<String, GraphQLType>,
}
impl Schema {
    /// Returns a map from DirectiveName ([`String`]) -> [`Directive`] for *all*
    /// directives (including built-in GraphQL directives) provided by this
    /// [`Schema`].
    ///
    /// > **⚠️ NOTE:** This map includes directives defined directly by this
    /// > [`Schema`] as well as implicitly-defined, built-in directives like
    /// > [`Directive::Deprecated`], [`Directive::Include`],
    /// > [`Directive::Skip`], etc.
    pub fn all_directives(&self) -> ReadOnlyMap<'_, String, Directive> {
        ReadOnlyMap::new(&self.directive_defs, None)
    }

    /// Returns a map from TypeName ([`String`]) -> [`GraphQLType`] for *all*
    /// types (including built-in GraphQL types) provided by this [`Schema`].
    ///
    /// > **⚠️ NOTE:** This map includes types defined directly by this
    /// > [`Schema`] as well as implicitly-defined, built-in types like
    /// > [`GraphQLType::Bool`], [`GraphQLType::Float`], [`GraphQLType::ID`],
    /// > etc.
    pub fn all_types(&self) -> ReadOnlyMap<'_, String, GraphQLType> {
        ReadOnlyMap::new(&self.types, None)
    }

    /// Helper function that just delegates to [`SchemaBuilder::new()`].
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }

    /// Returns this [`Schema`]'s Mutation root operation type (if one was
    /// defined).
    ///
    /// > **⚠️ NOTE:** It is ***strongly*** recommended that you use
    /// > [`Schema::mutation_type()`] in favor of looking for an
    /// > [`ObjectType`](crate::types::ObjectType) whose name is `"Mutation"`.
    /// > GraphQL [defines an object type named "Mutation" as the _default_
    /// > Mutation type ](https://spec.graphql.org/October2021/#sec-Root-Operation-Types.Default-Root-Operation-Type-Names),
    /// > but it is aslo [possible to override this default
    /// > ](https://spec.graphql.org/October2021/#RootOperationTypeDefinition) and
    /// > use a differently-named [`ObjectType`](crate::types::ObjectType)
    /// > instead. [`Schema::mutation_type()`] factors in any such override and
    /// > will return the _correct_ [`ObjectType`](crate::types::ObjectType) for
    /// > this schema.
    pub fn mutation_type(&self) -> Option<&GraphQLType> {
        self.mutation_type.as_ref().map(|named_ref| {
            named_ref.deref(self)
                .expect("type is present in schema")
        })
    }

    /// Returns this [`Schema`]'s Query root operation type.
    //
    /// > **⚠️ NOTE**: It is ***strongly*** recommended that you use
    /// > [`Schema::query_type()`] in favor of looking for an
    /// > [`ObjectType`](crate::types::ObjectType) whose name is `"Query"`.
    /// > GraphQL [defines an object type named "Query" as the _default_ Query
    /// > type](https://spec.graphql.org/October2021/#sec-Root-Operation-Types.Default-Root-Operation-Type-Names),
    /// > but it is aslo [possible to override this default
    /// > ](https://spec.graphql.org/October2021/#RootOperationTypeDefinition) and
    /// > use a differently-named [`ObjectType`](crate::types::ObjectType)
    /// > instead. [`Schema::query_type()`] factors in any such override and will
    /// > return the _correct_ [`ObjectType`](crate::types::ObjectType) for this
    /// > schema.
    pub fn query_type(&self) -> &GraphQLType {
        self.query_type.deref(self)
            .expect("type is present in schema")
    }

    /// Returns a map from DirectiveName ([`String`]) -> [`Directive`] for all
    /// ***non-builtin*** directives provided by this [`Schema`].
    ///
    /// > **⚠️ NOTE:** This map only includes directives that are defined
    /// > directly by this [`Schema`]. It does not include any of the
    /// > implicitly-defined, built-in directives like
    /// > [`Directive::Deprecated`], [`Directive::Include`],
    /// > [`Directive::Skip`], etc.
    pub fn schema_directives(&self) -> ReadOnlyMap<'_, String, Directive> {
        ReadOnlyMap::new(
            &self.directive_defs,
            Some(|(_, directive)| directive.is_builtin()),
        )
    }

    /// Returns a map from TypeName ([`String`]) -> [`GraphQLType`] for all
    /// ***non-builtin*** types provided by this [`Schema`].
    ///
    /// > **⚠️ NOTE:** This map only includes types that are defined directly by
    /// > this [`Schema`]. It does not include any of the implicitly-defined,
    /// > built-in types like [`GraphQLType::Bool`], [`GraphQLType::Float`],
    /// > [`GraphQLType::ID`], etc.
    pub fn schema_types(&self) -> ReadOnlyMap<'_, String, GraphQLType> {
        ReadOnlyMap::new(
            &self.types,
            Some(|(_, graphql_type)| graphql_type.is_builtin()),
        )
    }

    /// Returns this [`Schema`]'s Subscription root operation type.
    //
    /// > **⚠️ NOTE**: It is ***strongly*** recommended that you use
    /// > [`Schema::subscription_type()`] in favor of looking for an
    /// > [`ObjectType`](crate::types::ObjectType) whose name is `"Subscription"`.
    /// > GraphQL [defines an object type named "Subscription" as the _default_
    /// > Subscription type](https://spec.graphql.org/October2021/#sec-Root-Operation-Types.Default-Root-Operation-Type-Names),
    /// > but it is aslo [possible to override this default
    /// > ](https://spec.graphql.org/October2021/#RootOperationTypeDefinition) and
    /// > use a differently-named [`ObjectType`](crate::types::ObjectType)
    /// > instead. [`Schema::subscription_type()`] factors in any such override
    /// > and will return the _correct_ [`ObjectType`](crate::types::ObjectType)
    /// > for this schema.
    pub fn subscription_type(&self) -> Option<&GraphQLType> {
        self.subscription_type.as_ref().map(|named_ref| {
            named_ref.deref(self)
                .expect("type is present in schema")
        })
    }
}
