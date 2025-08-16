use crate::operation::FragmentSet;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::OperationBuilderTrait;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use indexmap::IndexMap;

// Implements the set of things
pub(super) trait OperationTrait<
    'schema,
    'fragset,
    TAst,
    TBuildError,
    TBuilder: OperationBuilderTrait<'schema, 'fragset, TAst, TBuildError, Self>,
> where Self: Sized {
    fn builder(
        schema: &'schema Schema,
        fragset: Option<&'fragset FragmentSet<'schema>>,
    ) -> TBuilder;
    fn def_location(&self) -> Option<&loc::FilePosition>;
    fn directives(&self) -> &Vec<DirectiveAnnotation>;
    fn name(&self) -> Option<&str>;
    fn selection_set(&self) -> &SelectionSet<'fragset>;
    fn variables(&self) -> &IndexMap<String, Variable>;
}
