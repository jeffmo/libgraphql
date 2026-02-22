//! Tests for [`crate::ast::UnionTypeDefinition`] and
//! [`crate::ast::UnionTypeDefinitionSyntax`].

use crate::ast::UnionTypeDefinition;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `UnionTypeDefinition` stores name and member
/// types, and `append_source` slices the correct source
/// range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Unions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_type_definition_source_slice() {
    let source =
        "union SearchResult = User | Post";
    let utd = UnionTypeDefinition {
        span: make_byte_span(0, 32),
        description: None,
        name: make_name("SearchResult", 6, 18),
        directives: vec![],
        members: vec![
            make_name("User", 21, 25),
            make_name("Post", 28, 32),
        ],
        syntax: None,
    };
    assert_eq!(utd.name.value, "SearchResult");
    assert_eq!(utd.members.len(), 2);
    assert_eq!(utd.members[0].value, "User");
    assert_eq!(utd.members[1].value, "Post");

    let mut sink = String::new();
    utd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
