use crate::operation::Selection;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::Schema;
use crate::types::DirectiveAnnotation;

pub(super) trait OperationBuilder<
    'schema,
    TOperation,
    TError,
> where Self: Sized {
    fn add_annotation(
        self,
        annot: DirectiveAnnotation,
    ) -> Result<Self, TError>;

    fn add_selection(
        self,
        selection: Selection<'schema>,
    ) -> Result<Self, TError>;

    fn add_variable(
        self,
        variable: Variable,
    ) -> Result<Self, TError>;

    fn build(self) -> Result<TOperation, TError>;

    fn new(schema: &'schema Schema) -> Self;

    fn set_annotations(
        self,
        annots: &[DirectiveAnnotation],
    ) -> Result<Self, TError>;

    fn set_name(
        self,
        name: Option<String>,
    ) -> Result<Self, TError>;

    fn set_selection_set(
        self,
        selection_set: SelectionSet<'schema>,
    ) -> Result<Self, TError>;

    fn set_variables(
        self,
        variables: Vec<Variable>,
    ) -> Result<Self, TError>;
}
