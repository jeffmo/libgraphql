use crate::loc;
use crate::operation::Operation;
use crate::operation::OperationBuilder;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::Schema;
use crate::types::DirectiveAnnotation;
use inherent::inherent;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::path::Path;

#[derive(Debug)]
pub(super) struct OperationImpl<
    'schema,
    TAst,
    TError,
    TOperation: Operation<'schema, TAst, TError, TOperation, TBuilder>,
    TBuilder: OperationBuilder<'schema, TAst, TError, TOperation>,
> {
    pub(super) annotations: Vec<DirectiveAnnotation>,
    pub(super) def_location: Option<loc::FilePosition>,
    pub(super) name: Option<String>,
    pub(super) schema: &'schema Schema,
    pub(super) selection_set: SelectionSet<'schema>,
    pub(super) variables: BTreeMap<String, Variable>,
    pub(super) phantom_ast: PhantomData<TAst>,
    pub(super) phantom_error: PhantomData<TError>,
    pub(super) phantom_op: PhantomData<TOperation>,
    pub(super) phantom_builder: PhantomData<TBuilder>,
}

#[inherent]
impl<
    'schema,
    TAst,
    TError,
    TOperation: Operation<'schema, TAst, TError, TOperation, TBuilder>,
    TBuilder: OperationBuilder<'schema, TAst, TError, TOperation>,
> Operation<'schema, TAst, TError, TOperation, TBuilder> for OperationImpl<
    'schema,
    TAst,
    TError,
    TOperation,
    TBuilder,
> {
    /// Convenience wrapper around [TBuilder::new()].
    pub fn builder(schema: &'schema Schema) -> TBuilder {
        TBuilder::new(schema)
    }

    /// Convenience wrapper around [MutationBuilder::from_ast()].
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: TAst,
    ) -> Result<TOperation, TError> {
        TBuilder::from_ast(schema, file_path, def)
    }
}
