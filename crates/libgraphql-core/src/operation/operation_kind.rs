/// Similar to [`Operation`](crate::operation::Operation), except without the
/// corresponding metadata. Useful when representing a group or category of
/// [`Operation`](crate::operation::Operation)s.
#[derive(Clone, Debug, PartialEq)]
pub enum OperationKind {
    Mutation,
    Query,
    Subscription,
}
