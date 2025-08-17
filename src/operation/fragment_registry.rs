use crate::operation::NamedFragment;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentRegistry<'schema> {
    pub(super) fragments: HashMap<String, NamedFragment<'schema>>,
}

impl<'schema> FragmentRegistry<'schema> {
    pub fn fragments(&self) -> &HashMap<String, NamedFragment<'schema>> {
        &self.fragments
    }
}
