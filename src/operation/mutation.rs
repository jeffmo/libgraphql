use crate::ast;
use crate::operation::MutationBuilder;
use crate::operation::MutationBuildError;
use crate::operation::Operation;
use crate::operation::OperationImpl;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::DirectiveAnnotation;
use crate::types::ObjectType;
use std::collections::BTreeMap;
use inherent::inherent;
use std::path::Path;

type Result<T> = std::result::Result<T, MutationBuildError>;
type TOperationImpl<'schema> = OperationImpl<
    'schema,
    ast::operation::Mutation,
    MutationBuildError,
    Mutation<'schema>,
    MutationBuilder<'schema>,
>;

/// Represents a Mutation operation over a given [Schema].
#[derive(Debug)]
pub struct Mutation<'schema>(pub(super) TOperationImpl<'schema>);

#[inherent]
impl<'schema> Operation<
    'schema,
    ast::operation::Mutation,
    MutationBuildError,
    Self,
    MutationBuilder<'schema>,
> for Mutation<'schema> {
    /// Access the [DirectiveAnnotation]s defined on this [Query].
    pub fn annotations(&self) -> &Vec<DirectiveAnnotation> {
        self.0.annotations()
    }

    /// Convenience wrapper around [MutationBuilder::new()].
    pub fn builder(schema: &'schema Schema) -> Result<MutationBuilder<'schema>> {
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

    /// Access the [GraphQLType] that defines this [Mutation] operation.
    pub fn operation_type(&self) -> &ObjectType {
        self.0.schema.mutation_type().unwrap()
    }

    /// Access the name of this [Mutation] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.0.name()
    }

    /// Access the [SelectionSet] defined for this [Mutation].
    pub fn selection_set(&self) -> &SelectionSet<'schema> {
        self.0.selection_set()
    }

    /// Access the [Variable]s defined on this [Mutation].
    pub fn variables(&self) -> &BTreeMap<String, Variable> {
        self.0.variables()
    }
}
