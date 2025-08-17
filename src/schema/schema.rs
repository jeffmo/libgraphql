use crate::schema::SchemaBuilder;
use crate::types::Directive;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;
use crate::types::TypeAnnotation;
use std::collections::HashMap;

/// Represents a fully typechecked and immutable GraphQL schema.
#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    pub(crate) directive_defs: HashMap<String, Directive>,
    pub(crate) query_type: NamedGraphQLTypeRef,
    pub(crate) query_type_annotation: TypeAnnotation,
    pub(crate) mutation_type: Option<NamedGraphQLTypeRef>,
    pub(crate) mutation_type_annotation: Option<TypeAnnotation>,
    pub(crate) subscription_type: Option<NamedGraphQLTypeRef>,
    pub(crate) subscription_type_annotation: Option<TypeAnnotation>,
    pub(crate) types: HashMap<String, GraphQLType>,
}
impl Schema {
    /// Returns a [`HashMap<String, GraphQLType>`] containing all types defined
    /// within this [`Schema`].
    ///
    /// [^note] This map includes both types defined while building this
    /// [`Schema`] as well as implicitly-defined built-in types like
    /// [`GraphQLType::Bool`].
    pub fn all_types(&self) -> &HashMap<String, GraphQLType> {
        &self.types
    }

    /// Helper function that just delegates to [SchemaBuilder::new()].
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::new()
    }

    /// Looks up a [Directive] by name.
    ///
    /// Note that this will return both schema-defined types as well as built-in
    /// types like `"deprecated"` -> [Directive::Deprecated], `"include"` ->
    /// [Directive::Include], etc.
    pub fn lookup_directive(&self, directive_name: &str) -> Option<&Directive> {
        self.directive_defs.get(directive_name)
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
    pub fn mutation_type(&self) -> Option<&GraphQLType> {
        self.mutation_type.as_ref().map(|named_ref| {
            named_ref.deref(self)
                .expect("type is present in schema")
        })
    }

    pub(crate) fn mutation_type_annotation(&self) -> Option<&TypeAnnotation> {
        self.mutation_type_annotation.as_ref()
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
    pub fn query_type(&self) -> &GraphQLType {
        self.query_type.deref(self)
            .expect("type is present in schema")
    }

    pub(crate) fn query_type_annotation(&self) -> &TypeAnnotation {
        &self.query_type_annotation
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
    pub fn subscription_type(&self) -> Option<&GraphQLType> {
        self.subscription_type.as_ref().map(|named_ref| {
            named_ref.deref(self)
                .expect("type is present in schema")
        })
    }

    pub(crate) fn subscription_type_annotation(
        &self,
    ) -> Option<&TypeAnnotation> {
        self.subscription_type_annotation.as_ref()
    }
}
