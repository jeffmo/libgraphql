use crate::operation::FieldSelection;
use crate::operation::InlineFragmentSelection;
use crate::operation::NamedFragmentSelection;

#[derive(Clone, Debug, PartialEq)]
pub enum Selection<'schema> {
    Field(FieldSelection<'schema>),
    InlineFragment(InlineFragmentSelection<'schema>),
    NamedFragment(NamedFragmentSelection<'schema>),
}
