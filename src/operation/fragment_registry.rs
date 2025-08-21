use crate::operation::Fragment;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentRegistry<'schema> {
    pub(super) fragments: HashMap<String, Fragment<'schema>>,
}

impl<'schema> FragmentRegistry<'schema> {
    pub fn fragments(&self) -> &HashMap<String, Fragment<'schema>> {
        &self.fragments
    }
}
