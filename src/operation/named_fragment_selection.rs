use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::FragmentSet;
use crate::operation::NamedFragment;
use crate::operation::NamedFragmentRef;

#[derive(Clone, Debug, PartialEq)]
pub struct NamedFragmentSelection<'fragset> {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) fragment: NamedFragmentRef<'fragset>,
}
impl<'fragset> NamedFragmentSelection<'fragset> {
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    pub fn fragment(
        &self,
        fragment_set: &'fragset FragmentSet<'_>,
    ) -> &'fragset NamedFragment<'_> {
        self.fragment.deref(fragment_set).expect(
            "fragment is present in the fragment set",
        )
    }

    pub fn fragment_name(&self) -> &str {
        self.fragment.name.as_str()
    }
}
