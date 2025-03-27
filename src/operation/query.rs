use crate::ast;
use crate::loc;
use crate::operation::QueryBuilder;
use crate::operation::QueryBuildError;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::DirectiveAnnotation;
use std::collections::BTreeMap;
use std::path::Path;

type Result<T> = std::result::Result<T, QueryBuildError>;

/// TODO
#[derive(Debug)]
pub struct Query<'schema> {
    pub(super) query_annotations: Vec<DirectiveAnnotation>,
    pub(super) name: Option<String>,
    pub(super) schema: &'schema Schema,
    pub(super) selection_set: SelectionSet<'schema>,
    pub(super) def_location: Option<loc::FilePosition>,
    pub(super) variables: BTreeMap<String, Variable>,
}
impl<'schema> Query<'schema> {
    /// Convenience wrapper around [QueryBuilder::new()].
    pub fn builder(schema: &'schema Schema) -> QueryBuilder<'schema> {
        QueryBuilder::new(schema)
    }

    /// Convenience wrapper around [QueryBuilder::from_ast()].
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Query,
    ) -> Result<Query<'schema>> {
        QueryBuilder::from_ast(schema, file_path, def)
    }
}
