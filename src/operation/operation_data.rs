use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use indexmap::IndexMap;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct OperationData<'schema, 'fragset> {
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) def_location: Option<loc::FilePosition>,
    pub(super) name: Option<String>,
    pub(super) schema: &'schema Schema,
    pub(super) selection_set: SelectionSet<'fragset>,
    pub(super) variables: IndexMap<String, Variable>,
}
