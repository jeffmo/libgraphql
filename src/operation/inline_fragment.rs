use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::SelectionSet;
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;

#[derive(Clone, Debug, PartialEq)]
pub struct InlineFragment<'schema> {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) selection_set: SelectionSet<'schema>,
    pub(super) type_condition: Option<NamedGraphQLTypeRef>,
}
impl<'schema> InlineFragment<'schema> {
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    pub fn selection_set(&self) -> &SelectionSet<'schema> {
        &self.selection_set
    }

    pub fn type_condition_on(
        &self,
        schema: &'schema Schema,
    ) -> Option<&'schema GraphQLType> {
        self.type_condition.as_ref().map(|graphql_type| {
            graphql_type.deref(schema).expect(
                "type is present in schema",
            )
        })
    }
}
