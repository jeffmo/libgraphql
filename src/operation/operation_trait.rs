use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::OperationBuilder;
use crate::operation::SelectionSet;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::ObjectType;
use std::collections::BTreeMap;
use std::path::Path;

// Implements the set of things
pub(super) trait OperationTrait<
    'schema,
    'fragset,
    TAst,
    TError,
    TOperation: OperationTrait<'schema, 'fragset, TAst, TError, TOperation, TBuilder>,
    TBuilder: OperationBuilder<'schema, 'fragset, TAst, TError, TOperation>,
> where Self: Sized {
    fn builder(schema: &'schema Schema) -> Result<TBuilder, TError>;
    fn def_location(&self) -> Option<&loc::FilePosition>;
    fn directives(&self) -> &Vec<DirectiveAnnotation>;
    fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: TAst,
    ) -> Result<TOperation, TError>;
    fn operation_type(&self) -> &ObjectType;
    fn name(&self) -> Option<&str>;
    fn selection_set(&self) -> &SelectionSet<'fragset>;
    fn variables(&self) -> &BTreeMap<String, Variable>;
}
