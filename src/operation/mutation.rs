use crate::ast;
use crate::schema::Schema;
use std::path::Path;
use thiserror::Error;

type Result<T> = std::result::Result<T, MutationBuildError>;

/// TODO
#[derive(Debug)]
pub struct Mutation<'schema> {
    schema: &'schema Schema,
}
impl<'schema> Mutation<'schema> {
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Mutation,
    ) -> Result<Mutation<'schema>> {
        todo!()
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum MutationBuildError {
}
