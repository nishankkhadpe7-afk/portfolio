//! The Cypheron demo's finding catalogue and scoring rules.

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Sev {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Sev {
    pub fn weight(self) -> u32 {
        match self {
            Sev::Critical => 30,
            Sev::High => 15,
            Sev::Medium => 8,
            Sev::Low => 3,
            Sev::Info => 0,
        }
    }
    pub fn css(self) -> &'static str {
        match self {
            Sev::Critical => "critical",
            Sev::High => "high",
            Sev::Medium => "medium",
            Sev::Low => "low",
            Sev::Info => "info",
        }
    }
    pub const ALL: [Sev; 5] = [Sev::Critical, Sev::High, Sev::Medium, Sev::Low, Sev::Info];
}

pub const MODS: &[(&str, &str)] = &[
    ("tls", "HTTPS / TLS"),
    ("headers", "Security headers"),
    ("cookies", "Cookies"),
    ("cors", "CORS policy"),
    ("forms", "Forms"),
    ("disclosure", "Info disclosure"),
    ("files", "Exposed files"),
];

#[derive(PartialEq)]
pub struct Finding {
    pub m: &'static str,
    pub sev: Sev,
    pub t: &'static str,
    pub d: &'static str,
    pub f: &'static str,
    pub cwe: &'static str,
}

pub const FINDINGS: &[Finding] = &[
    Finding { m: "files", sev: Sev::Critical, t: "Exposed .git directory", d: "GET /.git/HEAD returned 200 with a ref line. Source code and commit history can be reconstructed.", f: "Deny /.git/ at the web server and rotate any secret that ever appeared in history.", cwe: "CWE-538" },
    Finding { m: "files", sev: Sev::High, t: "Backup file reachable: /config.php.bak", d: "A backup copy of a server config is served as plain text.", f: "Delete backup artifacts from the web root and block *.bak, *.old, *.swp.", cwe: "CWE-530" },
    Finding { m: "cors", sev: Sev::High, t: "CORS reflects any Origin with credentials", d: "Access-Control-Allow-Origin echoes the request Origin while Allow-Credentials is true.", f: "Use an explicit allow-list of trusted origins; never reflect arbitrary origins with credentials.", cwe: "CWE-942" },
    Finding { m: "headers", sev: Sev::High, t: "Content-Security-Policy missing", d: "No CSP header, so injected scripts run without restriction.", f: "Start with default-src 'self' and tighten per resource type.", cwe: "CWE-693" },
    Finding { m: "cookies", sev: Sev::Medium, t: "Session cookie lacks HttpOnly and Secure", d: "Cookie \"sessionid\" is readable from JavaScript and can travel over HTTP.", f: "Set HttpOnly, Secure and SameSite=Lax on session cookies.", cwe: "CWE-1004" },
    Finding { m: "tls", sev: Sev::Medium, t: "Strict-Transport-Security not set", d: "Browsers may still try HTTP first, which opens a downgrade window.", f: "Send Strict-Transport-Security: max-age=31536000; includeSubDomains.", cwe: "CWE-319" },
    Finding { m: "tls", sev: Sev::Medium, t: "TLS 1.0 still accepted", d: "Handshake succeeded with TLS 1.0, a deprecated protocol.", f: "Allow TLS 1.2+ only.", cwe: "CWE-326" },
    Finding { m: "forms", sev: Sev::Medium, t: "Login form has no CSRF token", d: "POST /login carries no anti-CSRF token field.", f: "Add a per-session CSRF token and validate it server-side.", cwe: "CWE-352" },
    Finding { m: "disclosure", sev: Sev::Low, t: "Server banner reveals version", d: "Server: nginx/1.18.0 leaks the exact version.", f: "Set server_tokens off.", cwe: "CWE-200" },
    Finding { m: "headers", sev: Sev::Low, t: "X-Content-Type-Options missing", d: "Browsers may MIME-sniff responses.", f: "Send X-Content-Type-Options: nosniff.", cwe: "CWE-16" },
    Finding { m: "headers", sev: Sev::Low, t: "Clickjacking protection missing", d: "No X-Frame-Options or frame-ancestors directive.", f: "Add frame-ancestors 'none' to CSP.", cwe: "CWE-1021" },
    Finding { m: "disclosure", sev: Sev::Info, t: "robots.txt lists /admin", d: "The path hint is public, though not a vulnerability by itself.", f: "Protect /admin with auth; do not rely on obscurity.", cwe: "CWE-200" },
];

/// `100 − Σ weights`, floored at zero.
pub fn score(fs: &[&Finding]) -> u32 {
    100u32.saturating_sub(fs.iter().map(|f| f.sev.weight()).sum())
}

/// Letter grade and the CSS colour variable to paint it with.
pub fn grade(score: u32) -> (&'static str, &'static str) {
    match score {
        90.. => ("A", "var(--ok)"),
        80..=89 => ("B", "var(--ok)"),
        65..=79 => ("C", "var(--warn)"),
        50..=64 => ("D", "var(--high)"),
        _ => ("F", "var(--crit)"),
    }
}
