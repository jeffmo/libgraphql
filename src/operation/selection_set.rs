use crate::operation::FragmentRegistry;
use crate::operation::SelectionSetBuilder;
use crate::operation::Selection;
use crate::schema::Schema;

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionSet<'schema> {
    pub(super) selections: Vec<Selection<'schema>>,
}
impl<'schema> SelectionSet<'schema> {
    pub fn builder<'fragreg>(
        schema: &'schema Schema,
        fragment_registry: Option<&'fragreg FragmentRegistry<'schema>>,
    ) -> SelectionSetBuilder<'schema, 'fragreg> {
        SelectionSetBuilder::new(schema, fragment_registry)
    }

    pub fn selections(&self) -> &Vec<Selection<'schema>> {
        &self.selections
    }
}

