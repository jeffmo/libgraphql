use crate::DirectiveAnnotation;
use crate::operation::Operation;
use crate::operation::Selection;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::Schema;
use std::path::Path;

/// Pretty much just used to constrain the common aspects of the 3 different
/// builder APIs to stay consistent.
pub(super) trait OperationBuilder<
    'schema,
    TAst,
    TError,
    TOperation: Operation<'schema, TAst, TError, TOperation, Self>,
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

    fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: TAst,
    ) -> Result<TOperation, TError>;

    fn new(schema: &'schema Schema) -> Result<Self, TError>;

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
