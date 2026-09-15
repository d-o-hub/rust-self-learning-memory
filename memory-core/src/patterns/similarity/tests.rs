use super::*;

/// Reference: naive full-matrix Levenshtein over char slices.
/// Independent of the optimized single-row DP, used for cross-checking.
fn reference_levenshtein(a: &[char], b: &[char]) -> usize {
    reference_levenshtein_slice(a, b)
}

/// Reference: naive full-matrix Levenshtein over arbitrary element slices.
fn reference_levenshtein_slice<T: PartialEq>(a: &[T], b: &[T]) -> usize {
    let (rows, cols) = (a.len() + 1, b.len() + 1);
    let mut matrix = vec![vec![0usize; cols]; rows];
    for (i, row) in matrix.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..rows {
        for j in 1..cols {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }
    matrix[a.len()][b.len()]
}

#[test]
fn test_sequence_similarity() {
    let seq1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let seq2 = vec!["a".to_string(), "b".to_string(), "c".to_string()];

    assert_eq!(sequence_similarity(&seq1, &seq2), 1.0);

    let seq3 = vec!["a".to_string(), "b".to_string(), "d".to_string()];
    let sim = sequence_similarity(&seq1, &seq3);
    // 2 out of 3 match
    assert!(sim > 0.6 && sim < 0.7);
}

#[test]
fn test_string_similarity() {
    assert_eq!(string_similarity("hello", "hello"), 1.0);
    assert_eq!(string_similarity("", ""), 1.0);
    assert_eq!(string_similarity("abc", ""), 0.0);

    // "hello" vs "hallo" - one character different
    let sim = string_similarity("hello", "hallo");
    assert!(sim > 0.7 && sim < 0.9);
}

#[test]
fn test_context_similarity() {
    let ctx1 = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        tags: vec!["async".to_string(), "http".to_string()],
        ..Default::default()
    };

    let ctx2 = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        tags: vec!["async".to_string(), "rest".to_string()],
        ..Default::default()
    };

    let similarity = context_similarity(&ctx1, &ctx2);

    // Same domain, same language, some tag overlap
    assert!(similarity > 0.7);
}

/// Reference: true Jaccard counts via deduplicated sets.
/// Independent of the linear-scan fast path, used for cross-checking.
fn reference_jaccard_counts(tags1: &[String], tags2: &[String]) -> (usize, usize) {
    let set1: std::collections::HashSet<&str> = tags1.iter().map(String::as_str).collect();
    let set2: std::collections::HashSet<&str> = tags2.iter().map(String::as_str).collect();
    let common = set1.intersection(&set2).count();
    let union_size = set1.union(&set2).count();
    (common, union_size)
}

fn str_vec(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn test_tag_jaccard_counts_match_reference() {
    // Cases include empty sides, exact matches, disjoint sets, duplicates
    // (both sides), asymmetric sizes in both directions, inputs straddling
    // the 16-element fast-path threshold, and multibyte tags.
    let big: Vec<String> = (0..20).map(|i| format!("tag{i:02}")).collect();
    let big_overlap: Vec<String> = (10..30).map(|i| format!("tag{i:02}")).collect();
    let cases: Vec<(Vec<String>, Vec<String>)> = vec![
        (vec![], vec![]),
        (vec![], str_vec(&["a"])),
        (str_vec(&["a"]), vec![]),
        (str_vec(&["a"]), str_vec(&["a"])),
        (str_vec(&["a", "b"]), str_vec(&["b", "c"])),
        (str_vec(&["x", "y"]), str_vec(&["a", "b", "c"])),
        // Duplicates collapse to one vote per side (true Jaccard).
        (str_vec(&["a", "a", "b"]), str_vec(&["a", "c"])),
        (str_vec(&["a"]), str_vec(&["a", "a", "a"])),
        (str_vec(&["a", "a"]), str_vec(&["a", "a"])),
        // Asymmetric: small vs large in both directions.
        (str_vec(&["tag05"]), big.clone()),
        (big.clone(), str_vec(&["tag25"])),
        (big.clone(), big_overlap.clone()),
        // Multibyte tags compare by value, not by byte length.
        (str_vec(&["café", "naïve"]), str_vec(&["café", "plain"])),
    ];

    for (t1, t2) in &cases {
        assert_eq!(
            calculate_tag_jaccard_counts(t1, t2),
            reference_jaccard_counts(t1, t2),
            "mismatch for {t1:?} vs {t2:?}"
        );
        // Both paths must agree with each other via the reference;
        // also assert symmetry explicitly.
        assert_eq!(
            calculate_tag_jaccard_counts(t2, t1),
            reference_jaccard_counts(t1, t2),
            "asymmetric result for {t1:?} vs {t2:?}"
        );
    }
}

#[test]
fn test_tag_jaccard_duplicate_tags_stay_within_unit_range() {
    // Regression pin: the previous implementation counted duplicate
    // occurrences against a deduplicated union and could exceed 1.0.
    let dupes = str_vec(&["a", "a", "a"]);
    let single = str_vec(&["a"]);
    let (common, union_size) = calculate_tag_jaccard_counts(&dupes, &single);

    assert_eq!((common, union_size), (1, 1));
    assert!(common <= union_size);
}

#[test]
fn test_edit_distance_matches_reference() {
    // Cases include empty sides, exact matches, substitutions, insertions,
    // deletions, asymmetry, and multibyte elements.
    let cases: &[(&[&str], &[&str])] = &[
        (&[], &[]),
        (&[], &["a"]),
        (&["a"], &[]),
        (&["a"], &["a"]),
        (&["a", "b", "c"], &["a", "b", "c"]),
        (&["a", "b", "c"], &["a", "b", "d"]),
        (&["x", "y"], &["a", "b", "c"]),
        (&["tool_a", "tool_b", "tool_c"], &["tool_a", "tool_b"]),
        (&["café"], &["cafe"]), // multibyte elements
    ];

    for (s1, s2) in cases {
        let v1: Vec<String> = s1.iter().map(|s| (*s).to_string()).collect();
        let v2: Vec<String> = s2.iter().map(|s| (*s).to_string()).collect();
        let expected = reference_levenshtein_slice(s1, s2);
        assert_eq!(
            edit_distance(&v1, &v2),
            expected,
            "distance({s1:?}, {s2:?})"
        );
        // Levenshtein is symmetric; also exercises both arg orderings.
        assert_eq!(
            edit_distance(&v2, &v1),
            expected,
            "distance({s2:?}, {s1:?})"
        );
    }
}

#[test]
fn test_char_edit_distance_streamed_matches_reference() {
    // Includes empty sides, classic cases, and multibyte strings where byte
    // length and char count disagree.
    let cases: &[(&str, &str)] = &[
        ("", ""),
        ("", "hello"),
        ("hello", ""),
        ("hello", "hello"),
        ("hello", "hallo"),
        ("kitten", "sitting"),
        ("goodbye", "hi"),
        ("héllo", "hello"),
        ("ééé", "xyzw"),
        ("x", "yyyy"),
        ("yyyy", "x"),
    ];

    for (s1, s2) in cases {
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let expected = reference_levenshtein(&s1_chars, &s2_chars);
        let (distance, len2) = char_edit_distance_streamed(&s1_chars, s2);
        assert_eq!(distance, expected, "distance({s1:?}, {s2:?})");
        assert_eq!(len2, s2_chars.len(), "len2({s1:?}, {s2:?})");
    }
}

#[test]
fn test_string_similarity_longer_first_and_unicode() {
    // First argument strictly longer in characters -> exercises the
    // short/long selection `else` branch (s1 longer than s2).
    assert_eq!(string_similarity("goodbye", "hi"), 0.0);

    // One substitution; max length 5 chars.
    assert_eq!(string_similarity("héllo", "hello"), 0.8);

    // No overlap, multibyte; char-count selection keeps the DP row minimal.
    assert_eq!(string_similarity("ééé", "xyzw"), 0.0);

    // Symmetry: swapping arguments must not change the result.
    for (a, b) in [("goodbye", "hi"), ("héllo", "hello"), ("ééé", "xyzw")] {
        assert_eq!(string_similarity(a, b), string_similarity(b, a));
    }
}

#[test]
fn test_char_edit_distance_streamed_empty() {
    let s1: Vec<char> = vec![];
    let (dist, len2) = char_edit_distance_streamed(&s1, "hello");
    assert_eq!(dist, 5);
    assert_eq!(len2, 5);
}

#[test]
fn test_edit_distance_empty() {
    let seq1: Vec<String> = vec![];
    let seq2: Vec<String> = vec![];
    assert_eq!(edit_distance(&seq1, &seq2), 0);

    let seq3 = vec!["a".to_string()];
    assert_eq!(edit_distance(&seq1, &seq3), 1);
}
