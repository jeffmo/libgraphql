use crate::token::GraphQLToken;

/// A matched pair of delimiter tokens (parentheses, brackets,
/// or braces). Bundled into one struct so that an open
/// delimiter without a matching close is unrepresentable.
#[derive(Clone, Debug, PartialEq)]
pub struct DelimiterPair<'src> {
    pub open: GraphQLToken<'src>,
    pub close: GraphQLToken<'src>,
}
