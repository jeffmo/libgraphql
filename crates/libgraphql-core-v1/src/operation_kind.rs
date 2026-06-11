/// The kind of GraphQL operation.
///
/// See [Operations](https://spec.graphql.org/September2025/#sec-Language.Operations).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum OperationKind {
    Mutation,
    Query,
    Subscription,
}

impl From<libgraphql_parser::ast::OperationKind> for OperationKind {
    fn from(kind: libgraphql_parser::ast::OperationKind) -> Self {
        match kind {
            libgraphql_parser::ast::OperationKind::Mutation => Self::Mutation,
            libgraphql_parser::ast::OperationKind::Query => Self::Query,
            libgraphql_parser::ast::OperationKind::Subscription => Self::Subscription,
        }
    }
}

impl std::fmt::Display for OperationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Mutation => "mutation",
            Self::Query => "query",
            Self::Subscription => "subscription",
        })
    }
}
