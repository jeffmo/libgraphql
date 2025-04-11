use crate::operation::OperationBuilder;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::Schema;
use crate::types::DirectiveAnnotation;
use crate::types::GraphQLType;
use std::collections::BTreeMap;
use std::path::Path;

// Implements the set of things
pub(super) trait Operation<
    'schema,
    TAst,
    TError,
    TOperation: Operation<'schema, TAst, TError, TOperation, TBuilder>,
    TBuilder: OperationBuilder<'schema, TAst, TError, TOperation>,
> where Self: Sized {
    fn annotations(&self) -> &Vec<DirectiveAnnotation>;
    fn builder(schema: &'schema Schema) -> Result<TBuilder, TError>;
    fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: TAst,
    ) -> Result<TOperation, TError>;
    fn graphql_type(&self) -> &GraphQLType;
    fn name(&self) -> Option<&str>;
    fn selection_set(&self) -> &SelectionSet<'schema>;
    fn variables(&self) -> &BTreeMap<String, Variable>;
}
