use crate::DirectiveAnnotation;
use crate::loc;
use crate::schema::Schema;
use crate::types::Field;
use crate::types::InterfaceType;
use std::collections::BTreeMap;

pub(super) trait ObjectOrInterfaceTypeTrait {
    fn def_location(&self) -> &loc::SchemaDefLocation;
    fn directives(&self) -> &Vec<DirectiveAnnotation>;
    fn fields(&self) -> &BTreeMap<String, Field>;
    fn interfaces<'schema>(&self, schema: &'schema Schema) -> Vec<&'schema InterfaceType>;
    fn interface_names(&self) -> Vec<&str>;
    fn name(&self) -> &str;
}
