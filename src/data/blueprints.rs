//! System-design diagrams. Coordinates live in a 780 × 380 viewBox.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Svc,
    Ext,
    Ai,
    Db,
}

pub struct Node {
    pub id: &'static str,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub l: &'static str,
    pub s: &'static str,
    pub k: NodeKind,
}

pub struct Blueprint {
    pub id: &'static str,
    pub title: &'static str,
    pub nodes: &'static [Node],
    pub edges: &'static [(&'static str, &'static str, &'static str)],
    pub notes: &'static [(&'static str, &'static str)],
}

pub const NODE_H: f64 = 46.0;

use NodeKind::*;

const fn n(id: &'static str, x: f64, y: f64, w: f64, l: &'static str, s: &'static str, k: NodeKind) -> Node {
    Node { id, x, y, w, l, s, k }
}

pub const BLUEPRINTS: &[Blueprint] = &[
    Blueprint {
        id: "cypheron", title: "Cypheron",
        nodes: &[
            n("ui", 20.0, 40.0, 150.0, "React + Tailwind", "scan dashboard", Svc),
            n("api", 230.0, 40.0, 150.0, "FastAPI gateway", "REST + WebSocket", Svc),
            n("orc", 230.0, 170.0, 150.0, "Scan orchestrator", "async job runner", Svc),
            n("mods", 440.0, 170.0, 170.0, "Check modules ×7", "TLS · headers · CORS …", Svc),
            n("tgt", 650.0, 170.0, 110.0, "Target site", "HTTPX probes", Ext),
            n("ai", 440.0, 40.0, 170.0, "Claude AI", "triage + remediation", Ai),
            n("db", 230.0, 300.0, 150.0, "SQLite", "scan history", Db),
            n("grd", 440.0, 300.0, 170.0, "Risk scorer", "weights → A–F grade", Svc),
        ],
        edges: &[("ui", "api", "POST /scan"), ("api", "ui", "ws progress"), ("api", "orc", ""), ("orc", "mods", "fan-out"), ("mods", "tgt", "GET / HEAD"), ("mods", "grd", "findings"), ("grd", "ai", "scored"), ("ai", "api", "report"), ("orc", "db", "persist")],
        notes: &[
            ("Live progress", "WebSockets push each module result as it lands, so the dashboard never polls."),
            ("Pluggable checks", "Each module is independent; adding a check means adding one coroutine."),
            ("AI after scoring", "Deterministic rules decide severity; the LLM only explains and prioritises, so grades stay reproducible."),
            ("History", "Every scan is stored in SQLite for comparison over time."),
        ],
    },
    Blueprint {
        id: "aviai", title: "AvIAI",
        nodes: &[
            n("os", 20.0, 60.0, 140.0, "OpenSky API", "live ADS-B states", Ext),
            n("ing", 210.0, 60.0, 150.0, "Ingest worker", "60 NM geofence", Svc),
            n("rd", 410.0, 60.0, 150.0, "Redis", "state + pub/sub", Db),
            n("ctl", 410.0, 200.0, 150.0, "AI controller", "conflict + sequencing", Svc),
            n("llm", 610.0, 200.0, 140.0, "Mistral AI", "clearance phrasing", Ai),
            n("api", 610.0, 60.0, 140.0, "FastAPI WS", "stream to clients", Svc),
            n("map", 610.0, 320.0, 140.0, "Leaflet map", "digital twin UI", Svc),
            n("db", 210.0, 200.0, 150.0, "SQLite", "clearance log", Db),
        ],
        edges: &[("os", "ing", "poll"), ("ing", "rd", "upsert"), ("rd", "ctl", "subscribe"), ("ctl", "llm", "intent"), ("llm", "ctl", "clearance"), ("rd", "api", "publish"), ("api", "map", "ws"), ("ctl", "db", "log"), ("ctl", "rd", "assign")],
        notes: &[
            ("Hot state in Redis", "Aircraft state changes every few seconds; Redis keeps reads cheap and fans out via pub/sub."),
            ("Decide, then phrase", "The controller decides heading, altitude and speed; the LLM turns that into standard clearance phrasing."),
            ("Bounded airspace", "Only traffic inside 60 NM is ingested, which caps load and keeps the twin focused."),
            ("Push, not poll", "Clients receive a stream of updates over WebSockets."),
        ],
    },
    Blueprint {
        id: "fintrix", title: "Fintrix",
        nodes: &[
            n("doc", 20.0, 40.0, 150.0, "RBI / FEMA texts", "source corpus", Ext),
            n("chk", 210.0, 40.0, 140.0, "Chunker", "clause-aware splits", Svc),
            n("emb", 400.0, 0.0, 170.0, "SentenceTransformers", "dense vectors", Ai),
            n("fs", 620.0, 0.0, 130.0, "FAISS", "vector index", Db),
            n("bm", 620.0, 100.0, 130.0, "BM25", "keyword index", Db),
            n("hy", 400.0, 200.0, 170.0, "Hybrid retriever", "score fusion", Svc),
            n("llm", 210.0, 200.0, 140.0, "Mistral AI", "grounded answer", Ai),
            n("ui", 20.0, 200.0, 150.0, "React UI", "chat + what-if", Svc),
            n("wi", 210.0, 320.0, 140.0, "What-If engine", "rule-based risk", Svc),
            n("db", 400.0, 320.0, 170.0, "SQLite", "sessions + scenarios", Db),
        ],
        edges: &[("doc", "chk", ""), ("chk", "emb", "embed"), ("emb", "fs", "index"), ("chk", "bm", "tokenise"), ("fs", "hy", "top-k"), ("bm", "hy", "top-k"), ("hy", "llm", "context + cites"), ("llm", "ui", "answer"), ("ui", "wi", "scenario"), ("wi", "db", "store")],
        notes: &[
            ("Why hybrid", "Legal text mixes exact terms (section numbers, \"LRS\") and paraphrase; BM25 catches the first, embeddings the second."),
            ("Source-backed", "Every answer carries the chunk it came from; low retrieval confidence means no answer."),
            ("Rules for risk", "What-If uses deterministic rules, not the LLM, so risk verdicts are auditable."),
            ("Small footprint", "FAISS + SQLite run on one machine with no external vector DB."),
        ],
    },
    Blueprint {
        id: "nishank-os", title: "NishankOS",
        nodes: &[
            n("rs", 20.0, 30.0, 150.0, "Rust crate", "apps · os · data", Svc),
            n("tr", 220.0, 30.0, 150.0, "Trunk + rustc", "wasm32 target", Ext),
            n("wasm", 420.0, 30.0, 150.0, "nishank_os.wasm", "wasm-bindgen glue", Ai),
            n("lp", 420.0, 160.0, 150.0, "Leptos runtime", "fine-grained signals", Svc),
            n("wm", 220.0, 160.0, 150.0, "Window manager", "z-order · snap · resize", Svc),
            n("cv", 620.0, 160.0, 140.0, "Canvas 2D", "wallpapers · radar", Svc),
            n("wk", 620.0, 300.0, 140.0, "Web Worker", "CodeLab sandbox", Ext),
            n("ls", 220.0, 300.0, 150.0, "localStorage", "theme · progress", Db),
            n("dom", 420.0, 300.0, 150.0, "DOM", "surgical updates", Svc),
        ],
        edges: &[("rs", "tr", "cargo build"), ("tr", "wasm", "emit"), ("wasm", "lp", "mount"), ("lp", "wm", "signals"), ("lp", "cv", "rAF loops"), ("lp", "dom", "patch"), ("wm", "ls", "persist"), ("cv", "wk", ""), ("dom", "wk", "postMessage")],
        notes: &[
            ("No virtual DOM", "Leptos signals update exactly the text node or attribute that changed, so dragging a window touches four style properties per frame."),
            ("Compute in WASM", "Wallpaper physics, radar kinematics and BM25 scoring run as compiled Rust, not interpreted JS."),
            ("Isolation", "Visitor code in CodeLab never runs on the main thread; a Worker is spawned per run and terminated after 4 s."),
            ("Data as code", "Every project, cert and skill is a typed const, so a typo in the portfolio is a compile error."),
        ],
    },
];

pub fn by_id(id: &str) -> Option<&'static Blueprint> {
    BLUEPRINTS.iter().find(|b| b.id == id)
}
