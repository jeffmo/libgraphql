use crate::located::Located;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::UnionType;

// Verifies UnionType accessors and member tracking.
// https://spec.graphql.org/September2025/#sec-Unions
// Written by Claude Code, reviewed by a human.
#[test]
fn union_type_accessors() {
    let union_type = UnionType {
        description: Some("A search result".to_string()),
        directives: vec![],
        members: vec![
            Located { value: TypeName::new("User"), span: Span::builtin() },
            Located { value: TypeName::new("Post"), span: Span::builtin() },
        ],
        name: TypeName::new("SearchResult"),
        span: Span::builtin(),
    };
    assert_eq!(union_type.name().as_str(), "SearchResult");
    assert_eq!(union_type.description(), Some("A search result"));
    assert_eq!(union_type.members().len(), 2);
    assert_eq!(union_type.members()[0].value.as_str(), "User");
    assert_eq!(union_type.members()[1].value.as_str(), "Post");
}
