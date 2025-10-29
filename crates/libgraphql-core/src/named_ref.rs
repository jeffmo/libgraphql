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
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct NamedRef<
    TSource,
    TRefLocation,
    TResource: DerefByName<Source=TSource, RefLocation=TRefLocation>,
> {
    name: String,
    phantom: PhantomData<TResource>,
    ref_location: TRefLocation,
}
impl<
    TSource,
    TRefLocation,
    TResource: DerefByName<Source=TSource, RefLocation=TRefLocation>,
> NamedRef<TSource, TRefLocation, TResource> {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn new(
        name: impl AsRef<str>,
        ref_location: TRefLocation,
    ) -> NamedRef<TSource, TRefLocation, TResource> {
        NamedRef {
            name: name.as_ref().to_string(),
            ref_location,
            phantom: PhantomData,
        }
    }
}
impl<
    TSource,
    TRefLocation,
    TResource: DerefByName<Source=TSource, RefLocation=TRefLocation>,
> NamedRef<TSource, TRefLocation, TResource> {
    pub fn ref_location(&self) -> &TRefLocation {
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
    type RefLocation;

    fn deref_name<'a>(
        source: &'a Self::Source,
        name: &str,
    ) -> Result<&'a Self, DerefByNameError> where Self: Sized;

    fn named_ref(name: &str, ref_location: Self::RefLocation) -> NamedRef<
        Self::Source,
        Self::RefLocation,
        Self,
    > {
        NamedRef::<Self::Source, Self::RefLocation, Self>::new(
            name,
            ref_location,
        )
    }
}

#[derive(Clone, Debug)]
pub enum DerefByNameError {
    DanglingReference(String),
}
