use crate::names::TypeName;

/// Finds type names in `candidates` that are within a reasonable
/// edit distance of `name`. Returns at most 3 suggestions,
/// sorted by distance (best first).
///
/// The `max_distance` threshold is adaptive: shorter names
/// require closer matches to avoid nonsensical suggestions.
pub(crate) fn find_similar_names<'a>(
    name: &str,
    candidates: impl Iterator<Item = &'a TypeName>,
    max_distance: usize,
) -> Vec<&'a TypeName> {
    let mut scored: Vec<(usize, &'a TypeName)> = candidates
        .filter_map(|candidate| {
            let dist =
                levenshtein_distance(name, candidate.as_str());
            if dist > 0 && dist <= max_distance {
                Some((dist, candidate))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|(d1, n1), (d2, n2)| {
        d1.cmp(d2).then_with(|| n1.cmp(n2))
    });
    scored
        .into_iter()
        .take(3)
        .map(|(_, name)| name)
        .collect()
}

/// Computes the Levenshtein edit distance between two strings.
///
/// Uses the classic dynamic-programming algorithm with O(min(a,
/// b)) space via a single-row buffer.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let a_len = a_chars.len();
    let b_len = b_chars.len();

    // Ensure `b` is the shorter side for space efficiency.
    if a_len < b_len {
        return levenshtein_distance(b, a);
    }

    let mut prev_row: Vec<usize> =
        (0..=b_len).collect();
    let mut curr_row: Vec<usize> =
        vec![0; b_len + 1];

    for i in 1..=a_len {
        curr_row[0] = i;
        for j in 1..=b_len {
            let cost =
                if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
            curr_row[j] = (prev_row[j] + 1)
                .min(curr_row[j - 1] + 1)
                .min(prev_row[j - 1] + cost);
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

#[cfg(test)]
mod tests;
