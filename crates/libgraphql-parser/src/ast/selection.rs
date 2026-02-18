use crate::ast::AstNode;
use crate::ast::Field;
use crate::ast::FragmentSpread;
use crate::ast::InlineFragment;
use inherent::inherent;

/// A single selection within a selection set.
///
/// See
/// [Selection Sets](https://spec.graphql.org/September2025/#sec-Selection-Sets)
/// in the spec.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum Selection<'src> {
    Field(Field<'src>),
    FragmentSpread(FragmentSpread<'src>),
    InlineFragment(InlineFragment<'src>),
}

#[inherent]
impl AstNode for Selection<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        match self {
            Selection::Field(s) => {
                s.append_source(sink, source)
            },
            Selection::FragmentSpread(s) => {
                s.append_source(sink, source)
            },
            Selection::InlineFragment(s) => {
                s.append_source(sink, source)
            },
        }
    }
}
