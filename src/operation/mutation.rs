use crate::ast;
use crate::loc;
use crate::operation::MutationBuilder;
use crate::operation::MutationBuildError;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::DirectiveAnnotation;
use std::collections::BTreeMap;
use std::path::Path;

type Result<T> = std::result::Result<T, MutationBuildError>;

/// TODO
#[derive(Debug)]
pub struct Mutation<'schema> {
    pub(super) def_location: Option<loc::FilePosition>,
    pub(super) mutation_annotations: Vec<DirectiveAnnotation>,
    pub(super) name: Option<String>,
    pub(super) schema: &'schema Schema,
    pub(super) selection_set: SelectionSet<'schema>,
    pub(super) variables: BTreeMap<String, Variable>,
}
impl<'schema> Mutation<'schema> {
    pub fn builder(schema: &'schema Schema) -> MutationBuilder<'schema> {
        MutationBuilder::new(schema)
    }

    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Mutation,
    ) -> Result<Mutation<'schema>> {
        MutationBuilder::from_ast(schema, file_path, def)
    }
}
