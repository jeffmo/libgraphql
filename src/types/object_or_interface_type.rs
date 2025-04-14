use crate::loc;
use crate::Schema;
use crate::types::DirectiveAnnotation;
use crate::types::Field;
use crate::types::InterfaceType;
use std::collections::BTreeMap;

pub(super) trait ObjectOrInterfaceType<'schema> {
    fn def_location(&self) -> &loc::SchemaDefLocation;
    fn directives(&self) -> &Vec<DirectiveAnnotation>;
    fn fields(&self) -> &BTreeMap<String, Field>;
    fn interfaces(&self) -> Vec<&'schema InterfaceType>;
    fn name(&self) -> &str;
}
