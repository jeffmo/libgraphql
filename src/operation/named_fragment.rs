use crate::ast;
use crate::operation::FragmentSet;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::schema::Schema;
use std::marker::PhantomData;
use std::path::Path;
use thiserror::Error;

type Result<T> = std::result::Result<T, NamedFragmentBuildError>;

/// TODO
#[derive(Clone, Debug, PartialEq)]
pub struct NamedFragment<'schema> {
    schema: &'schema PhantomData<Schema>,
}
impl<'schema> NamedFragment<'schema> {
    pub fn from_ast(
        _schema: &'schema Schema,
        _file_path: &Path,
        _def: ast::operation::FragmentDefinition,
    ) -> Result<NamedFragment<'schema>> {
        todo!()
    }
}
impl<'schema> DerefByName for NamedFragment<'schema> {
    type Source = FragmentSet<'schema>;

    fn deref_name<'a>(
        source: &'a Self::Source,
        name: &str,
    ) -> std::result::Result<&'a NamedFragment<'schema>, DerefByNameError> {
        source.lookup_fragment(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string()),
        )
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum NamedFragmentBuildError {
}

pub type NamedFragmentRef<'schema> = NamedRef<
    FragmentSet<'schema>,
    NamedFragment<'schema>,
>;
