//! Tests for [`crate::ast::DirectiveLocation`] and
//! [`crate::ast::DirectiveLocationKind`].

use crate::ast::DirectiveLocation;
use crate::ast::DirectiveLocationKind;
use crate::ast::tests::ast_test_utils::make_byte_span;

/// Verify `DirectiveLocation` stores its kind and
/// produces the correct source slice.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#DirectiveLocations
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_location_source_slice() {
    let source = "FIELD_DEFINITION";
    let dl = DirectiveLocation {
        kind: DirectiveLocationKind::FieldDefinition,
        span: make_byte_span(0, 16),
        syntax: None,
    };
    assert_eq!(
        dl.kind,
        DirectiveLocationKind::FieldDefinition,
    );

    let mut sink = String::new();
    dl.append_source(&mut sink, Some(source));
    assert_eq!(sink, "FIELD_DEFINITION");
}

/// Verify `DirectiveLocation` works for an executable
/// location kind.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#ExecutableDirectiveLocation
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_location_executable_kind() {
    let source = "QUERY";
    let dl = DirectiveLocation {
        kind: DirectiveLocationKind::Query,
        span: make_byte_span(0, 5),
        syntax: None,
    };
    assert_eq!(dl.kind, DirectiveLocationKind::Query);

    let mut sink = String::new();
    dl.append_source(&mut sink, Some(source));
    assert_eq!(sink, "QUERY");
}
