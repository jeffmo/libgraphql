use crate::ast;
use crate::types::GraphQLType;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::FragmentRegistry;
use crate::operation::OperationTrait;
use crate::operation::OperationData;
use crate::operation::QueryBuilder;
use crate::operation::QueryBuildError;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use indexmap::IndexMap;
use inherent::inherent;

/// Represents a Query operation over a given [`Schema`].
#[derive(Clone, Debug, PartialEq)]
pub struct Query<'schema: 'fragreg, 'fragreg>(
    pub(super) OperationData<'schema, 'fragreg>,
);

#[inherent]
impl<'schema: 'fragreg, 'fragreg> OperationTrait<
    'schema,
    'fragreg,
    ast::operation::Query,
    QueryBuildError,
    QueryBuilder<'schema, 'fragreg>,
> for Query<'schema, 'fragreg> {
    /// Convenience wrapper around [`QueryBuilder::new()`].
    pub fn builder(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
    ) -> QueryBuilder<'schema, 'fragreg> {
        QueryBuilder::new(schema, fragment_registry)
    }

    /// The [`loc::SourceLocation`] indicating where this [`Query`] operation
    /// was defined.
    pub fn def_location(&self) -> &loc::SourceLocation {
        &self.0.def_location
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`Query`].
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.0.directives
    }

    /// Access the name of this [`Query`] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    /// Returns the root `Query` [`GraphQLType`] defined in the schema for this
    /// [`Query`] operation.
    pub fn root_graphql_type(&self, schema: &'schema Schema) -> &GraphQLType {
        schema.query_type()
    }

    /// Access the [`SelectionSet`] defined for this [`Query`].
    pub fn selection_set(&self) -> &SelectionSet<'fragreg> {
        &self.0.selection_set
    }

    /// Access the [`Variable`]s defined on this [`Query`].
    pub fn variables(&self) -> &IndexMap<String, Variable> {
        &self.0.variables
    }
}
