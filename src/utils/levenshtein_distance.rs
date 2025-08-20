use std::cmp::min;

pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    let mut dp = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 0..=a_len { dp[i][0] = i; }
    for j in 0..=b_len { dp[0][j] = j; }

    for (i, a_char) in a_chars.iter().enumerate() {
        for (j, b_char) in b_chars.iter().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            dp[i + 1][j + 1] = min(
                dp[i][j + 1] + 1,
                min(
                    dp[i + 1][j] + 1,
                    dp[i][j] + cost
                )
            );
        }
    }

    dp[a_len][b_len]
}