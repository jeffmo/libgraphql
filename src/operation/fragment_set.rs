use crate::operation::NamedFragment;
use std::collections::HashMap;

#[derive(Debug)]
pub struct FragmentSet<'schema>(
    pub(crate) HashMap<String, NamedFragment<'schema>>,
);
impl<'schema> FragmentSet<'schema> {
    pub fn lookup_fragment(
        &self,
        fragment_name: &str,
    ) -> Option<&NamedFragment<'schema>> {
        self.0.get(fragment_name)
    }
}
