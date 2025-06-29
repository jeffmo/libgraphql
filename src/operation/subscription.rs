use crate::ast;
use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::OperationTrait;
use crate::operation::OperationImpl;
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
type TOperationImpl<'schema, 'fragset> = OperationImpl<
    'schema,
    'fragset,
    ast::operation::Subscription,
    SubscriptionBuildError,
    Subscription<'schema, 'fragset>,
    SubscriptionBuilder<'schema, 'fragset>,
>;

#[derive(Clone, Debug, PartialEq)]
pub struct Subscription<'schema, 'fragset: 'schema>(pub(super) TOperationImpl<'schema, 'fragset>);
#[inherent]
impl<'schema, 'fragset: 'schema> OperationTrait<
    'schema,
    'fragset,
    ast::operation::Subscription,
    SubscriptionBuildError,
    Self,
    SubscriptionBuilder<'schema, 'fragset>,
> for Subscription<'schema, 'fragset> {
    /// Convenience wrapper around [`SubscriptionBuilder::new()`].
    pub fn builder(schema: &'schema Schema) -> Result<SubscriptionBuilder<'schema, 'fragset>> {
        OperationImpl::builder(schema)
    }

    /// The list of [`DirectiveAnnotation`]s applied to this [`Subscription`].
    pub fn directives(&self) -> &Vec<DirectiveAnnotation> {
        self.0.directives()
    }

    /// The [`DefLocation`](loc::FilePosition) indicating where this
    /// [`Subscription`] was defined.
    pub fn def_location(&self) -> Option<&loc::FilePosition> {
        self.0.def_location.as_ref()
    }

    /// Convenience wrapper around [`SubscriptionBuilder::from_ast()`].
    pub fn from_ast(
        schema: &'schema Schema,
        file_path: &Path,
        def: ast::operation::Subscription,
    ) -> Result<Subscription<'schema, 'fragset>> {
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
    pub fn selection_set(&self) -> &SelectionSet<'fragset> {
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
