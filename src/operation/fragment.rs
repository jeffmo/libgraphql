use crate::ast;
use crate::types::NamedGraphQLTypeRef;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::FragmentRegistry;
use crate::operation::SelectionSet;
use crate::named_ref::DerefByName;
use crate::named_ref::DerefByNameError;
use crate::named_ref::NamedRef;
use crate::schema::Schema;
use std::path::Path;
use thiserror::Error;

type Result<T> = std::result::Result<T, FragmentBuildError>;

/// TODO
#[derive(Clone, Debug, PartialEq)]
pub struct Fragment<'schema> {
    pub(super) def_location: loc::FilePosition,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) name: String,
    pub(super) schema: &'schema Schema,
    pub(super) selection_set: SelectionSet<'schema>,
    pub(super) type_condition: NamedGraphQLTypeRef,
}

impl<'schema> Fragment<'schema> {
    // TODO: Move this to a `FragmentBuilder` to be more consistent with
    //       other builder-focused API patterns.
    pub fn from_ast(
        _schema: &'schema Schema,
        _file_path: &Path,
        _def: ast::operation::FragmentDefinition,
    ) -> Result<Fragment<'schema>> {
        todo!()
    }
}

impl<'schema> DerefByName for Fragment<'schema> {
    type Source = FragmentRegistry<'schema>;

    fn deref_name<'a>(
        source: &'a Self::Source,
        name: &str,
    ) -> std::result::Result<&'a Fragment<'schema>, DerefByNameError> {
        source.fragments.get(name).ok_or_else(
            || DerefByNameError::DanglingReference(name.to_string()),
        )
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum FragmentBuildError {
}

pub type FragmentRef<'schema> = NamedRef<
    FragmentRegistry<'schema>,
    Fragment<'schema>,
>;
