use crate::operation::FragmentRegistry;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::OperationBuilderTrait;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use indexmap::IndexMap;

// Implements the set of things
pub(super) trait OperationTrait<
    'schema: 'fragreg,
    'fragreg,
    TAst,
    TBuildError,
    TBuilder: OperationBuilderTrait<'schema, 'fragreg, TAst, TBuildError, Self>,
> where Self: Sized {
    fn builder(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
    ) -> TBuilder;
    fn def_location(&self) -> &loc::SourceLocation;
    fn directives(&self) -> &Vec<DirectiveAnnotation>;
    fn name(&self) -> Option<&str>;
    fn selection_set(&self) -> &SelectionSet<'fragreg>;
    fn variables(&self) -> &IndexMap<String, Variable>;
}
