use crate::ast;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::FragmentRegistry;
use crate::operation::MutationBuilder;
use crate::operation::MutationBuildError;
use crate::operation::OperationTrait;
use crate::operation::OperationData;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use indexmap::IndexMap;
use inherent::inherent;

/// Represents a Mutation operation over a given [Schema].
#[derive(Clone, Debug, PartialEq)]
pub struct Mutation<'schema: 'fragreg, 'fragreg>(
    pub(super) OperationData<'schema, 'fragreg>,
);

#[inherent]
impl<'schema: 'fragreg, 'fragreg> OperationTrait<
    'schema,
    'fragreg,
    ast::operation::Mutation,
    MutationBuildError,
    MutationBuilder<'schema, 'fragreg>,
> for Mutation<'schema, 'fragreg> {
    /// Convenience wrapper around [MutationBuilder::new()].
    pub fn builder(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
    ) -> MutationBuilder<'schema, 'fragreg> {
        MutationBuilder::new(schema, fragment_registry)
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

    /// Access the name of this [Mutation] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    /// Access the [SelectionSet] defined for this [Mutation].
    pub fn selection_set(&self) -> &SelectionSet<'fragreg> {
        &self.0.selection_set
    }

    /// Access the [Variable]s defined on this [Mutation].
    pub fn variables(&self) -> &IndexMap<String, Variable> {
        &self.0.variables
    }
}
