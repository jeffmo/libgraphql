use crate::loc;
use crate::operation::Operation;
use crate::operation::OperationBuilder;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::Schema;
use crate::types::DirectiveAnnotation;
use crate::types::ObjectType;
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
    /// Access the [DirectiveAnnotation]s defined on this [OperationImpl].
    pub fn annotations(&self) -> &Vec<DirectiveAnnotation> {
        &self.annotations
    }

    /// Convenience wrapper around [TBuilder::new()].
    pub fn builder(schema: &'schema Schema) -> Result<TBuilder, TError> {
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

    /// Access the name of this [OperationImpl] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Access the [SelectionSet] defined for this [OperationImpl].
    pub fn selection_set(&self) -> &SelectionSet<'schema> {
        &self.selection_set
    }

    /// Access the [GraphQLType] that defines the operation represented by this [OperationImpl].
    fn operation_type(&self) -> &ObjectType {
        panic!(
            "This method should be implemented specifically for each \
            operation type"
        )
    }

    /// Access the [Variable]s defined on this [OperationImpl].
    pub fn variables(&self) -> &BTreeMap<String, Variable> {
        &self.variables
    }
}
