use crate::operation::Fragment;
use std::{collections::HashMap, sync::OnceLock};

fn empty_fragment_registry() -> &'static FragmentRegistry<'static> {
    static EMPTY_FRAGMENT_REGISTRY: OnceLock<FragmentRegistry> = OnceLock::new();
    EMPTY_FRAGMENT_REGISTRY.get_or_init(|| {
        FragmentRegistry {
            fragments: HashMap::new(),
        }
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentRegistry<'schema> {
    pub(super) fragments: HashMap<String, Fragment<'schema>>,
}

impl<'schema> FragmentRegistry<'schema> {
    pub fn empty() -> &'static FragmentRegistry<'static> {
        empty_fragment_registry()
    }

    pub fn fragments(&self) -> &HashMap<String, Fragment<'schema>> {
        &self.fragments
    }
}
