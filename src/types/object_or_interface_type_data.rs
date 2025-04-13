use crate::loc;
use crate::Schema;
use crate::types::DirectiveAnnotation;
use crate::types::Field;
use crate::types::InterfaceType;
use crate::types::NamedGraphQLTypeRef;
use crate::types::ObjectOrInterfaceType;
use inherent::inherent;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ObjectOrInterfaceTypeData {
    pub(super) def_location: loc::SchemaDefLocation,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) fields: BTreeMap<String, Field>,
    pub(super) interfaces: Vec<NamedGraphQLTypeRef>,
    pub(super) name: String,
}

#[inherent]
impl ObjectOrInterfaceType for ObjectOrInterfaceTypeData {
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
    }

    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    pub fn fields(&self) -> &BTreeMap<String, Field> {
        &self.fields
    }

    pub fn interfaces<'schema>(
        &self,
        schema: &'schema Schema,
    ) -> Vec<&'schema InterfaceType> {
        self.interfaces
            .iter()
            .map(|iface_ref| {
                iface_ref.deref(schema).unwrap().unwrap_interface()
            })
            .collect()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
