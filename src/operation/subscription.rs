use crate::ast;
use crate::DirectiveAnnotation;
use crate::operation::Operation;
use crate::operation::OperationBuilder;
use crate::operation::OperationImpl;
use crate::operation::Selection;
use crate::operation::SelectionSet;
use crate::operation::SubscriptionBuilder;
use crate::operation::SubscriptionBuildError;
use crate::operation::Variable;
use crate::schema::Schema;
use crate::types::ObjectType;
use inherent::inherent;
use std::collections::BTreeMap;
use std::path::Path;

type Result<T> = std::result::Result<T, SubscriptionBuildError>;
type TOperationImpl<'schema> = OperationImpl<
    'schema,
    ast::operation::Subscription,
    SubscriptionBuildError,
    Subscription<'schema>,
    SubscriptionBuilder<'schema>,
>;

#[derive(Debug)]
pub struct Subscription<'schema>(pub(super) TOperationImpl<'schema>);
#[inherent]
impl<'schema> Operation<
    'schema,
    ast::operation::Subscription,
    SubscriptionBuildError,
    Self,
    SubscriptionBuilder<'schema>,
> for Subscription<'schema> {
    /// Access the [`DirectiveAnnotation`]s defined on this [`Subscription`].
    pub fn annotations(&self) -> &Vec<DirectiveAnnotation> {
        self.0.annotations()
    }

    /// Convenience wrapper around [`SubscriptionBuilder::new()`].
    pub fn builder(schema: &'schema Schema) -> Result<SubscriptionBuilder<'schema>> {
        OperationImpl::builder(schema)
    }

    /// Convenience wrapper around [`SubscriptionBuilder::from_ast()`].
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Subscription,
    ) -> Result<Subscription<'schema>> {
        OperationImpl::from_ast(schema, file_path, def)
    }

    /// Access the [`ObjectType`] that defines this [`Subscription`] operation.
    pub fn operation_type(&self) -> &ObjectType {
        self.0.schema.query_type()
    }

    /// Access the name of this [`Subscription`] (if one was specified).
    pub fn name(&self) -> Option<&str> {
        self.0.name()
    }

    /// Access the [`SelectionSet`] defined for this [`Subscription`].
    pub fn selection_set(&self) -> &SelectionSet<'schema> {
        self.0.selection_set()
    }

    /// Access the [`Variable`]s defined on this [`Subscription`].
    pub fn variables(&self) -> &BTreeMap<String, Variable> {
        self.0.variables()
    }
}
/*
impl<'schema> Subscription<'schema> {
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Subscription,
    ) -> Result<Subscription<'schema>> {
        todo!()
    }
}
*/
