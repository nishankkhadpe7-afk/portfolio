//! A read-only virtual filesystem rooted at `~` (= /home/nishank).
//! `~/src` holds this OS's *real* source, embedded at compile time.

use crate::data::{certs::CERTS, projects::PROJECTS};

pub enum Body {
    /// Terminal markup (see `markup.rs`).
    Markup(String),
    /// Shown verbatim.
    Plain(&'static str),
}

pub const SRC: &[(&str, &str)] = &[
    ("main.rs", include_str!("../../main.rs")),
    ("app.rs", include_str!("../../app.rs")),
    ("state.rs", include_str!("../../state.rs")),
    ("Cargo.toml", include_str!("../../../Cargo.toml")),
];

/// Entries of a directory; names ending in `/` are directories.
pub fn list(dir: &str, all: bool) -> Option<Vec<String>> {
    let mut v: Vec<String> = match dir {
        "~" => ["about.txt", "resume.md", "contact.vcf", "projects/", "certificates/", "src/"].map(String::from).to_vec(),
        "~/projects" => PROJECTS.iter().map(|p| format!("{}.md", p.id)).collect(),
        "~/certificates" => CERTS.iter().map(|c| format!("{}.cert", c.id)).collect(),
        "~/src" => SRC.iter().map(|(n, _)| n.to_string()).collect(),
        _ => return None,
    };
    if all && dir == "~" {
        v.insert(0, ".secrets".into());
        v.insert(0, ".zshrc".into());
    }
    Some(v)
}

pub fn is_dir(p: &str) -> bool {
    list(p, false).is_some()
}

/// Normalise `arg` against `cwd` into an absolute `~/…` path.
pub fn resolve(cwd: &str, arg: &str) -> String {
    let arg = arg.trim_end_matches('/');
    let base: Vec<&str> = if arg.starts_with('~') || arg.starts_with("/home/nishank") {
        vec!["~"]
    } else {
        cwd.split('/').collect()
    };
    let rest = arg.trim_start_matches("/home/nishank").trim_start_matches('~');
    let mut parts: Vec<String> = base.iter().map(|s| s.to_string()).collect();
    for seg in rest.split('/').filter(|s| !s.is_empty() && *s != ".") {
        if seg == ".." {
            if parts.len() > 1 {
                parts.pop();
            }
        } else {
            parts.push(seg.to_string());
        }
    }
    parts.join("/")
}

pub fn read(path: &str) -> Option<Body> {
    use crate::data::profile::ME;
    let rel = path.trim_start_matches("~/");
    match rel {
        "about.txt" => Some(Body::Markup(format!(
            "[u]{}[/] — {}\n{}, {} ({}) · CGPA {}/10\n\nI build systems where AI meets real-time data and security: a web vulnerability\nscanner with AI triage, a hybrid-RAG compliance assistant and an AI-run ATC digital twin.\nInterests: system design · cyber security · web development · ML development.\nCurrently: {}",
            ME.name, ME.role, ME.degree, ME.school, ME.years, ME.cgpa, ME.status
        ))),
        "resume.md" => Some(Body::Markup(super::commands::resume())),
        "contact.vcf" => Some(Body::Markup(format!(
            "BEGIN:VCARD\nVERSION:3.0\nFN:{}\nEMAIL:{}\nTEL:{}\nURL:{}\nURL:{}\nADR:{}\nEND:VCARD\n{{download vcard|→ download contact.vcf}}",
            ME.name, ME.email, ME.phone, ME.linkedin, ME.github, ME.loc
        ))),
        ".zshrc" => Some(Body::Plain("# ~/.zshrc — NishankOS\nexport EDITOR=hx\nexport CURIOSITY=unbounded\nalias please='sudo'\nalias ll='ls -la'\nsetopt CORRECT   # spelling is a feature\n")),
        ".secrets" => Some(Body::Markup("[g]flag{curiosity_is_a_feature}[/]\n[m]You found the hidden file. Achievement unlocked: ls -a.[/]".into())),
        _ => {
            if let Some(id) = rel.strip_prefix("projects/").and_then(|s| s.strip_suffix(".md")) {
                return crate::data::projects::by_id(id).map(|p| Body::Markup(super::commands::project_text(p)));
            }
            if let Some(id) = rel.strip_prefix("certificates/").and_then(|s| s.strip_suffix(".cert")) {
                return crate::data::certs::by_id(id).map(|c| {
                    let mut s = format!("[u]{}[/]\n{} · {}", c.title, c.issuer, c.date);
                    if let Some(h) = c.hours {
                        s += &format!(" · {h}");
                    }
                    if let Some(u) = c.url {
                        s += &format!("\nverify: <<{u}|{}>>", u.trim_start_matches("https://"));
                    }
                    s += &format!("\n{{open certs {}|→ open in Certificates}}", c.id);
                    Body::Markup(s)
                });
            }
            if let Some(name) = rel.strip_prefix("src/") {
                return SRC.iter().find(|(n, _)| *n == name).map(|(_, body)| Body::Plain(body));
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolves_paths() {
        assert_eq!(resolve("~", "projects"), "~/projects");
        assert_eq!(resolve("~/projects", ".."), "~");
        assert_eq!(resolve("~/projects", "../src/main.rs"), "~/src/main.rs");
        assert_eq!(resolve("~/src", "~"), "~");
        assert_eq!(resolve("~", "/home/nishank/certificates/"), "~/certificates");
    }
}
