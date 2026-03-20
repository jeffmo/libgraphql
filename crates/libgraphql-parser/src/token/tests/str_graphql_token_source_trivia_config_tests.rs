//! Tests for `StrGraphQLTokenSourceConfig` trivia flag behavior.
//!
//! Each flag (`retain_comments`, `retain_commas`, `retain_whitespace`)
//! controls whether the corresponding trivia type is captured on
//! emitted tokens. These tests verify that each flag works
//! independently and that the default config captures all trivia.
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLTriviaToken;
use crate::token::StrGraphQLTokenSource;
use crate::token::StrGraphQLTokenSourceConfig;

struct TriviaCounts {
    comments: usize,
    commas: usize,
    whitespace: usize,
}

/// Helper: count trivia by variant across all tokens.
fn count_trivia_by_kind(
    tokens: &[crate::token::GraphQLToken<'_>],
) -> TriviaCounts {
    let mut counts = TriviaCounts {
        comments: 0,
        commas: 0,
        whitespace: 0,
    };
    for token in tokens {
        for trivia in &token.preceding_trivia {
            match trivia {
                GraphQLTriviaToken::Comment { .. } => counts.comments += 1,
                GraphQLTriviaToken::Comma { .. } => counts.commas += 1,
                GraphQLTriviaToken::Whitespace { .. } => {
                    counts.whitespace += 1;
                },
            }
        }
    }
    counts
}

// =========================================================================
// Default config (all trivia retained)
// =========================================================================

/// Verifies that the default config retains all three trivia types:
/// whitespace, comments, and commas.
///
/// Source `"# comment\na, b"` produces:
///   on `a`: [Comment(" comment"), WS("\n")]
///   on `b`: [Comma, WS(" ")]
///   → 1 comment, 1 comma, 2 whitespace
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn default_config_retains_all_trivia() {
    let source = "# comment\na, b";
    let tokens: Vec<_> = StrGraphQLTokenSource::new(source).collect();
    let counts = count_trivia_by_kind(&tokens);

    assert_eq!(counts.comments, 1, "expected 1 comment");
    assert_eq!(counts.commas, 1, "expected 1 comma");
    assert_eq!(counts.whitespace, 2, "expected 2 whitespace runs");
}

// =========================================================================
// no_trivia() config (all trivia discarded)
// =========================================================================

/// Verifies that `no_trivia()` discards all trivia types.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn no_trivia_config_discards_all_trivia() {
    let source = "# comment\na, b";
    let config = StrGraphQLTokenSourceConfig::no_trivia();
    let tokens: Vec<_> =
        StrGraphQLTokenSource::with_config(source, config).collect();
    let counts = count_trivia_by_kind(&tokens);

    assert_eq!(counts.comments, 0, "no_trivia should discard comments");
    assert_eq!(counts.commas, 0, "no_trivia should discard commas");
    assert_eq!(counts.whitespace, 0, "no_trivia should discard whitespace");
}

// =========================================================================
// Individual flag tests: retain_comments
// =========================================================================

/// Verifies that `retain_comments: false` discards comments but keeps
/// commas and whitespace.
///
/// Source `"# comment\na, b"` with comments off:
///   on `a`: [WS("\n")]
///   on `b`: [Comma, WS(" ")]
///   → 0 comments, 1 comma, 2 whitespace
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn retain_comments_false_discards_comments() {
    let source = "# comment\na, b";
    let config = StrGraphQLTokenSourceConfig {
        retain_comments: false,
        ..Default::default()
    };
    let tokens: Vec<_> =
        StrGraphQLTokenSource::with_config(source, config).collect();
    let counts = count_trivia_by_kind(&tokens);

    assert_eq!(counts.comments, 0, "comments should be discarded");
    assert_eq!(counts.commas, 1, "commas should still be retained");
    assert_eq!(counts.whitespace, 2, "whitespace should still be retained");
}

/// Verifies that `retain_comments: true` preserves comments when
/// all other trivia is disabled.
///
/// Source `"# hello\nfield"` with only comments on:
///   on `field`: [Comment(" hello")]
///   → 1 comment, 0 commas, 0 whitespace
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn retain_comments_true_preserves_comments() {
    let source = "# hello\nfield";
    let config = StrGraphQLTokenSourceConfig {
        retain_comments: true,
        retain_commas: false,
        retain_whitespace: false,
    };
    let tokens: Vec<_> =
        StrGraphQLTokenSource::with_config(source, config).collect();
    let counts = count_trivia_by_kind(&tokens);

    assert_eq!(counts.comments, 1, "should have exactly 1 comment trivia");
    assert_eq!(counts.commas, 0);
    assert_eq!(counts.whitespace, 0);
    assert!(matches!(
        &tokens[0].preceding_trivia[0],
        GraphQLTriviaToken::Comment { value, .. } if value == " hello"
    ));
}

// =========================================================================
// Individual flag tests: retain_commas
// =========================================================================

/// Verifies that `retain_commas: false` discards commas but keeps
/// comments and whitespace.
///
/// Source `"# comment\na, b"` with commas off:
///   on `a`: [Comment(" comment"), WS("\n")]
///   on `b`: [WS(" ")]
///   → 1 comment, 0 commas, 2 whitespace
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn retain_commas_false_discards_commas() {
    let source = "# comment\na, b";
    let config = StrGraphQLTokenSourceConfig {
        retain_commas: false,
        ..Default::default()
    };
    let tokens: Vec<_> =
        StrGraphQLTokenSource::with_config(source, config).collect();
    let counts = count_trivia_by_kind(&tokens);

    assert_eq!(counts.comments, 1, "comments should still be retained");
    assert_eq!(counts.commas, 0, "commas should be discarded");
    assert_eq!(counts.whitespace, 2, "whitespace should still be retained");
}

/// Verifies that `retain_commas: true` preserves commas when all
/// other trivia is disabled.
///
/// Source `"a,b"` with only commas on:
///   on `b`: [Comma]
///   → 0 comments, 1 comma, 0 whitespace
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn retain_commas_true_preserves_commas() {
    let source = "a,b";
    let config = StrGraphQLTokenSourceConfig {
        retain_comments: false,
        retain_commas: true,
        retain_whitespace: false,
    };
    let tokens: Vec<_> =
        StrGraphQLTokenSource::with_config(source, config).collect();
    let counts = count_trivia_by_kind(&tokens);

    assert_eq!(counts.comments, 0);
    assert_eq!(counts.commas, 1, "should have exactly 1 comma trivia");
    assert_eq!(counts.whitespace, 0);
    assert!(matches!(
        &tokens[1].preceding_trivia[0],
        GraphQLTriviaToken::Comma { .. }
    ));
}

// =========================================================================
// Individual flag tests: retain_whitespace
// =========================================================================

/// Verifies that `retain_whitespace: false` discards whitespace but
/// keeps comments and commas.
///
/// Source `"# comment\na, b"` with whitespace off:
///   on `a`: [Comment(" comment")]
///   on `b`: [Comma]
///   → 1 comment, 1 comma, 0 whitespace
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn retain_whitespace_false_discards_whitespace() {
    let source = "# comment\na, b";
    let config = StrGraphQLTokenSourceConfig {
        retain_whitespace: false,
        ..Default::default()
    };
    let tokens: Vec<_> =
        StrGraphQLTokenSource::with_config(source, config).collect();
    let counts = count_trivia_by_kind(&tokens);

    assert_eq!(counts.comments, 1, "comments should still be retained");
    assert_eq!(counts.commas, 1, "commas should still be retained");
    assert_eq!(counts.whitespace, 0, "whitespace should be discarded");
}

/// Verifies that `retain_whitespace: true` preserves whitespace runs
/// with correct values and ordering.
///
/// Source `"  a  b"` with only whitespace on:
///   on `a`: [WS("  ")]
///   on `b`: [WS("  ")]
///   → 0 comments, 0 commas, 2 whitespace
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn retain_whitespace_true_preserves_whitespace_runs() {
    let source = "  a  b";
    let config = StrGraphQLTokenSourceConfig {
        retain_comments: false,
        retain_commas: false,
        retain_whitespace: true,
    };
    let tokens: Vec<_> =
        StrGraphQLTokenSource::with_config(source, config).collect();

    // Token 0: Name("a") with Whitespace("  ") trivia
    assert_eq!(tokens[0].preceding_trivia.len(), 1);
    assert!(matches!(
        &tokens[0].preceding_trivia[0],
        GraphQLTriviaToken::Whitespace { value, .. } if value == "  "
    ));

    // Token 1: Name("b") with Whitespace("  ") trivia
    assert_eq!(tokens[1].preceding_trivia.len(), 1);
    assert!(matches!(
        &tokens[1].preceding_trivia[0],
        GraphQLTriviaToken::Whitespace { value, .. } if value == "  "
    ));
}

// =========================================================================
// Whitespace content tests
// =========================================================================

/// Verifies that whitespace trivia captures newlines correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn whitespace_captures_newlines() {
    let source = "a\n\nb";
    let config = StrGraphQLTokenSourceConfig {
        retain_comments: false,
        retain_commas: false,
        retain_whitespace: true,
    };
    let tokens: Vec<_> =
        StrGraphQLTokenSource::with_config(source, config).collect();

    // Token 1: Name("b") with Whitespace("\n\n") trivia
    assert_eq!(tokens[1].preceding_trivia.len(), 1);
    assert!(matches!(
        &tokens[1].preceding_trivia[0],
        GraphQLTriviaToken::Whitespace { value, .. } if value == "\n\n"
    ));
}

/// Verifies that whitespace trivia captures tabs.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn whitespace_captures_tabs() {
    let source = "\t\ta";
    let config = StrGraphQLTokenSourceConfig {
        retain_comments: false,
        retain_commas: false,
        retain_whitespace: true,
    };
    let tokens: Vec<_> =
        StrGraphQLTokenSource::with_config(source, config).collect();

    assert_eq!(tokens[0].preceding_trivia.len(), 1);
    assert!(matches!(
        &tokens[0].preceding_trivia[0],
        GraphQLTriviaToken::Whitespace { value, .. } if value == "\t\t"
    ));
}

/// Verifies that whitespace trivia captures mixed whitespace
/// (spaces, tabs, newlines) as a single run.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn whitespace_captures_mixed_run() {
    let source = " \t\n  a";
    let config = StrGraphQLTokenSourceConfig {
        retain_comments: false,
        retain_commas: false,
        retain_whitespace: true,
    };
    let tokens: Vec<_> =
        StrGraphQLTokenSource::with_config(source, config).collect();

    assert_eq!(tokens[0].preceding_trivia.len(), 1);
    assert!(matches!(
        &tokens[0].preceding_trivia[0],
        GraphQLTriviaToken::Whitespace { value, .. } if value == " \t\n  "
    ));
}

// =========================================================================
// Interleaving tests
// =========================================================================

/// Verifies correct trivia ordering when whitespace, comments, and
/// commas are interleaved.
///
/// Source: `"  # comment\n  a,  b"`
/// Expected trivia on `a`: [WS("  "), Comment(" comment"), WS("\n  ")]
/// Expected trivia on `b`: [Comma, WS("  ")]
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn interleaved_trivia_ordering() {
    let source = "  # comment\n  a,  b";
    let tokens: Vec<_> = StrGraphQLTokenSource::new(source).collect();

    // Token 0: Name("a")
    // Leading trivia: WS("  "), Comment(" comment"), WS("\n  ")
    assert_eq!(
        tokens[0].preceding_trivia.len(), 3,
        "trivia on 'a': {:?}", tokens[0].preceding_trivia,
    );
    assert!(matches!(
        &tokens[0].preceding_trivia[0],
        GraphQLTriviaToken::Whitespace { value, .. } if value == "  "
    ));
    assert!(matches!(
        &tokens[0].preceding_trivia[1],
        GraphQLTriviaToken::Comment { value, .. } if value == " comment"
    ));
    assert!(matches!(
        &tokens[0].preceding_trivia[2],
        GraphQLTriviaToken::Whitespace { value, .. } if value == "\n  "
    ));

    // Token 1: Name("b")
    // Leading trivia: Comma, WS("  ")
    assert_eq!(
        tokens[1].preceding_trivia.len(), 2,
        "trivia on 'b': {:?}", tokens[1].preceding_trivia,
    );
    assert!(matches!(
        &tokens[1].preceding_trivia[0],
        GraphQLTriviaToken::Comma { .. }
    ));
    assert!(matches!(
        &tokens[1].preceding_trivia[1],
        GraphQLTriviaToken::Whitespace { value, .. } if value == "  "
    ));
}
