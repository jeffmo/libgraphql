use crate::located::Located;
use crate::names::TypeName;
use crate::span::Span;

// Verifies Located wraps a value with a span.
// Written by Claude Code, reviewed by a human.
#[test]
fn located_construction() {
    let located = Located {
        value: TypeName::new("Node"),
        span: Span::builtin(),
    };
    assert_eq!(located.value.as_str(), "Node");
    assert_eq!(located.span, Span::builtin());
}

// Verifies AsRef<T> returns the inner value, enabling ergonomic
// pass-through to methods expecting &T.
// Written by Claude Code, reviewed by a human.
#[test]
fn located_as_ref() {
    let located = Located {
        value: TypeName::new("User"),
        span: Span::builtin(),
    };
    let name_ref: &TypeName = located.as_ref();
    assert_eq!(name_ref.as_str(), "User");
}

// Verifies Located<T> can be cloned independently.
// Written by Claude Code, reviewed by a human.
#[test]
fn located_clone() {
    let located = Located {
        value: TypeName::new("Query"),
        span: Span::builtin(),
    };
    let cloned = located.clone();
    assert_eq!(cloned.value.as_str(), "Query");
}
