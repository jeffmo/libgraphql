use crate::loc;
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
#[derive(Clone, Debug, PartialEq)]
pub struct NamedRef<
    TSource,
    TResource: DerefByName<Source=TSource>,
> {
    pub name: String,
    pub ref_location: loc::SchemaDefLocation,
    phantom: PhantomData<TResource>,
}
impl<TSource, TResource: DerefByName<Source=TSource>> NamedRef<TSource, TResource> {
    pub fn new(
        name: impl AsRef<str>,
        ref_location: loc::SchemaDefLocation,
    ) -> NamedRef<TSource, TResource> {
        NamedRef {
            name: name.as_ref().to_string(),
            ref_location,
            phantom: PhantomData,
        }
    }
}
impl<TSource, TResource: DerefByName<Source=TSource>> NamedRef<TSource, TResource> {
    pub fn get_ref_location(&self) -> &loc::SchemaDefLocation {
        &self.ref_location
    }

    pub fn deref<'a>(
        &self,
        source: &'a TSource,
    ) -> Result<&'a TResource, DerefByNameError> {
        TResource::deref_name(source, self.name.as_str())
    }
}

/// Implement this trait for any type that could be referenced by named. This
/// will enable usage of NamedRef<T> for that type.
pub trait DerefByName: Clone + core::fmt::Debug {
    type Source;

    fn deref_name<'a>(
        source: &'a Self::Source,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> where Self: Sized;

    fn named_ref(name: &str, location: loc::SchemaDefLocation) -> NamedRef<Self::Source, Self> {
        NamedRef::<Self::Source, Self>::new(name, location)
    }
}

#[derive(Clone, Debug)]
pub enum DerefByNameError {
    DanglingReference(String),
}
