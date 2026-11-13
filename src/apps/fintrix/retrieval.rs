//! Hybrid retrieval, for real: Okapi BM25 over stop-worded tokens fused with
//! cosine similarity of character-trigram vectors (a dependency-free stand-in
//! for dense embeddings). The index is built once, lazily, on first query.

use crate::data::corpus::CORPUS;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

const STOP: &[&str] = &[
    "a", "an", "the", "of", "to", "in", "on", "for", "and", "or", "is", "are", "be", "can", "i", "my", "me", "do", "does",
    "how", "what", "who", "with", "by", "under", "per", "as", "at", "from", "it", "this", "that", "much",
];
const K1: f64 = 1.4;
const B: f64 = 0.75;
/// Fusion weight on BM25 (the rest goes to the trigram score).
pub const ALPHA: f64 = 0.55;

/// A deliberately tiny suffix-stripping stemmer: enough that "violating",
/// "violate" and "violation" meet at "violat".
fn stem(w: &str) -> String {
    for suf in ["ations", "ation", "ions", "ing", "ion", "ies", "ed", "es", "s", "e"] {
        if let Some(st) = w.strip_suffix(suf) {
            if st.len() >= 4 {
                return st.to_string();
            }
        }
    }
    w.to_string()
}

pub fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty() && !STOP.contains(w))
        .map(stem)
        .collect()
}

fn trigrams(s: &str) -> HashMap<String, f64> {
    let norm: String = s.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' }).collect();
    let t: Vec<char> = format!(" {norm} ").chars().collect();
    let mut m = HashMap::new();
    for w in t.windows(3) {
        *m.entry(w.iter().collect::<String>()).or_insert(0.0) += 1.0;
    }
    m
}

fn cosine(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    let dot: f64 = a.iter().filter_map(|(k, x)| b.get(k).map(|y| x * y)).sum();
    let na: f64 = a.values().map(|x| x * x).sum::<f64>().sqrt();
    let nb: f64 = b.values().map(|x| x * x).sum::<f64>().sqrt();
    if na * nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

struct Doc {
    len: usize,
    tf: HashMap<String, f64>,
    tri: HashMap<String, f64>,
}

struct Index {
    docs: Vec<Doc>,
    df: HashMap<String, f64>,
    avg: f64,
}

fn index() -> &'static Index {
    static IDX: OnceLock<Index> = OnceLock::new();
    IDX.get_or_init(|| {
        let docs: Vec<Doc> = CORPUS
            .iter()
            .map(|c| {
                let text = format!("{} {} {}", c.src, c.text, c.kw);
                let toks = tokenize(&text);
                let mut tf = HashMap::new();
                for t in &toks {
                    *tf.entry(t.clone()).or_insert(0.0) += 1.0;
                }
                Doc { len: toks.len(), tf, tri: trigrams(&text) }
            })
            .collect();
        let mut df = HashMap::new();
        for d in &docs {
            for w in d.tf.keys().collect::<HashSet<_>>() {
                *df.entry(w.clone()).or_insert(0.0) += 1.0;
            }
        }
        let avg = docs.iter().map(|d| d.len as f64).sum::<f64>() / docs.len() as f64;
        Index { docs, df, avg }
    })
}

#[derive(Clone, PartialEq, Debug)]
pub struct Hit {
    pub doc: usize,
    pub bm25: f64,
    pub dense: f64,
    pub hybrid: f64,
}

pub struct Ranking {
    pub hits: Vec<Hit>,
    pub terms: Vec<String>,
    /// IDF-weighted share of the query the top chunk actually covers.
    pub coverage: f64,
    pub grounded: bool,
}

/// Minimum IDF-weighted query coverage for an answer to count as grounded.
pub const MIN_COVERAGE: f64 = 0.42;

/// Rank every chunk for `q`. "Grounded" requires both a real lexical match
/// and a strong fused score — otherwise Fintrix declines to answer.
pub fn retrieve(q: &str) -> Ranking {
    let ix = index();
    let qt = tokenize(q);
    let qv = trigrams(q);
    let n = ix.docs.len() as f64;
    let mut hits: Vec<Hit> = ix
        .docs
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let bm25 = qt
                .iter()
                .filter_map(|w| {
                    let f = *d.tf.get(w)?;
                    let df = ix.df.get(w).copied().unwrap_or(0.0);
                    let idf = (1.0 + (n - df + 0.5) / (df + 0.5)).ln();
                    Some(idf * f * (K1 + 1.0) / (f + K1 * (1.0 - B + B * d.len as f64 / ix.avg)))
                })
                .sum();
            Hit { doc: i, bm25, dense: cosine(&qv, &d.tri), hybrid: 0.0 }
        })
        .collect();
    let mx = hits.iter().map(|h| h.bm25).fold(1e-9, f64::max);
    let dmx = hits.iter().map(|h| h.dense).fold(f64::MIN, f64::max);
    let dmn = hits.iter().map(|h| h.dense).fold(f64::MAX, f64::min);
    let span = if dmx - dmn > 0.0 { dmx - dmn } else { 1.0 };
    for h in &mut hits {
        h.hybrid = ALPHA * (h.bm25 / mx) + (1.0 - ALPHA) * ((h.dense - dmn) / span);
    }
    hits.sort_by(|a, b| b.hybrid.total_cmp(&a.hybrid));
    let idf = |w: &String| {
        let df = ix.df.get(w).copied().unwrap_or(0.0);
        (1.0 + (n - df + 0.5) / (df + 0.5)).ln()
    };
    let coverage = hits.first().map_or(0.0, |t| {
        let total: f64 = qt.iter().map(idf).sum();
        let got: f64 = qt.iter().filter(|w| ix.docs[t.doc].tf.contains_key(*w)).map(idf).sum();
        if total > 0.0 { got / total } else { 0.0 }
    });
    let grounded = hits.first().is_some_and(|t| t.bm25 > 0.6 && t.hybrid > 0.5) && coverage >= MIN_COVERAGE;
    Ranking { hits, terms: qt, coverage, grounded }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_lrs_and_declines_offtopic() {
        let r = retrieve("How much money can I send abroad in a year?");
        assert!(r.grounded);
        assert_eq!(r.hits[0].doc, 0);
        assert!(!retrieve("What is the capital of France?").grounded);
        for q in crate::data::corpus::SUGGESTIONS {
            let r = retrieve(q);
            println!("{q:50} top={} bm={:.2} h={:.2} cov={:.2} grounded={}", r.hits[0].doc, r.hits[0].bm25, r.hits[0].hybrid, r.coverage, r.grounded);
        }
        for q in ["what happens if I break FEMA rules", "documents needed to open a bank account", "can I buy US stocks", "best pizza in mumbai", "send 300000 dollars abroad"] {
            let r = retrieve(q);
            println!("{q:50} top={} bm={:.2} h={:.2} cov={:.2} grounded={}", r.hits[0].doc, r.hits[0].bm25, r.hits[0].hybrid, r.coverage, r.grounded);
        }
    }
}
