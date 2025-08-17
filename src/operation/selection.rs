use crate::operation::FieldSelection;
use crate::operation::InlineFragmentSelection;
use crate::operation::NamedFragmentSelection;

#[derive(Clone, Debug, PartialEq)]
pub enum Selection<'fragreg> {
    Field(FieldSelection<'fragreg>),
    InlineFragment(InlineFragmentSelection<'fragreg>),
    NamedFragment(NamedFragmentSelection<'fragreg>),
}
