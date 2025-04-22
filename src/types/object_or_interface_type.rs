use crate::DirectiveAnnotation;
use crate::loc;
use crate::Schema;
use crate::types::Field;
use crate::types::InterfaceType;
use std::collections::BTreeMap;

pub(super) trait ObjectOrInterfaceType {
    fn def_location(&self) -> &loc::SchemaDefLocation;
    fn directives(&self) -> &Vec<DirectiveAnnotation>;
    fn fields(&self) -> &BTreeMap<String, Field>;
    fn interfaces<'schema>(&self, schema: &'schema Schema) -> Vec<&'schema InterfaceType>;
    fn name(&self) -> &str;
}
