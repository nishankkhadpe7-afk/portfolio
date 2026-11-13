//! Terminal output model and a tiny markup language for writing it.
//!
//! ```text
//! [y]yellow text[/]      colour classes: b u d g y r m c
//! {cmd}  {cmd|label}     clickable command (runs in this shell)
//! <<url|label>>          external link
//! ```
//! User input never goes through the parser — it is always a plain segment.

#[derive(Clone, PartialEq)]
pub enum Link {
    None,
    Cmd(String),
    Url(String),
}

#[derive(Clone, PartialEq)]
pub struct Seg {
    pub text: String,
    pub cls: &'static str,
    pub link: Link,
}

#[derive(Clone, PartialEq)]
pub enum Line {
    Segs(Vec<Seg>),
    Prompt { cwd: String, cmd: String },
    Neofetch(Vec<(&'static str, String)>),
}

fn class(c: char) -> &'static str {
    match c {
        'b' => "t-b",
        'u' => "t-u",
        'd' => "t-d",
        'g' => "t-g",
        'y' => "t-y",
        'r' => "t-r",
        'm' => "t-m",
        'c' => "t-c",
        _ => "",
    }
}

/// A plain, unparsed line (for file contents and user input).
pub fn plain(s: impl Into<String>) -> Line {
    Line::Segs(vec![Seg { text: s.into(), cls: "", link: Link::None }])
}

pub fn plain_cls(s: impl Into<String>, cls: &'static str) -> Line {
    Line::Segs(vec![Seg { text: s.into(), cls, link: Link::None }])
}

/// Parse markup into lines (split on `\n`).
pub fn md(src: &str) -> Vec<Line> {
    src.split('\n').map(parse_line).collect()
}

fn parse_line(s: &str) -> Line {
    let mut out: Vec<Seg> = Vec::new();
    let mut buf = String::new();
    let mut cls: &'static str = "";
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let flush = |buf: &mut String, out: &mut Vec<Seg>, cls: &'static str| {
        if !buf.is_empty() {
            out.push(Seg { text: std::mem::take(buf), cls, link: Link::None });
        }
    };
    while i < chars.len() {
        let c = chars[i];
        // [x] … [/]
        if c == '[' && i + 2 < chars.len() && chars[i + 2] == ']' && (!class(chars[i + 1]).is_empty() || chars[i + 1] == '/') {
            flush(&mut buf, &mut out, cls);
            cls = if chars[i + 1] == '/' { "" } else { class(chars[i + 1]) };
            i += 3;
            continue;
        }
        // {cmd|label}
        if c == '{' {
            if let Some(end) = chars[i..].iter().position(|&x| x == '}') {
                flush(&mut buf, &mut out, cls);
                let inner: String = chars[i + 1..i + end].iter().collect();
                let (cmd, label) = inner.split_once('|').map(|(a, b)| (a.to_string(), b.to_string())).unwrap_or((inner.clone(), inner));
                out.push(Seg { text: label, cls: "t-link", link: Link::Cmd(cmd) });
                i += end + 1;
                continue;
            }
        }
        // <<url|label>>
        if c == '<' && chars.get(i + 1) == Some(&'<') {
            let rest: String = chars[i + 2..].iter().collect();
            if let Some(end) = rest.find(">>") {
                flush(&mut buf, &mut out, cls);
                let inner = &rest[..end];
                let (url, label) = inner.split_once('|').unwrap_or((inner, inner));
                out.push(Seg { text: label.to_string(), cls: "t-link", link: Link::Url(url.to_string()) });
                i += 2 + rest[..end].chars().count() + 2;
                continue;
            }
        }
        buf.push(c);
        i += 1;
    }
    flush(&mut buf, &mut out, cls);
    Line::Segs(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_colour_links_and_urls() {
        let Line::Segs(s) = &md("[y]hi[/] {help} {open x|go} <<https://a.b|site>>")[0] else { panic!() };
        assert_eq!(s[0].cls, "t-y");
        assert!(matches!(&s[2].link, Link::Cmd(c) if c == "help"));
        assert!(matches!(&s[4].link, Link::Cmd(c) if c == "open x"));
        assert_eq!(s[4].text, "go");
        assert!(matches!(&s[6].link, Link::Url(u) if u == "https://a.b"));
    }
}
