use crate::ast;
use crate::operation::Operation;
use crate::operation::OperationImpl;
use crate::operation::QueryBuilder;
use crate::operation::QueryBuildError;
use crate::schema::Schema;
use inherent::inherent;
use std::path::Path;

type Result<T> = std::result::Result<T, QueryBuildError>;

/// Represents a Query operation over a given [Schema].
#[derive(Debug)]
pub struct Query<'schema>(pub(super) OperationImpl<
    'schema,
    ast::operation::Query,
    QueryBuildError,
    Query<'schema>,
    QueryBuilder<'schema>,
>);

#[inherent]
impl<'schema> Operation<
    'schema,
    ast::operation::Query,
    QueryBuildError,
    Self,
    QueryBuilder<'schema>,
> for Query<'schema> {
    /// Convenience wrapper around [QueryBuilder::new()].
    pub fn builder(schema: &'schema Schema) -> QueryBuilder<'schema> {
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
}
