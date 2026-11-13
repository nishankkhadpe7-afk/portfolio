//! A tiny fuzzy matcher for the command palette and the app launcher.
//! Scores subsequence matches, rewarding consecutive runs and word starts —
//! so `cyp` finds "Cypheron" and `ws` finds "Web Security".

pub fn score(query: &str, target: &str) -> Option<i32> {
    let q: Vec<char> = query.to_lowercase().chars().filter(|c| !c.is_whitespace()).collect();
    if q.is_empty() {
        return Some(0);
    }
    let t: Vec<char> = target.to_lowercase().chars().collect();
    let (mut qi, mut score, mut run) = (0usize, 0i32, 0i32);
    let mut prev: Option<char> = None;
    for (i, &c) in t.iter().enumerate() {
        if qi < q.len() && c == q[qi] {
            let word_start = i == 0 || matches!(prev, Some(' ' | '-' | '_' | '·' | '/' | '.'));
            run += 1;
            score += 10 + run * 6 + if word_start { 18 } else { 0 } - (i as i32 / 6);
            qi += 1;
        } else {
            run = 0;
        }
        prev = Some(c);
    }
    (qi == q.len()).then_some(score)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subsequence() {
        assert!(score("cyp", "Cypheron").is_some());
        assert!(score("xyz", "Cypheron").is_none());
        assert!(score("ws", "web security").unwrap() > score("ws", "towers").unwrap());
    }
}
