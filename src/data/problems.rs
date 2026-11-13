//! CodeLab problems. Visitors write JavaScript; expected outputs are produced
//! by the reference solutions below — real Rust, compiled into the page.

use serde_json::{json, Value};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cmp {
    Exact,
    /// A two-element array whose order does not matter.
    PairAnyOrder,
}

pub struct Problem {
    pub id: &'static str,
    pub n: u32,
    pub title: &'static str,
    pub diff: &'static str,
    pub tags: &'static [&'static str],
    pub params: &'static [&'static str],
    pub func: &'static str,
    pub ret: &'static str,
    /// Trusted, hand-written HTML.
    pub desc: &'static str,
    pub cons: &'static [&'static str],
    pub follow: Option<&'static str>,
    /// Explanations for the first three examples.
    pub notes: [Option<&'static str>; 3],
    pub visible: usize,
    pub cmp: Cmp,
    pub solution: &'static str,
    pub tests: fn() -> Vec<Vec<Value>>,
    pub reference: fn(&[Value]) -> Value,
}

impl Problem {
    pub fn starter(&self) -> String {
        format!("function {}({}) {{\n    // your code here\n    \n}}\n", self.func, self.params.join(", "))
    }

    pub fn judge(&self, got: &Value, want: &Value) -> bool {
        match self.cmp {
            Cmp::Exact => got == want,
            Cmp::PairAnyOrder => {
                let norm = |v: &Value| -> Option<Vec<i64>> {
                    let mut a: Vec<i64> = v.as_array()?.iter().map(|x| x.as_i64()).collect::<Option<_>>()?;
                    a.sort();
                    Some(a)
                };
                matches!((norm(got), norm(want)), (Some(a), Some(b)) if a.len() == 2 && a == b)
            }
        }
    }
}

fn ints(v: &Value) -> Vec<i64> {
    v.as_array().map(|a| a.iter().filter_map(|x| x.as_i64()).collect()).unwrap_or_default()
}

// ───────────────────────── reference solutions ─────────────────────────

fn two_sum(a: &[Value]) -> Value {
    let (nums, t) = (ints(&a[0]), a[1].as_i64().unwrap_or(0));
    let mut seen = std::collections::HashMap::new();
    for (i, &x) in nums.iter().enumerate() {
        if let Some(&j) = seen.get(&(t - x)) {
            return json!([j, i]);
        }
        seen.insert(x, i);
    }
    Value::Null
}

fn is_valid(a: &[Value]) -> Value {
    let s = a[0].as_str().unwrap_or("");
    let mut st = Vec::new();
    for c in s.chars() {
        match c {
            ')' | ']' | '}' => {
                let want = match c { ')' => '(', ']' => '[', _ => '{' };
                if st.pop() != Some(want) {
                    return json!(false);
                }
            }
            _ => st.push(c),
        }
    }
    json!(st.is_empty())
}

fn security_grade(a: &[Value]) -> Value {
    let w = |s: &str| match s { "critical" => 30, "high" => 15, "medium" => 8, "low" => 3, _ => 0 };
    let sum: i64 = a[0].as_array().map(|v| v.iter().map(|x| w(x.as_str().unwrap_or(""))).sum()).unwrap_or(0);
    let score = (100 - sum).max(0);
    let g = match score { 90.. => "A", 80..=89 => "B", 65..=79 => "C", 50..=64 => "D", _ => "F" };
    json!([score, g])
}

fn max_sub_array(a: &[Value]) -> Value {
    let v = ints(&a[0]);
    let (mut best, mut cur) = (v[0], v[0]);
    for &x in &v[1..] {
        cur = x.max(cur + x);
        best = best.max(cur);
    }
    json!(best)
}

fn longest_substring(a: &[Value]) -> Value {
    let s: Vec<char> = a[0].as_str().unwrap_or("").chars().collect();
    let mut last = std::collections::HashMap::new();
    let (mut start, mut best) = (0usize, 0usize);
    for (i, c) in s.iter().enumerate() {
        if let Some(&j) = last.get(c) {
            if j >= start {
                start = j + 1;
            }
        }
        last.insert(*c, i);
        best = best.max(i + 1 - start);
    }
    json!(best)
}

fn rate_limiter(a: &[Value]) -> Value {
    let (ts, lim, w) = (ints(&a[0]), a[1].as_i64().unwrap_or(1) as usize, a[2].as_i64().unwrap_or(1));
    let mut q: std::collections::VecDeque<i64> = Default::default();
    let out: Vec<bool> = ts
        .iter()
        .map(|&t| {
            while q.front().is_some_and(|&f| f <= t - w) {
                q.pop_front();
            }
            if q.len() < lim {
                q.push_back(t);
                true
            } else {
                false
            }
        })
        .collect();
    json!(out)
}

fn merge(a: &[Value]) -> Value {
    let mut iv: Vec<(i64, i64)> = a[0]
        .as_array()
        .map(|v| v.iter().map(|p| { let p = ints(p); (p[0], p[1]) }).collect())
        .unwrap_or_default();
    iv.sort();
    let mut out: Vec<(i64, i64)> = Vec::new();
    for (s, e) in iv {
        match out.last_mut() {
            Some(l) if s <= l.1 => l.1 = l.1.max(e),
            _ => out.push((s, e)),
        }
    }
    json!(out.iter().map(|(s, e)| vec![*s, *e]).collect::<Vec<_>>())
}

// ───────────────────────── the catalogue ─────────────────────────

pub const PROBLEMS: &[Problem] = &[
    Problem {
        id: "two-sum", n: 1, title: "Two Sum", diff: "Easy", tags: &["Array", "Hash Table"], params: &["nums", "target"],
        func: "twoSum", ret: "array of 2 indices",
        desc: "<p>Given an array of integers <code>nums</code> and an integer <code>target</code>, return the <em>indices</em> of the two numbers that add up to <code>target</code>.</p><p>Each input has exactly one solution, and you may not use the same element twice. You can return the answer in any order.</p>",
        cons: &["2 ≤ nums.length ≤ 10⁴", "−10⁹ ≤ nums[i], target ≤ 10⁹", "Exactly one valid answer exists"],
        follow: Some("Can you do better than O(n²)?"),
        notes: [Some("Because nums[0] + nums[1] == 9, we return [0, 1]."), None, None],
        visible: 3, cmp: Cmp::PairAnyOrder,
        tests: || vec![
            vec![json!([2, 7, 11, 15]), json!(9)], vec![json!([3, 2, 4]), json!(6)], vec![json!([3, 3]), json!(6)],
            vec![json!([1, 5, 9, 13, 2]), json!(11)], vec![json!([-3, 4, 3, 90]), json!(0)], vec![json!([0, 4, 3, 0]), json!(0)],
            vec![json!((1..=2000).collect::<Vec<i64>>()), json!(3999)],
        ],
        reference: two_sum,
        solution: "function twoSum(nums, target) {\n    const seen = new Map();\n    for (let i = 0; i < nums.length; i++) {\n        if (seen.has(target - nums[i])) return [seen.get(target - nums[i]), i];\n        seen.set(nums[i], i);\n    }\n}\n",
    },
    Problem {
        id: "valid-parentheses", n: 2, title: "Valid Parentheses", diff: "Easy", tags: &["String", "Stack"], params: &["s"],
        func: "isValid", ret: "boolean",
        desc: "<p>Given a string <code>s</code> containing just the characters <code>(</code>, <code>)</code>, <code>{</code>, <code>}</code>, <code>[</code> and <code>]</code>, determine if the input string is valid.</p><p>Open brackets must be closed by the same type of bracket, in the correct order, and every close bracket needs a matching open bracket.</p>",
        cons: &["1 ≤ s.length ≤ 10⁴", "s consists of brackets only: ()[]{}"],
        follow: None, notes: [None, None, Some("The ( is closed by ], which is the wrong type.")],
        visible: 3, cmp: Cmp::Exact,
        tests: || ["()", "()[]{}", "(]", "([)]", "{[]}", "((", "]", "{[()()]}"].iter().map(|s| vec![json!(s)]).collect(),
        reference: is_valid,
        solution: "function isValid(s) {\n    const pairs = { ')': '(', ']': '[', '}': '{' };\n    const stack = [];\n    for (const ch of s) {\n        if (pairs[ch]) {\n            if (stack.pop() !== pairs[ch]) return false;\n        } else stack.push(ch);\n    }\n    return stack.length === 0;\n}\n",
    },
    Problem {
        id: "security-grade", n: 3, title: "Security Grade", diff: "Easy", tags: &["Cypheron", "Simulation"], params: &["findings"],
        func: "securityGrade", ret: "[score, grade]",
        desc: "<p>Cypheron grades every scan. Given the list of finding severities from one scan, compute the risk score and letter grade.</p><p>Start at <code>100</code> and subtract <code>30</code> per <code>\"critical\"</code>, <code>15</code> per <code>\"high\"</code>, <code>8</code> per <code>\"medium\"</code> and <code>3</code> per <code>\"low\"</code>. <code>\"info\"</code> costs nothing. The score never goes below <code>0</code>.</p><p>Grade: <code>A</code> if score ≥ 90, <code>B</code> ≥ 80, <code>C</code> ≥ 65, <code>D</code> ≥ 50, otherwise <code>F</code>. Return <code>[score, grade]</code>.</p>",
        cons: &["0 ≤ findings.length ≤ 100", "Each finding is one of \"critical\", \"high\", \"medium\", \"low\", \"info\""],
        follow: None, notes: [Some("100 − 3 = 97, which is an A."), Some("100 − 15 − 8 − 3 = 74, which is a C."), None],
        visible: 3, cmp: Cmp::Exact,
        tests: || vec![
            vec![json!(["low", "info"])], vec![json!(["high", "medium", "low"])], vec![json!(["critical", "critical", "high", "high"])],
            vec![json!([])], vec![json!(["medium", "medium"])], vec![json!(["critical", "high", "high", "high", "medium"])],
            vec![json!(["high", "low", "low"])], vec![json!(["critical", "critical", "critical", "critical"])],
        ],
        reference: security_grade,
        solution: "function securityGrade(findings) {\n    const weight = { critical: 30, high: 15, medium: 8, low: 3, info: 0 };\n    const score = Math.max(0, 100 - findings.reduce((s, f) => s + weight[f], 0));\n    const grade = score >= 90 ? 'A' : score >= 80 ? 'B' : score >= 65 ? 'C' : score >= 50 ? 'D' : 'F';\n    return [score, grade];\n}\n",
    },
    Problem {
        id: "maximum-subarray", n: 4, title: "Maximum Subarray", diff: "Medium", tags: &["Array", "Dynamic Programming"], params: &["nums"],
        func: "maxSubArray", ret: "number",
        desc: "<p>Given an integer array <code>nums</code>, find the contiguous subarray with the largest sum and return <em>its sum</em>.</p>",
        cons: &["1 ≤ nums.length ≤ 10⁵", "−10⁴ ≤ nums[i] ≤ 10⁴"],
        follow: Some("Kadane's algorithm does it in one pass with O(1) memory."),
        notes: [Some("The subarray [4, −1, 2, 1] has the largest sum, 6."), None, None],
        visible: 3, cmp: Cmp::Exact,
        tests: || vec![
            vec![json!([-2, 1, -3, 4, -1, 2, 1, -5, 4])], vec![json!([1])], vec![json!([5, 4, -1, 7, 8])],
            vec![json!([-3, -1, -2])], vec![json!([0, 0, 0])], vec![json!([2, -1, 2, -1, 2])],
            vec![json!((0..3000).map(|i| ((i * 7919) % 201) - 100).collect::<Vec<i64>>())],
        ],
        reference: max_sub_array,
        solution: "function maxSubArray(nums) {\n    let best = nums[0], cur = nums[0];\n    for (let i = 1; i < nums.length; i++) {\n        cur = Math.max(nums[i], cur + nums[i]);\n        best = Math.max(best, cur);\n    }\n    return best;\n}\n",
    },
    Problem {
        id: "longest-substring", n: 5, title: "Longest Substring Without Repeating Characters", diff: "Medium", tags: &["String", "Sliding Window", "Hash Table"], params: &["s"],
        func: "lengthOfLongestSubstring", ret: "number",
        desc: "<p>Given a string <code>s</code>, find the length of the longest substring without repeating characters.</p>",
        cons: &["0 ≤ s.length ≤ 5 × 10⁴", "s consists of letters, digits, symbols and spaces"],
        follow: None,
        notes: [Some("The answer is \"abc\", with length 3."), Some("The answer is \"b\", with length 1."), Some("The answer is \"wke\". \"pwke\" is a subsequence, not a substring.")],
        visible: 3, cmp: Cmp::Exact,
        tests: || ["abcabcbb", "bbbbb", "pwwkew", "", " ", "dvdf", "abba", "tmmzuxt"].iter().map(|s| vec![json!(s)]).collect(),
        reference: longest_substring,
        solution: "function lengthOfLongestSubstring(s) {\n    const last = new Map();\n    let start = 0, best = 0;\n    for (let i = 0; i < s.length; i++) {\n        if (last.has(s[i]) && last.get(s[i]) >= start) start = last.get(s[i]) + 1;\n        last.set(s[i], i);\n        best = Math.max(best, i - start + 1);\n    }\n    return best;\n}\n",
    },
    Problem {
        id: "rate-limiter", n: 6, title: "Sliding Window Rate Limiter", diff: "Medium", tags: &["System Design", "Sliding Window", "Queue"], params: &["timestamps", "limit", "window"],
        func: "rateLimiter", ret: "boolean[]",
        desc: "<p>You are putting a rate limiter in front of a FastAPI gateway. Requests arrive with non-decreasing integer <code>timestamps</code> in seconds.</p><p>A request at time <code>t</code> is <strong>allowed</strong> if fewer than <code>limit</code> requests were allowed inside the window <code>(t − window, t]</code>. Rejected requests do not count toward the window.</p><p>Return an array of booleans, one per request.</p>",
        cons: &["1 ≤ timestamps.length ≤ 10⁵", "1 ≤ limit, window ≤ 10⁴", "timestamps is sorted in non-decreasing order"],
        follow: Some("Each request should be handled in amortised O(1)."),
        notes: [Some("The third request at t=1 hits the limit of 2. At t=3 the window (1, 3] is empty again."), None, Some("Three requests fit, the fourth is rejected.")],
        visible: 3, cmp: Cmp::Exact,
        tests: || vec![
            vec![json!([1, 1, 1, 2, 3, 4]), json!(2), json!(2)], vec![json!([1, 2, 3, 4, 5]), json!(1), json!(1)],
            vec![json!([0, 0, 0, 0]), json!(3), json!(10)], vec![json!([1, 3, 5, 7, 9, 11]), json!(2), json!(5)],
            vec![json!([5, 5, 6, 6, 7, 7, 8]), json!(2), json!(2)], vec![json!([10]), json!(1), json!(1)],
            vec![json!((0..2000).map(|i| i >> 2).collect::<Vec<i64>>()), json!(3), json!(1)],
        ],
        reference: rate_limiter,
        solution: "function rateLimiter(timestamps, limit, window) {\n    const q = [];\n    let head = 0;\n    return timestamps.map(t => {\n        while (head < q.length && q[head] <= t - window) head++;\n        if (q.length - head < limit) { q.push(t); return true; }\n        return false;\n    });\n}\n",
    },
    Problem {
        id: "merge-intervals", n: 7, title: "Merge Intervals", diff: "Medium", tags: &["Array", "Sorting"], params: &["intervals"],
        func: "merge", ret: "intervals",
        desc: "<p>Given an array of <code>intervals</code> where <code>intervals[i] = [start, end]</code>, merge all overlapping intervals and return the non-overlapping intervals that cover all of the input, sorted by start.</p>",
        cons: &["1 ≤ intervals.length ≤ 10⁴", "0 ≤ start ≤ end ≤ 10⁴"],
        follow: None,
        notes: [Some("[1,3] and [2,6] overlap, so they merge into [1,6]."), Some("[1,4] and [4,5] touch, so they merge."), None],
        visible: 3, cmp: Cmp::Exact,
        tests: || vec![
            vec![json!([[1, 3], [2, 6], [8, 10], [15, 18]])], vec![json!([[1, 4], [4, 5]])], vec![json!([[1, 4], [0, 4]])],
            vec![json!([[1, 4], [2, 3]])], vec![json!([[5, 6], [1, 2], [2, 4]])], vec![json!([[2, 3], [4, 5], [6, 7], [8, 9], [1, 10]])],
            vec![json!([[1, 1]])],
        ],
        reference: merge,
        solution: "function merge(intervals) {\n    const a = intervals.map(x => [...x]).sort((x, y) => x[0] - y[0]);\n    const out = [];\n    for (const x of a) {\n        if (out.length && x[0] <= out[out.length - 1][1]) {\n            out[out.length - 1][1] = Math.max(out[out.length - 1][1], x[1]);\n        } else out.push(x);\n    }\n    return out;\n}\n",
    },
];

pub fn by_id(id: &str) -> Option<&'static Problem> {
    PROBLEMS.iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn references_agree_with_known_answers() {
        let p = by_id("maximum-subarray").unwrap();
        assert_eq!((p.reference)(&(p.tests)()[0]), json!(6));
        let p = by_id("rate-limiter").unwrap();
        assert_eq!((p.reference)(&(p.tests)()[0]), json!([true, true, false, false, true, true]));
        let p = by_id("security-grade").unwrap();
        assert_eq!((p.reference)(&(p.tests)()[1]), json!([74, "C"]));
    }
}
