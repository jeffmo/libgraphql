use crate::ast;
use crate::operation::Operation;
use crate::operation::OperationImpl;
use crate::operation::QueryBuilder;
use crate::operation::QueryBuildError;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::DirectiveAnnotation;
use crate::types::ObjectType;
use std::collections::BTreeMap;
use inherent::inherent;
use std::path::Path;

type Result<T> = std::result::Result<T, QueryBuildError>;
type TOperationImpl<'schema> = OperationImpl<
    'schema,
    ast::operation::Query,
    QueryBuildError,
    Query<'schema>,
    QueryBuilder<'schema>,
>;


/// Represents a Query operation over a given [Schema].
#[derive(Debug)]
pub struct Query<'schema>(pub(super) TOperationImpl<'schema>);

#[inherent]
impl<'schema> Operation<
    'schema,
    ast::operation::Query,
    QueryBuildError,
    Self,
    QueryBuilder<'schema>,
> for Query<'schema> {
    /// Access the [DirectiveAnnotation]s defined on this [Query].
    pub fn annotations(&self) -> &Vec<DirectiveAnnotation> {
        self.0.annotations()
    }

    /// Convenience wrapper around [QueryBuilder::new()].
    pub fn builder(schema: &'schema Schema) -> Result<QueryBuilder<'schema>> {
        OperationImpl::builder(schema)
    }

    /// Convenience wrapper around [QueryBuilder::from_ast()].
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Query,
    ) -> Result<Query<'schema>> {
        OperationImpl::from_ast(schema, file_path, def)
    }

    /// Access the [GraphQLType] that defines this [Query] operation.
    pub fn operation_type(&self) -> &ObjectType {
        self.0.schema.query_type()
    }

    /// Access the name of this [Query] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.0.name()
    }

    /// Access the [SelectionSet] defined for this [Query].
    pub fn selection_set(&self) -> &SelectionSet<'schema> {
        self.0.selection_set()
    }

    /// Access the [Variable]s defined on this [Query].
    pub fn variables(&self) -> &BTreeMap<String, Variable> {
        self.0.variables()
    }
}
