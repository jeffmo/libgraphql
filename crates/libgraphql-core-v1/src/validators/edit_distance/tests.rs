use crate::names::TypeName;
use crate::validators::edit_distance::find_similar_names;
use crate::validators::edit_distance::levenshtein_distance;

// Verifies that two identical strings have an edit distance of 0.
// Written by Claude Code, reviewed by a human.
#[test]
fn exact_match_is_distance_zero() {
    assert_eq!(levenshtein_distance("String", "String"), 0);
    assert_eq!(levenshtein_distance("", ""), 0);
    assert_eq!(levenshtein_distance("a", "a"), 0);
}

// Verifies that a single-character substitution produces an
// edit distance of 1.
// Written by Claude Code, reviewed by a human.
#[test]
fn single_char_substitution_is_distance_one() {
    assert_eq!(levenshtein_distance("Strng", "Strng"), 0);
    assert_eq!(levenshtein_distance("String", "Strung"), 1);
    assert_eq!(levenshtein_distance("cat", "bat"), 1);
    assert_eq!(levenshtein_distance("abc", "adc"), 1);
}

// Verifies that insertions and deletions are counted correctly.
// Written by Claude Code, reviewed by a human.
#[test]
fn insertion_and_deletion() {
    // Deletion: "String" -> "Sting" (remove 'r')
    assert_eq!(levenshtein_distance("String", "Sting"), 1);
    // Insertion: "Sting" -> "String" (add 'r')
    assert_eq!(levenshtein_distance("Sting", "String"), 1);
    // Multiple operations
    assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    // Empty vs non-empty
    assert_eq!(levenshtein_distance("", "abc"), 3);
    assert_eq!(levenshtein_distance("abc", ""), 3);
}

// Verifies that find_similar_names returns the best matches
// sorted by edit distance, limited to at most 3 results.
// Written by Claude Code, reviewed by a human.
#[test]
fn find_similar_names_returns_best_matches() {
    let candidates = [
        TypeName::new("String"),
        TypeName::new("Int"),
        TypeName::new("Float"),
        TypeName::new("Boolean"),
        TypeName::new("Strong"),
    ];
    let results = find_similar_names(
        "Strng",
        candidates.iter(),
        /* max_distance = */ 3,
    );
    // "String" (distance 1) and "Strong" (distance 2) should
    // match; "Int", "Float", "Boolean" are too far.
    assert!(!results.is_empty());
    assert_eq!(
        results[0].as_str(), "String",
        "best match for 'Strng' should be 'String'",
    );
}

// Verifies that find_similar_names returns an empty vec when
// no candidates are within the max_distance threshold.
// Written by Claude Code, reviewed by a human.
#[test]
fn find_similar_names_returns_empty_for_very_different_names() {
    let candidates = [
        TypeName::new("String"),
        TypeName::new("Int"),
        TypeName::new("Float"),
    ];
    let results = find_similar_names(
        "CompletelyUnrelated",
        candidates.iter(),
        /* max_distance = */ 3,
    );
    assert!(
        results.is_empty(),
        "expected no suggestions for a very different name, \
        got: {results:?}",
    );
}

// Verifies that find_similar_names returns at most 3
// suggestions even when more candidates match.
// Written by Claude Code, reviewed by a human.
#[test]
fn find_similar_names_limits_to_three() {
    let candidates = [
        TypeName::new("Aa"),
        TypeName::new("Ab"),
        TypeName::new("Ac"),
        TypeName::new("Ad"),
        TypeName::new("Ae"),
    ];
    let results = find_similar_names(
        "Ax",
        candidates.iter(),
        /* max_distance = */ 3,
    );
    assert!(
        results.len() <= 3,
        "expected at most 3 suggestions, got: {}",
        results.len(),
    );
}

// Verifies that find_similar_names excludes exact matches
// (distance 0) since the caller is looking for *similar* but
// *different* names.
// Written by Claude Code, reviewed by a human.
#[test]
fn find_similar_names_excludes_exact_match() {
    let candidates = [
        TypeName::new("String"),
        TypeName::new("Strng"),
    ];
    let results = find_similar_names(
        "String",
        candidates.iter(),
        /* max_distance = */ 3,
    );
    // "String" itself (distance 0) should be excluded; only
    // "Strng" (distance 1) should appear.
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), "Strng");
}
