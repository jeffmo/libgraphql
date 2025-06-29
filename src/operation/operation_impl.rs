use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::Operation;
use crate::operation::OperationBuilder;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::ObjectType;
use inherent::inherent;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub(super) struct OperationImpl<
    'schema,
    'fragset,
    TAst,
    TError,
    TOperation: Operation<'schema, 'fragset, TAst, TError, TOperation, TBuilder>,
    TBuilder: OperationBuilder<'schema, 'fragset, TAst, TError, TOperation>,
> {
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) def_location: Option<loc::FilePosition>,
    pub(super) name: Option<String>,
    pub(super) schema: &'schema Schema,
    pub(super) selection_set: SelectionSet<'fragset>,
    pub(super) variables: BTreeMap<String, Variable>,
    pub(super) phantom_ast: PhantomData<TAst>,
    pub(super) phantom_error: PhantomData<TError>,
    pub(super) phantom_op: PhantomData<TOperation>,
    pub(super) phantom_builder: PhantomData<TBuilder>,
}

#[inherent]
impl<
    'schema,
    'fragset,
    TAst,
    TError,
    TOperation: Operation<'schema, 'fragset, TAst, TError, TOperation, TBuilder>,
    TBuilder: OperationBuilder<'schema, 'fragset, TAst, TError, TOperation>,
> Operation<'schema, 'fragset, TAst, TError, TOperation, TBuilder> for OperationImpl<
    'schema,
    'fragset,
    TAst,
    TError,
    TOperation,
    TBuilder,
> {
    /// Convenience wrapper around [`TBuilder::new()`].
    pub fn builder(schema: &'schema Schema) -> Result<TBuilder, TError> {
        TBuilder::new(schema)
    }
    /// Access the [`DirectiveAnnotation`]s defined on this [`OperationImpl`].
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        &self.directives
    }

    /// Convenience wrapper around [`MutationBuilder::from_ast()`].
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: TAst,
    ) -> Result<TOperation, TError> {
        TBuilder::from_ast(schema, file_path, def)
    }

    /// Access the name of this [`OperationImpl`] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Access the [`SelectionSet`] defined for this [`OperationImpl`].
    pub fn selection_set(&self) -> &SelectionSet<'fragset> {
        &self.selection_set
    }

    /// Access the [`GraphQLType`] that defines the operation represented by this [`OperationImpl`].
    fn operation_type(&self) -> &ObjectType {
        panic!(
            "This method should be implemented specifically for each \
            operation type"
        )
    }

    /// Access the [`Variable`]s defined on this [`OperationImpl`].
    pub fn variables(&self) -> &BTreeMap<String, Variable> {
        &self.variables
    }
}
