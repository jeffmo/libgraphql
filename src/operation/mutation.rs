use crate::ast;
use crate::DirectiveAnnotation;
use crate::operation::MutationBuilder;
use crate::operation::MutationBuildError;
use crate::operation::Operation;
use crate::operation::OperationImpl;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::ObjectType;
use std::collections::BTreeMap;
use inherent::inherent;
use std::path::Path;

type Result<T> = std::result::Result<T, MutationBuildError>;
type TOperationImpl<'schema, 'fragset> = OperationImpl<
    'schema,
    'fragset,
    ast::operation::Mutation,
    MutationBuildError,
    Mutation<'schema, 'fragset>,
    MutationBuilder<'schema, 'fragset>,
>;

/// Represents a Mutation operation over a given [Schema].
#[derive(Debug, PartialEq)]
pub struct Mutation<'schema, 'fragset>(pub(super) TOperationImpl<'schema, 'fragset>);

#[inherent]
impl<'schema, 'fragset> Operation<
    'schema,
    'fragset,
    ast::operation::Mutation,
    MutationBuildError,
    Self,
    MutationBuilder<'schema, 'fragset>,
> for Mutation<'schema, 'fragset> {
    /// Convenience wrapper around [MutationBuilder::new()].
    pub fn builder(schema: &'schema Schema) -> Result<MutationBuilder<'schema, 'fragset>> {
        OperationImpl::builder(schema)
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`Mutation`].
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        self.0.directives()
    }

    /// Convenience wrapper around [MutationBuilder::from_ast()].
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Mutation,
    ) -> Result<Mutation<'schema, 'fragset>> {
        OperationImpl::from_ast(schema, file_path, def)
    }

    /// Access the [ObjectType] that defines this [Mutation] operation.
    pub fn operation_type(&self) -> &ObjectType {
        self.0.schema.mutation_type().unwrap()
    }

    /// Access the name of this [Mutation] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.0.name()
    }

    /// Access the [SelectionSet] defined for this [Mutation].
    pub fn selection_set(&self) -> &SelectionSet<'fragset> {
        self.0.selection_set()
    }

    /// Access the [Variable]s defined on this [Mutation].
    pub fn variables(&self) -> &BTreeMap<String, Variable> {
        self.0.variables()
    }
}
