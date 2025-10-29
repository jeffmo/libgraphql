use crate::operation::FieldSelection;
use crate::operation::InlineFragment;
use crate::operation::FragmentSpread;

#[derive(Clone, Debug, PartialEq)]
pub enum Selection<'schema> {
    Field(FieldSelection<'schema>),
    FragmentSpread(FragmentSpread<'schema>),
    InlineFragment(InlineFragment<'schema>),
}
