use crate::loc;
use std::marker::PhantomData;

/// Represents a strongly-typed, `String`-named reference to a
/// "resource" (`TResource`) stored within some other data-store (`TSource`)
/// without holding an explicit reference to the data-store. De-referencing a
/// [NamedRef] is done via [NamedRef::deref()] by providing an explicit
/// reference to the `TSource`.
///
/// `TSource` types are bound to implement the `DerefByName` trait in order to
/// execute de-referencing operations for a `TResource` given its [`String`]
/// name.
///
/// As a more concrete example, [crate::types::ObjectType] stores a
/// `Vec<NamedRef<crate::Schema, crate::types::GraphQLType>>` as a way of
/// storing "pointers" to the [crate::types::InterfaceType]s implemented by that
/// [crate::types::ObjectType]. Storing [NamedRef] references to the
/// [crate::types::InterfaceType]s instead of direct, [std::ops::Deref]-based
/// references allows structures like [crate::schema::Schema] to store all of the
/// Schema's defined types without a need for self-references.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedRef<
    TSource,
    TResource: DerefByName<Source=TSource>,
> {
    pub name: String,
    pub def_location: loc::SchemaDefLocation,
    phantom: PhantomData<TResource>,
}
impl<TSource, TResource: DerefByName<Source=TSource>> NamedRef<TSource, TResource> {
    pub fn new(
        name: impl AsRef<str>,
        def_location: loc::SchemaDefLocation,
    ) -> NamedRef<TSource, TResource> {
        NamedRef {
            name: name.as_ref().to_string(),
            def_location,
            phantom: PhantomData,
        }
    }
}
impl<TSource, TResource: DerefByName<Source=TSource>> NamedRef<TSource, TResource> {
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        &self.def_location
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
