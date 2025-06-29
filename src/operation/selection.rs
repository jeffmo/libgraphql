use crate::operation::FieldSelection;
use crate::operation::InlineFragmentSelection;
use crate::operation::NamedFragmentSelection;

#[derive(Clone, Debug, PartialEq)]
pub enum Selection<'fragset> {
    Field(FieldSelection<'fragset>),
    InlineFragment(InlineFragmentSelection<'fragset>),
    NamedFragment(NamedFragmentSelection<'fragset>),
}
