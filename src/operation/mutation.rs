use crate::ast;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::MutationBuilder;
use crate::operation::MutationBuildError;
use crate::operation::OperationTrait;
use crate::operation::OperationData;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use indexmap::IndexMap;
use inherent::inherent;
use std::path::Path;

type Result<T> = std::result::Result<T, MutationBuildError>;
type TOperationData<'schema, 'fragset> = OperationData<'schema, 'fragset>;

/// Represents a Mutation operation over a given [Schema].
#[derive(Clone, Debug, PartialEq)]
pub struct Mutation<'schema, 'fragset>(pub(super) TOperationData<'schema, 'fragset>);

#[inherent]
impl<'schema, 'fragset> OperationTrait<
    'schema,
    'fragset,
    ast::operation::Mutation,
    MutationBuildError,
    Self,
    MutationBuilder<'schema, 'fragset>,
> for Mutation<'schema, 'fragset> {
    /// Convenience wrapper around [MutationBuilder::new()].
    pub fn builder(schema: &'schema Schema) -> Result<MutationBuilder<'schema, 'fragset>> {
        MutationBuilder::new(schema)
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`Mutation`].
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.0.directives
    }

    /// The [`DefLocation`](loc::FilePosition) indicating where this
    /// [`Mutation`] was defined.
    pub fn def_location(&self) -> Option<&loc::FilePosition> {
        self.0.def_location.as_ref()
    }

    /// Convenience wrapper around [MutationBuilder::from_ast()].
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Mutation,
    ) -> Result<Mutation<'schema, 'fragset>> {
        MutationBuilder::from_ast(schema, file_path, def)
    }

    /// Access the name of this [Mutation] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    /// Access the [SelectionSet] defined for this [Mutation].
    pub fn selection_set(&self) -> &SelectionSet<'fragset> {
        &self.0.selection_set
    }

    /// Access the [Variable]s defined on this [Mutation].
    pub fn variables(&self) -> &IndexMap<String, Variable> {
        &self.0.variables
    }
}
