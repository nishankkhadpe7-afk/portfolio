//! A small JavaScript syntax highlighter producing escaped HTML.

use crate::util::esc;

const KW: &[&str] = &[
    "const", "let", "var", "function", "return", "if", "else", "for", "while", "do", "of", "in", "new", "break", "continue",
    "switch", "case", "default", "class", "extends", "try", "catch", "finally", "throw", "typeof", "instanceof", "await",
    "async", "yield", "this", "delete", "void", "static", "get", "set",
];
const BUILTIN: &[&str] = &[
    "Math", "Map", "Set", "Array", "Object", "JSON", "console", "Number", "String", "Boolean", "Infinity", "NaN",
    "undefined", "null", "true", "false", "BigInt", "Symbol", "WeakMap", "Date",
];

pub fn js(src: &str) -> String {
    let c: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len() * 2);
    let mut i = 0;
    let span = |out: &mut String, cls: &str, s: &str| {
        out.push_str(&format!("<span class=\"{cls}\">{}</span>", esc(s)));
    };
    while i < c.len() {
        let ch = c[i];
        // comments
        if ch == '/' && c.get(i + 1) == Some(&'/') {
            let end = c[i..].iter().position(|&x| x == '\n').map(|p| i + p).unwrap_or(c.len());
            span(&mut out, "tk-c", &c[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        if ch == '/' && c.get(i + 1) == Some(&'*') {
            let mut end = i + 2;
            while end + 1 < c.len() && !(c[end] == '*' && c[end + 1] == '/') {
                end += 1;
            }
            let end = (end + 2).min(c.len());
            span(&mut out, "tk-c", &c[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        // strings
        if ch == '"' || ch == '\'' || ch == '`' {
            let mut end = i + 1;
            while end < c.len() && c[end] != ch {
                if c[end] == '\\' {
                    end += 1;
                }
                if c[end.min(c.len() - 1)] == '\n' && ch != '`' {
                    break;
                }
                end += 1;
            }
            let end = (end + 1).min(c.len());
            span(&mut out, "tk-s", &c[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        // numbers
        if ch.is_ascii_digit() {
            let mut end = i;
            while end < c.len() && (c[end].is_ascii_alphanumeric() || c[end] == '.' || c[end] == '_') {
                end += 1;
            }
            span(&mut out, "tk-n", &c[i..end].iter().collect::<String>());
            i = end;
            continue;
        }
        // identifiers
        if ch.is_alphabetic() || ch == '_' || ch == '$' {
            let mut end = i;
            while end < c.len() && (c[end].is_alphanumeric() || c[end] == '_' || c[end] == '$') {
                end += 1;
            }
            let word: String = c[i..end].iter().collect();
            let next = c[end..].iter().find(|x| !x.is_whitespace());
            if KW.contains(&word.as_str()) {
                span(&mut out, "tk-k", &word);
            } else if BUILTIN.contains(&word.as_str()) {
                span(&mut out, "tk-b", &word);
            } else if next == Some(&'(') {
                span(&mut out, "tk-f", &word);
            } else {
                out.push_str(&esc(&word));
            }
            i = end;
            continue;
        }
        out.push_str(&esc(&ch.to_string()));
        i += 1;
    }
    // Keep the overlay's height in sync with a trailing newline in the textarea.
    out.push_str("\n ");
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn highlights_and_escapes() {
        let h = super::js("const a = '<b>'; // hi");
        assert!(h.contains("tk-k"));
        assert!(h.contains("&lt;b&gt;"));
        assert!(h.contains("tk-c"));
    }
}
