//! Who is behind the OS.

pub struct Profile {
    pub name: &'static str,
    pub handle: &'static str,
    pub host: &'static str,
    pub role: &'static str,
    pub school: &'static str,
    pub degree: &'static str,
    pub years: &'static str,
    pub cgpa: &'static str,
    pub email: &'static str,
    pub phone: &'static str,
    pub linkedin: &'static str,
    pub github: &'static str,
    pub loc: &'static str,
    pub status: &'static str,
}

pub const ME: Profile = Profile {
    name: "Nishank Khadpe",
    handle: "nishank",
    host: "nishank-os",
    role: "AI & Data Science undergrad · builder of secure, real-time AI systems",
    school: "SIES Graduate School of Technology, Nerul, Navi Mumbai",
    degree: "B.E. Artificial Intelligence & Data Science",
    years: "2024 – 2028",
    cgpa: "9.24",
    email: "khadpenishank@gmail.com",
    phone: "+91 99672 48550",
    linkedin: "https://linkedin.com/in/nishank-k-84ba63363",
    github: "https://github.com/nishankkhadpe7-afk",
    loc: "Uran, Navi Mumbai, India",
    status: "Open to internships · security, AI/ML, data",
};

pub struct Education {
    pub years: &'static str,
    pub title: &'static str,
    pub place: &'static str,
    pub score: &'static str,
}

pub const EDUCATION: &[Education] = &[
    Education { years: "2024 – 2028", title: ME.degree, place: ME.school, score: "CGPA 9.24" },
    Education { years: "2024", title: "HSC, Science", place: "Uran Education Society, Uran", score: "79.17%" },
    Education { years: "2022", title: "SSC", place: "Uran Education Society, Uran", score: "86.60%" },
];

/// The four things Nishank cares about; each card deep-links into an app.
pub struct Focus {
    pub icon: &'static str,
    pub title: &'static str,
    pub body: &'static str,
    pub cta: &'static str,
    pub app: &'static str,
    /// Self-assessed depth, 0–100, drawn on the skill radar.
    pub level: f64,
}

pub const FOCUS: &[Focus] = &[
    Focus { icon: "blueprint", title: "System Design", body: "Real-time pipelines with FastAPI, Redis and WebSockets; event-driven scanners and RAG services.", cta: "open blueprints →", app: "blueprints", level: 78.0 },
    Focus { icon: "shield", title: "Cyber Security", body: "Cypheron audits TLS, headers, cookies, CORS, forms and exposed files, then triages with AI.", cta: "run a scan →", app: "cypheron", level: 84.0 },
    Focus { icon: "code", title: "Web Development", body: "React + Tailwind front ends on FastAPI back ends — and now this OS, in Rust + WebAssembly.", cta: "see projects →", app: "files", level: 80.0 },
    Focus { icon: "brain", title: "ML Development", body: "Hybrid RAG with FAISS + BM25, LLM agents with Mistral and Claude, agentic AI with Python.", cta: "query Fintrix →", app: "fintrix", level: 82.0 },
];

/// Extra radar axes beyond the focus areas.
pub const RADAR_EXTRA: &[(&str, f64)] = &[("Data Analysis", 74.0), ("Rust / WASM", 62.0)];

/// The journey, newest first. Rendered as `git log` in the terminal and as
/// the timeline in whoami.
pub struct Milestone {
    pub hash: &'static str,
    pub date: &'static str,
    pub kind: &'static str,
    pub scope: &'static str,
    pub msg: &'static str,
}

pub const TIMELINE: &[Milestone] = &[
    Milestone { hash: "f3a91c0", date: "Nov 2026", kind: "feat", scope: "nishank-os", msg: "rewrite the portfolio OS in Rust + WebAssembly" },
    Milestone { hash: "c9d2e17", date: "Oct 2026", kind: "feat", scope: "cypheron", msg: "AI-powered web security scanner with live WebSocket dashboard" },
    Milestone { hash: "b71f4a2", date: "Sep 2026", kind: "award", scope: "agentx", msg: "second prize · AgentX 2026 autonomous AI agent ideathon" },
    Milestone { hash: "a04c8e9", date: "Sep 2026", kind: "learn", scope: "genai", msg: "Full Stack Generative & Agentic AI (32.5 h) + Web Dev (100 h)" },
    Milestone { hash: "9e3b55d", date: "Jun 2026", kind: "feat", scope: "analytics", msg: "Power BI dashboard · Adoption of Technology by World" },
    Milestone { hash: "8d6a1f3", date: "Apr 2026", kind: "feat", scope: "fintrix", msg: "hybrid RAG compliance assistant over RBI / FEMA" },
    Milestone { hash: "7c2e9b0", date: "Apr 2026", kind: "award", scope: "rapidcare", msg: "top 80 of 500+ teams · Quantum Arena 1.0" },
    Milestone { hash: "6b81d44", date: "Mar 2026", kind: "learn", scope: "fabric", msg: "Lakehouse implementation with Microsoft Fabric" },
    Milestone { hash: "5a07c3e", date: "Jan 2026", kind: "feat", scope: "aviai", msg: "airport digital twin with AI air-traffic controllers" },
    Milestone { hash: "1f00ba5", date: "Aug 2024", kind: "init", scope: "sies-gst", msg: "begin B.E. Artificial Intelligence & Data Science" },
];
