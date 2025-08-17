use crate::operation::FragmentRegistry;
use crate::schema::Schema;
use crate::DirectiveAnnotation;
use crate::operation::Selection;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use std::path::Path;

/// Used to constrain the common functions that should be present on all
/// Operation builders.
///
/// This is distinct from
/// [`SpecificOperationBuilderTrait`](crate::operation::SpecificOperationBuilderTrait)
/// in that it specifies functions that should be present on all three specific
/// operation builders ([`MutationBuilder`](crate::operation::MutationBuilder),
/// [`QueryBuilder`](crate::operation::QueryBuilder), and
/// [`SubscriptionBuilder`](crate::operation::SubscriptionBuilder)) **as well
/// as** the generic [`OperationBuilder`](crate::operation::OperationBuilder)
/// struct.
pub(super) trait OperationBuilderTrait<
    'schema: 'fragreg,
    'fragreg,
    TAst,
    TError,
    TOperation,
> where Self: Sized {
    fn add_directive(
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
        fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
        ast: &TAst,
        file_path: Option<&Path>,
    ) -> Result<Self, TError>;

    fn from_file(
        schema: &'schema Schema,
        fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
        file_path: impl AsRef<Path>,
    ) -> Result<Self, TError>;

    fn from_str(
        schema: &'schema Schema,
        fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
        content: impl AsRef<str>,
        file_path: Option<&Path>,
    ) -> Result<Self, TError>;

    fn new(
        schema: &'schema Schema,
        fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
    ) -> Self;

    fn set_directives(
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
