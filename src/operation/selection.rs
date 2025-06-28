use crate::operation::FieldSelection;
use crate::operation::InlineFragmentSelection;
use crate::operation::NamedFragmentSelection;

#[derive(Debug)]
pub enum Selection<'fragset> {
    Field(FieldSelection<'fragset>),
    InlineFragment(InlineFragmentSelection<'fragset>),
    NamedFragment(NamedFragmentSelection<'fragset>),
}
