//! Tests for [`crate::ast::UnionTypeExtension`] and
//! [`crate::ast::UnionTypeExtensionSyntax`].

use crate::ast::UnionTypeExtension;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `UnionTypeExtension` stores name and
/// members, and `append_source` slices the correct
/// source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Union-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_type_extension_source_slice() {
    let source =
        "extend union SearchResult = Photo";
    let ute = UnionTypeExtension {
        span: make_byte_span(0, 33),
        name: make_name("SearchResult", 13, 25),
        directives: vec![],
        members: vec![make_name("Photo", 28, 33)],
        syntax: None,
    };
    assert_eq!(ute.name.value, "SearchResult");
    assert_eq!(ute.members.len(), 1);
    assert_eq!(ute.members[0].value, "Photo");

    let mut sink = String::new();
    ute.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
