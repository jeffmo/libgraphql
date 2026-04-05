use crate::types::DeprecationState;

// Verifies Active state is not deprecated.
// https://spec.graphql.org/September2025/#sec--deprecated
// Written by Claude Code, reviewed by a human.
#[test]
fn active_not_deprecated() {
    let state = DeprecationState::Active;
    assert!(!state.is_deprecated());
}

// Verifies Deprecated state without reason.
// Written by Claude Code, reviewed by a human.
#[test]
fn deprecated_without_reason() {
    let state = DeprecationState::Deprecated { reason: None };
    assert!(state.is_deprecated());
}
