use crate::DirectiveAnnotation;
use crate::loc;
use crate::schema::Schema;
use crate::types::DeprecationState;
use crate::types::Field;
use crate::types::InterfaceType;
use crate::types::NamedGraphQLTypeRef;
use crate::types::ObjectOrInterfaceTypeTrait;
use indexmap::IndexMap;
use inherent::inherent;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(super) struct ObjectOrInterfaceTypeData {
    pub(super) def_location: loc::SourceLocation,
    pub(super) description: Option<String>,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) fields: IndexMap<String, Field>,
    pub(super) interfaces: Vec<NamedGraphQLTypeRef>,
    pub(super) name: String,
}

#[inherent]
impl ObjectOrInterfaceTypeTrait for ObjectOrInterfaceTypeData {
    pub fn def_location(&self) -> &loc::SourceLocation {
        &self.def_location
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn deprecation_state(&self) -> DeprecationState<'_> {
        (&self.directives).into()
    }

    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    pub fn fields(&self) -> &IndexMap<String, Field> {
        &self.fields
    }

    pub fn implements_interface<'schema>(
        &self,
        schema: &'schema Schema,
        interface: &'schema InterfaceType,
    ) -> bool {
        self.interfaces
            .iter()
            .any(|iface_ref| {
                if iface_ref.name() == interface.name() {
                    true
                } else {
                    iface_ref.deref(schema)
                        .expect("type is present in schema")
                        .as_interface()
                        .expect("type is an interface type")
                        .implements_interface(schema, interface)
                }
            })
    }

    pub fn interfaces<'schema>(
        &self,
        schema: &'schema Schema,
    ) -> Vec<&'schema InterfaceType> {
        self.interfaces
            .iter()
            .map(|iface_ref| {
                iface_ref.deref(schema)
                    .expect("type is present in schema")
                    .as_interface()
                    .expect("type is an interface type")
            })
            .collect()
    }

    pub fn interface_names(&self) -> Vec<&str> {
        self.interfaces
            .iter()
            .map(|iface_ref| iface_ref.name())
            .collect()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
