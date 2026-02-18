//! Tests for [`crate::ast::OperationKind`].

use crate::ast::OperationKind;

/// Verify that `OperationKind` has exactly the three
/// expected variants: Query, Mutation, Subscription.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Operations
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn operation_kind_variants() {
    let q = OperationKind::Query;
    let m = OperationKind::Mutation;
    let s = OperationKind::Subscription;

    assert_eq!(q, OperationKind::Query);
    assert_eq!(m, OperationKind::Mutation);
    assert_eq!(s, OperationKind::Subscription);

    // Verify they are distinct
    assert_ne!(q, m);
    assert_ne!(q, s);
    assert_ne!(m, s);
}
