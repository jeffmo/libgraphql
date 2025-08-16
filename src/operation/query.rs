use crate::ast;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::FragmentSet;
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
pub struct Query<'schema: 'fragset, 'fragset>(
    pub(super) OperationData<'schema, 'fragset>,
);

#[inherent]
impl<'schema, 'fragset> OperationTrait<
    'schema,
    'fragset,
    ast::operation::Query,
    QueryBuildError,
    QueryBuilder<'schema, 'fragset>,
> for Query<'schema, 'fragset> {
    /// Convenience wrapper around [`QueryBuilder::new()`].
    pub fn builder(
        schema: &'schema Schema,
        fragset: Option<&'fragset FragmentSet<'schema>>,
    ) -> QueryBuilder<'schema, 'fragset> {
        QueryBuilder::new(schema, fragset)
    }

    /// The [`DefLocation`](loc::FilePosition) indicating where this
    /// [`Query`] was defined.
    pub fn def_location(&self) -> Option<&loc::FilePosition> {
        self.0.def_location.as_ref()
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`Query`].
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.0.directives
    }

    /// Access the name of this [`Query`] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    /// Access the [`SelectionSet`] defined for this [`Query`].
    pub fn selection_set(&self) -> &SelectionSet<'fragset> {
        &self.0.selection_set
    }

    /// Access the [`Variable`]s defined on this [`Query`].
    pub fn variables(&self) -> &IndexMap<String, Variable> {
        &self.0.variables
    }
}
