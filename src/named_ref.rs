use crate::ast;
use crate::schema::Schema;
use std::marker::PhantomData;

/// Represents a reference to something by name.
///
/// For example, each field defined on
/// [SchemaType::Object](crate::types::SchemaType::Object) specifies a named
/// reference to some well-defined [SchemaType](crate::types::SchemaType).to
/// indicate the type for that field.
///
/// Similarly: When a directive is specified above a type definition in a
/// schema, the directive specified using a named reference to the definition
/// for that particular directive.
#[derive(Clone, Debug)]
pub struct NamedRef<T: DerefByName> {
    pub name: String,
    pub ref_location: ast::FileLocation,
    phantom: PhantomData<T>,
}
impl<T: DerefByName> NamedRef<T> {
    pub fn new(
        name: String,
        ref_location: ast::FileLocation,
    ) -> NamedRef<T> {
        NamedRef {
            name,
            ref_location,
            phantom: PhantomData,
        }
    }
}
impl<T: DerefByName> NamedRef<T> {
    pub fn deref<'a>(&self, schema: &'a Schema) -> &'a T {
        self.maybe_deref(schema).unwrap()
    }

    pub fn get_ref_location(&self) -> &ast::FileLocation {
        &self.ref_location
    }

    pub(crate) fn maybe_deref<'a>(&self, schema: &'a Schema) -> Result<&'a T, DerefByNameError> {
        T::deref_name(schema, self.name.as_str())
    }
}

/// Implement this trait for any type that could be referenced by named. This
/// will enable usage of NamedRef<T> for that type.
pub trait DerefByName: Clone + core::fmt::Debug {
    fn deref_name<'a>(
        schema: &'a Schema,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> where Self: Sized;
}

#[derive(Clone, Debug)]
pub enum DerefByNameError {
    DanglingReference
}
