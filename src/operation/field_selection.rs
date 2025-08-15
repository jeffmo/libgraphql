use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::SelectionSet;
use crate::Value;
use indexmap::IndexMap;

#[derive(Clone, Debug, PartialEq)]
pub struct FieldSelection<'fragset> {
    pub(super) alias: Option<String>,
    pub(super) arguments: IndexMap<String, Value>,
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) field_name: String,
    pub(super) selection_set: SelectionSet<'fragset>,
}
impl<'fragset> FieldSelection<'fragset> {
    pub fn alias(&self) -> &Option<String> {
        &self.alias
    }

    pub fn arguments(&self) -> &IndexMap<String, Value> {
        &self.arguments
    }

    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    pub fn field_name(&self) -> &str {
        self.field_name.as_str()
    }

    pub fn selection_set(&self) -> &SelectionSet<'fragset> {
        &self.selection_set
    }
}
