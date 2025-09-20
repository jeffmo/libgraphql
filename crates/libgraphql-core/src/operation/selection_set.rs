use crate::operation::FieldSelection;
use crate::operation::FragmentRegistry;
use crate::operation::SelectionSetBuilder;
use crate::operation::Selection;
use crate::schema::Schema;

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionSet<'schema> {
    pub(super) selections: Vec<Selection<'schema>>,
    pub(super) schema: &'schema Schema,
}
impl<'schema> SelectionSet<'schema> {
    pub fn builder<'fragreg>(
        schema: &'schema Schema,
        fragment_registry: &'fragreg FragmentRegistry<'schema>,
    ) -> SelectionSetBuilder<'schema, 'fragreg> {
        SelectionSetBuilder::new(schema, fragment_registry)
    }

    pub fn selected_fields(
        &'schema self,
        fragment_registry: &'schema FragmentRegistry<'schema>,
    ) -> Box<dyn Iterator<Item = &'schema FieldSelection<'schema>> + 'schema> {
        Box::new(
            self.selections()
                .iter()
                .flat_map(|selection: &Selection<'schema>| match selection {
                    Selection::Field(field_selection) =>
                        Box::new(vec![field_selection].into_iter()),

                    Selection::FragmentSpread(frag_spread) => {
                        frag_spread.fragment(fragment_registry)
                            .selection_set()
                            .selected_fields(fragment_registry)
                    },

                    Selection::InlineFragment(inline_frag) =>
                        inline_frag.selection_set()
                            .selected_fields(fragment_registry),
                })
        )
    }

    pub fn selections(&self) -> &Vec<Selection<'schema>> {
        &self.selections
    }
}

