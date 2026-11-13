//! Skills, presented as installed crates. `(name, section, where it is used)`.

pub const SECTIONS: &[&str] = &["lang", "web", "ai-ml", "security", "data", "tools"];

pub const PKGS: &[(&str, &str, &str)] = &[
    ("python", "lang", "Cypheron, Fintrix, AvIAI"),
    ("rust", "lang", "NishankOS — this very desktop"),
    ("javascript", "lang", "Cypheron, Fintrix UI"),
    ("sql", "lang", "SQLite / PostgreSQL work"),
    ("html-css", "lang", "Web front ends"),
    ("fastapi", "web", "Cypheron, Fintrix, AvIAI"),
    ("react", "web", "Cypheron, Fintrix"),
    ("leptos", "web", "NishankOS reactive UI"),
    ("wasm-bindgen", "web", "NishankOS browser bindings"),
    ("flask", "web", "Python web services"),
    ("tailwindcss", "web", "Cypheron dashboard"),
    ("websockets", "web", "Cypheron live scans, AvIAI live tracking"),
    ("leaflet", "web", "AvIAI airspace map"),
    ("scikit-learn", "ai-ml", "ML coursework and models"),
    ("sentence-transformers", "ai-ml", "Fintrix dense embeddings"),
    ("faiss", "ai-ml", "Fintrix vector index"),
    ("bm25", "ai-ml", "Fintrix keyword retrieval (re-implemented in Rust here)"),
    ("rag", "ai-ml", "Fintrix hybrid pipeline"),
    ("mistral-ai", "ai-ml", "Fintrix answers, AvIAI clearances"),
    ("claude-ai", "ai-ml", "Cypheron security analysis"),
    ("agentic-ai", "ai-ml", "AgentX 2026 (2nd prize), Udemy GenAI course"),
    ("httpx", "security", "Cypheron async probing"),
    ("beautifulsoup4", "security", "Cypheron form and page parsing"),
    ("tls-audit", "security", "Cypheron HTTPS/TLS checks"),
    ("header-cookie-cors-audit", "security", "Cypheron policy checks"),
    ("ctf", "security", "Capture-the-flag practice"),
    ("sqlite", "data", "Cypheron history, Fintrix, AvIAI logs"),
    ("postgresql", "data", "Relational data work"),
    ("redis", "data", "AvIAI real-time state"),
    ("power-bi", "data", "Tech adoption dashboard"),
    ("ms-fabric-lakehouse", "data", "Lakehouse certification"),
    ("git", "tools", "GitHub workflow"),
    ("vscode", "tools", "Daily driver"),
    ("trunk", "tools", "NishankOS wasm bundler"),
    ("opensky-api", "tools", "AvIAI live flight data"),
];

/// Pseudo semver for the `cargo`-style listing, stable per package name.
pub fn version(name: &str) -> String {
    let h = crate::util::rng::fnv1a(name);
    format!("{}.{}.{}", 1 + h % 4, (h >> 4) % 20, (h >> 9) % 12)
}
