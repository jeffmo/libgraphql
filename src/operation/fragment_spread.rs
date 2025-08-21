use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::FragmentRegistry;
use crate::operation::Fragment;
use crate::operation::FragmentRef;

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentSpread<'schema> {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) fragment: FragmentRef<'schema>,
}
impl<'schema> FragmentSpread<'schema> {
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    pub fn fragment<'fragreg: 'schema>(
        &self,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
    ) -> &Fragment<'schema> {
        self.fragment.deref(fragment_registry).expect(
            "fragment is present in the fragment set",
        )
    }

    pub fn fragment_name(&self) -> &str {
        self.fragment.name.as_str()
    }
}
