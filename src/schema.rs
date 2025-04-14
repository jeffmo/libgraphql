use crate::SchemaBuilder;
use crate::types::Directive;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use crate::types::ObjectType;
use std::collections::HashMap;

/// Represents a fully typechecked and immutable GraphQL schema.
#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    pub(super) directive_defs: HashMap<String, Directive>,
    pub(super) query_type: NamedGraphQLTypeRef,
    pub(super) mutation_type: Option<NamedGraphQLTypeRef>,
    pub(super) subscription_type: Option<NamedGraphQLTypeRef>,
    pub(super) types: HashMap<String, GraphQLType>,
}
impl Schema {
    /// Helper function that just delegates to [SchemaBuilder::new()].
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }

    /// Returns this [Schema]'s Mutation[^note] root operation type (if one was
    /// defined).
    ///
    /// [^note] It is ***strongly*** recommended that you use
    /// [Schema::mutation_type()] in favor of looking for an [ObjectType] whose
    /// name is `"Mutation"`. GraphQL [defines an object type named "Mutation"
    /// as the _default_ Mutation type
    /// ](https://spec.graphql.org/October2021/#sec-Root-Operation-Types.Default-Root-Operation-Type-Names),
    /// but it is aslo [possible to override this default
    /// ](https://spec.graphql.org/October2021/#RootOperationTypeDefinition) and
    /// use a differently-named [ObjectType] instead. [Schema::mutation_type()]
    /// factors in any such override and will return the _correct_ [ObjectType]
    /// for this schema.
    pub fn mutation_type(&self) -> Option<&ObjectType> {
        self.mutation_type.as_ref().map(
            |named_ref| named_ref.deref(self).unwrap().unwrap_object()
        )
    }

    /// Returns this [Schema]'s Query[^note] root operation type.
    //
    /// [^note] It is ***strongly*** recommended that you use
    /// [Schema::query_type()] in favor of looking for an [ObjectType] whose
    /// name is `"Query"`. GraphQL [defines an object type named "Query" as the
    /// _default_ Query type
    /// ](https://spec.graphql.org/October2021/#sec-Root-Operation-Types.Default-Root-Operation-Type-Names),
    /// but it is aslo [possible to override this default
    /// ](https://spec.graphql.org/October2021/#RootOperationTypeDefinition) and
    /// use a differently-named [ObjectType] instead. [Schema::query_type()]
    /// factors in any such override and will return the _correct_ [ObjectType]
    /// for this schema.
    pub fn query_type(&self) -> &ObjectType {
        self.query_type.deref(self).unwrap().unwrap_object()
    }

    /// Looks up a [GraphQLType] by name.
    ///
    /// Note that this will return both schema-defined types as well as built-in
    /// types like `"Boolean"` -> [GraphQLType::Bool], `"ID"` ->
    /// [GraphQLType::ID], etc.
    pub fn lookup_type(&self, type_name: &str) -> Option<&GraphQLType> {
        self.types.get(type_name)
    }

    /// Returns this [Schema]'s Subscription[^note] root operation type.
    //
    /// [^note] It is ***strongly*** recommended that you use
    /// [Schema::subscription_type()] in favor of looking for an [ObjectType]
    /// whose name is `"Subscription"`. GraphQL [defines an object type named
    /// "Subscription" as the _default_ Subscription type
    /// ](https://spec.graphql.org/October2021/#sec-Root-Operation-Types.Default-Root-Operation-Type-Names),
    /// but it is aslo [possible to override this default
    /// ](https://spec.graphql.org/October2021/#RootOperationTypeDefinition) and
    /// use a differently-named [ObjectType] instead.
    /// [Schema::subscription_type()] factors in any such override and will
    /// return the _correct_ [ObjectType] for this schema.
    pub fn subscription_type(&self) -> Option<&ObjectType> {
        self.subscription_type.as_ref().map(
            |named_ref| named_ref.deref(self).unwrap().unwrap_object()
        )
    }
}
