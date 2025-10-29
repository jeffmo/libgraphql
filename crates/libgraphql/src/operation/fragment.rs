use crate::DirectiveAnnotation;
use crate::loc;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::operation::FragmentRegistry;
use crate::operation::SelectionSet;
use crate::schema::Schema;
use crate::types::NamedGraphQLTypeRef;

/// TODO
#[derive(Clone, Debug, PartialEq)]
pub struct Fragment<'schema> {
    pub(super) def_location: loc::SourceLocation,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) name: String,
    pub(super) schema: &'schema Schema,
    pub(super) selection_set: SelectionSet<'schema>,
    pub(super) type_condition_ref: NamedGraphQLTypeRef,
}

impl<'schema> Fragment<'schema> {
    pub fn selection_set(&self) -> &SelectionSet<'schema> {
        &self.selection_set
    }
}

impl<'schema> DerefByName for Fragment<'schema> {
    type Source = FragmentRegistry<'schema>;
    type RefLocation = loc::SourceLocation;

    fn deref_name<'a>(
        source: &'a Self::Source,
        name: &str,
    ) -> std::result::Result<&'a Fragment<'schema>, DerefByNameError> {
        source.fragments.get(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string()),
        )
    }
}

pub type FragmentRef<'schema> = NamedRef<
    FragmentRegistry<'schema>,
    loc::SourceLocation,
    Fragment<'schema>,
>;
