use crate::ast;
use crate::Schema;
use std::path::Path;
use thiserror::Error;

type Result<T> = std::result::Result<T, SubscriptionBuildError>;

/// TODO
#[derive(Debug)]
pub struct Subscription<'schema> {
    schema: &'schema Schema,
}
impl<'schema> Subscription<'schema> {
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Subscription,
    ) -> Result<Subscription<'schema>> {
        todo!()
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum SubscriptionBuildError {
}
