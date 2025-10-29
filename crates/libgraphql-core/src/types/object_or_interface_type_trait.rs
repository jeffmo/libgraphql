use crate::DirectiveAnnotation;
use crate::loc;
use crate::schema::Schema;
use crate::types::DeprecationState;
use crate::types::Field;
use crate::types::InterfaceType;
use indexmap::IndexMap;

pub(super) trait ObjectOrInterfaceTypeTrait {
    fn def_location(&self) -> &loc::SourceLocation;
    fn description(&self) -> Option<&str>;
    fn deprecation_state(&self) -> DeprecationState<'_>;
    fn directives(&self) -> &Vec<DirectiveAnnotation>;
    fn fields(&self) -> &IndexMap<String, Field>;
    fn interfaces<'schema>(&self, schema: &'schema Schema) -> Vec<&'schema InterfaceType>;
    fn interface_names(&self) -> Vec<&str>;
    fn name(&self) -> &str;
}
