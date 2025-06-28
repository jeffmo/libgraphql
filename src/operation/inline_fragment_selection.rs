use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::SelectionSet;
use crate::schema::Schema;
use crate::types::GraphQLType;
use crate::types::NamedGraphQLTypeRef;

#[derive(Debug)]
pub struct InlineFragmentSelection<'fragset> {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) selection_set: SelectionSet<'fragset>,
    pub(super) type_condition: Option<NamedGraphQLTypeRef>,
}
impl<'fragset> InlineFragmentSelection<'fragset> {
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    pub fn selection_set(&self) -> &SelectionSet<'fragset> {
        &self.selection_set
    }

    pub fn type_condition_on<'schema>(
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
