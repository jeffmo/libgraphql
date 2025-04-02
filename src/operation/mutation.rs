use crate::ast;
use crate::operation::MutationBuilder;
use crate::operation::MutationBuildError;
use crate::operation::Operation;
use crate::operation::OperationImpl;
use crate::schema::Schema;
use inherent::inherent;
use std::path::Path;

type Result<T> = std::result::Result<T, MutationBuildError>;

/// Represents a Mutation operation over a given [Schema].
#[derive(Debug)]
pub struct Mutation<'schema>(pub(super) OperationImpl<
    'schema,
    ast::operation::Mutation,
    MutationBuildError,
    Mutation<'schema>,
    MutationBuilder<'schema>,
>);

#[inherent]
impl<'schema> Operation<
    'schema,
    ast::operation::Mutation,
    MutationBuildError,
    Self,
    MutationBuilder<'schema>,
> for Mutation<'schema> {
    /// Convenience wrapper around [MutationBuilder::new()].
    pub fn builder(schema: &'schema Schema) -> MutationBuilder<'schema> {
        OperationImpl::builder(schema)
    }

    /// Convenience wrapper around [MutationBuilder::from_ast()].
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Mutation,
    ) -> Result<Mutation<'schema>> {
        OperationImpl::from_ast(schema, file_path, def)
    }
}
