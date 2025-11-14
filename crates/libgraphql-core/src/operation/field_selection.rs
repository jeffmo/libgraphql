use crate::types::Field;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::SelectionSet;
use crate::schema::Schema;
use crate::Value;
use indexmap::IndexMap;

#[derive(Clone, Debug, PartialEq)]
pub struct FieldSelection<'schema> {
    pub(super) alias: Option<String>,
    pub(super) arguments: IndexMap<String, Value>,
    pub(super) def_location: loc::SourceLocation,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) field: &'schema Field,
    pub(super) schema: &'schema Schema,
    pub(super) selection_set: Option<SelectionSet<'schema>>,
}
impl<'schema> FieldSelection<'schema> {
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    pub fn arguments(&self) -> &IndexMap<String, Value> {
        &self.arguments
    }

    pub fn def_location(&self) -> &loc::SourceLocation {
        &self.def_location
    }

    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    pub fn field(&self) -> &'schema Field {
        self.field
    }

    /**
     * If an alias was specified for this selection, return the alias.
     * Otherwise return the name of the field.
     */
    pub fn selected_name(&self) -> &str {
        self.alias().unwrap_or_else(|| self.field().name())
    }

    pub fn selection_set(&self) -> Option<&SelectionSet<'schema>> {
        self.selection_set.as_ref()
    }
}
