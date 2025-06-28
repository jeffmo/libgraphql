use crate::ast;
use crate::DirectiveAnnotation;
use crate::operation::Operation;
use crate::operation::OperationImpl;
use crate::operation::QueryBuilder;
use crate::operation::QueryBuildError;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::ObjectType;
use std::collections::BTreeMap;
use inherent::inherent;
use std::path::Path;

type Result<T> = std::result::Result<T, QueryBuildError>;
type TOperationImpl<'schema, 'fragset> = OperationImpl<
    'schema,
    'fragset,
    ast::operation::Query,
    QueryBuildError,
    Query<'schema, 'fragset>,
    QueryBuilder<'schema, 'fragset>,
>;

/// Represents a Query operation over a given [`Schema`].
#[derive(Debug)]
pub struct Query<'schema, 'fragset>(pub(super) TOperationImpl<'schema, 'fragset>);

#[inherent]
impl<'schema, 'fragset> Operation<
    'schema,
    'fragset,
    ast::operation::Query,
    QueryBuildError,
    Self,
    QueryBuilder<'schema, 'fragset>,
> for Query<'schema, 'fragset> {
    /// Convenience wrapper around [`QueryBuilder::new()`].
    pub fn builder(schema: &'schema Schema) -> Result<QueryBuilder<'schema, 'fragset>> {
        OperationImpl::builder(schema)
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`Query`].
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        self.0.directives()
    }

    /// Convenience wrapper around [`QueryBuilder::from_ast()`].
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Query,
    ) -> Result<Query<'schema, 'fragset>> {
        OperationImpl::from_ast(schema, file_path, def)
    }

    /// Access the [`ObjectType`] that defines this [`Query`] operation.
    pub fn operation_type(&self) -> &ObjectType {
        self.0.schema.query_type()
    }

    /// Access the name of this [`Query`] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.0.name()
    }

    /// Access the [`SelectionSet`] defined for this [`Query`].
    pub fn selection_set(&self) -> &SelectionSet<'fragset> {
        self.0.selection_set()
    }

    /// Access the [`Variable`]s defined on this [`Query`].
    pub fn variables(&self) -> &BTreeMap<String, Variable> {
        self.0.variables()
    }
}
