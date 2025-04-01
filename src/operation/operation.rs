use crate::operation::OperationBuilder;
use crate::Schema;
use std::path::Path;

// Implements the set of things
pub(super) trait Operation<
    'schema,
    TAst,
    TError,
    TOperation: Operation<'schema, TAst, TError, TOperation, TBuilder>,
    TBuilder: OperationBuilder<'schema, TAst, TError, TOperation>,
> where Self: Sized {
    fn builder(schema: &'schema Schema) -> TBuilder;

    fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: TAst,
    ) -> Result<TOperation, TError>;
}
