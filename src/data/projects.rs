//! Shipped work.

pub struct Project {
    pub id: &'static str,
    pub name: &'static str,
    pub date: &'static str,
    pub domain: &'static str,
    pub icon: &'static str,
    /// Key of the interactive demo app, if one exists in the OS.
    pub app: Option<&'static str>,
    /// Key of the matching system-design blueprint, if any.
    pub blueprint: Option<&'static str>,
    /// Public URL if the project is deployed and live.
    pub live: Option<&'static str>,
    pub tag: &'static str,
    pub stack: &'static [&'static str],
    pub pts: &'static [&'static str],
}

pub const PROJECTS: &[Project] = &[
    Project {
        id: "brandproof", name: "BrandProof", date: "Nov 2026", domain: "AI · Brand Governance", icon: "check",
        app: None, blueprint: None, live: Some("https://brand-proof-three.vercel.app/welcome"),
        tag: "Pre-publication brand-governance gate: checks copy and creatives against brand rules and approved facts",
        stack: &["Next.js", "FastAPI", "Pluggable LLM", "OCR", "Semantic matching", "Figma plugin", "Vercel", "Render"],
        pts: &[
            "Turns uploaded brand books and spec sheets into checkable rules, asking reviewers to confirm vague ones.",
            "Checks pasted text, banners, PDFs and Word files — including text inside images via OCR — for contradicted claims, expired facts and off-brand wording.",
            "Every flag cites the rule or fact behind it with a suggested fix; humans accept, edit or override with a reason, per-brand workspaces keep an approval record, and a Figma plugin applies fixes to text layers.",
        ],
    },
    Project {
        id: "cypheron", name: "Cypheron", date: "Oct 2026", domain: "Cyber Security", icon: "shield",
        app: Some("cypheron"), blueprint: Some("cypheron"), live: None,
        tag: "AI-powered web security scanner and vulnerability assessment platform",
        stack: &["Python", "FastAPI", "React", "WebSockets", "SQLite", "HTTPX", "BeautifulSoup", "Claude AI", "Tailwind CSS"],
        pts: &[
            "Runs automated security assessments across HTTPS/TLS, security headers, cookies, CORS, forms, information disclosure and exposed files.",
            "Uses Claude AI to turn raw findings into plain-English analysis, prioritise vulnerabilities and suggest concrete fixes.",
            "Real-time dashboard over FastAPI + WebSockets: live scan progress, risk scoring, A–F grading, persistent scan history and detailed reports.",
        ],
    },
    Project {
        id: "fintrix", name: "Fintrix", date: "Apr 2026", domain: "ML · RAG", icon: "scale",
        app: Some("fintrix"), blueprint: Some("fintrix"), live: None,
        tag: "AI-powered financial compliance assistant grounded in RBI / FEMA regulation",
        stack: &["Python", "FastAPI", "React", "FAISS", "BM25", "SentenceTransformers", "Mistral AI", "SQLite"],
        pts: &[
            "Hybrid RAG pipeline grounded in RBI and FEMA regulations to keep answers source-backed and cut hallucinations.",
            "Combines FAISS dense retrieval (SentenceTransformers) with BM25 keyword search for context-aware answers.",
            "What-If Analysis module scores financial scenarios with rule-based risk assessment.",
        ],
    },
    Project {
        id: "aviai", name: "AvIAI", date: "Jan 2026", domain: "ML · Real-time Systems", icon: "radar",
        app: Some("aviai"), blueprint: Some("aviai"), live: None,
        tag: "AI-powered airport digital twin and air traffic control simulation",
        stack: &["Python", "FastAPI", "Redis", "OpenSky API", "WebSockets", "Mistral AI", "Leaflet", "SQLite"],
        pts: &[
            "Airport digital twin with autonomous AI controllers managing a 60 nautical-mile airspace.",
            "Mistral AI generates real-time ATC clearances with heading, altitude and speed instructions.",
            "Real-time flight tracking and simulation pipeline built on OpenSky, Redis and WebSockets.",
        ],
    },
    Project {
        id: "nishank-os", name: "NishankOS", date: "Nov 2026", domain: "Rust · WebAssembly", icon: "cpu",
        app: Some("monitor"), blueprint: Some("nishank-os"), live: None,
        tag: "This portfolio: a desktop OS in the browser, written in Rust and compiled to wasm32",
        stack: &["Rust", "Leptos", "WebAssembly", "Canvas 2D", "Web Workers", "Trunk"],
        pts: &[
            "Fine-grained reactive window manager with drag, 8-way resize, edge snapping, minimise/maximise and an exposé overview.",
            "Six procedurally animated wallpapers, the AvIAI radar and the Fintrix BM25 + trigram retriever all run as Rust in WASM.",
            "CodeLab sandboxes untrusted JavaScript in a Web Worker and grades it against reference solutions written in Rust.",
        ],
    },
    Project {
        id: "rapidcare", name: "RapidCare", date: "Apr 2026", domain: "Hackathon", icon: "award",
        app: None, blueprint: None, live: None,
        tag: "ICU bed allocation and emergency response system",
        stack: &["Hospital matching", "Emergency routing"],
        pts: &[
            "Built at Quantum Arena 1.0, a national-level hackathon at Navsahyadri Group of Institutions, Pune.",
            "Intelligent hospital-matching to allocate ICU beds during emergencies.",
            "Ranked in the Top 80 of 500+ participating teams.",
        ],
    },
    Project {
        id: "techadoption", name: "Adoption of Technology by World", date: "Jun 2026", domain: "Data Analytics", icon: "chart",
        app: None, blueprint: None, live: None,
        tag: "Power BI dashboard built during a Data Analysis internship",
        stack: &["Power BI", "Data cleaning", "Visualisation"],
        pts: &[
            "Covers the full loop: data collection, cleaning, visualisation and trend interpretation.",
            "Built during the Data Visualization internship at SIES Graduate School of Technology.",
        ],
    },
];

/// Projects that are deployed and publicly reachable.
pub fn live() -> impl Iterator<Item = &'static Project> {
    PROJECTS.iter().filter(|p| p.live.is_some())
}

pub fn by_id(id: &str) -> Option<&'static Project> {
    PROJECTS.iter().find(|p| p.id == id)
}
