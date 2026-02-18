/// The kind of a GraphQL operation.
///
/// See
/// [Operations](https://spec.graphql.org/September2025/#sec-Language.Operations)
/// in the spec.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OperationKind {
    Mutation,
    Query,
    Subscription,
}
