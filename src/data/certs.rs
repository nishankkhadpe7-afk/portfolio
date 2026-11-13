//! Awards and courses.

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    Award,
    Course,
}

pub struct Cert {
    pub id: &'static str,
    pub kind: Kind,
    pub title: &'static str,
    pub sub: Option<&'static str>,
    pub issuer: &'static str,
    pub date: &'static str,
    pub hours: Option<&'static str>,
    pub url: Option<&'static str>,
    pub no: Option<&'static str>,
    /// Path to a scanned certificate under `assets/`.
    pub img: Option<&'static str>,
    pub facts: &'static [(&'static str, &'static str)],
}

pub const CERTS: &[Cert] = &[
    Cert {
        id: "agentx", kind: Kind::Award, title: "AgentX 2026 · Second Prize", sub: Some("Autonomous AI Agent Ideathon"),
        issuer: "ISTE TEC × NEAR AI · Terna Engineering College", date: "29 Sep 2026", hours: None, url: None, no: None,
        img: Some("assets/certs/agentx.jpg"),
        facts: &[
            ("Event", "AgentX 2026, an Autonomous AI Agent Ideathon"),
            ("Result", "Second Prize Winner"),
            ("Organised by", "ISTE TEC × NEAR AI"),
            ("Venue", "Terna Engineering College"),
            ("Cited for", "Innovative idea, thoughtful approach and exceptional presentation in autonomous AI agents"),
        ],
    },
    Cert {
        id: "genai", kind: Kind::Course, title: "Full Stack Generative and Agentic AI with Python", sub: None,
        issuer: "Udemy · Hitesh Choudhary, Piyush Garg", date: "18 Sep 2026", hours: Some("32.5 hours"),
        url: Some("https://ude.my/UC-320dc490-f82e-4f9c-a1a7-218d8459c490"), no: Some("UC-320dc490-f82e-4f9c-a1a7-218d8459c490"),
        img: None, facts: &[],
    },
    Cert {
        id: "webdev", kind: Kind::Course, title: "Complete Web Development Course", sub: None,
        issuer: "Udemy · Hitesh Choudhary", date: "18 Sep 2026", hours: Some("100 hours"),
        url: Some("https://ude.my/UC-55c5da25-a38f-4c54-953e-567b087a6146"), no: Some("UC-55c5da25-a38f-4c54-953e-567b087a6146"),
        img: None, facts: &[],
    },
    Cert {
        id: "quantum", kind: Kind::Award, title: "Quantum Arena 1.0 · Top 80 of 500+", sub: Some("National-level hackathon"),
        issuer: "Navsahyadri Group of Institutions, Pune", date: "Apr 2026", hours: None, url: None, no: None, img: None,
        facts: &[("Project", "RapidCare, ICU bed allocation and emergency response"), ("Result", "Top 80 of 500+ teams")],
    },
    Cert {
        id: "fabric", kind: Kind::Course, title: "Implementation of Lakehouse with Microsoft Fabric", sub: None,
        issuer: "SIES Graduate School of Technology", date: "Mar 2026", hours: None, url: None, no: None, img: None,
        facts: &[("Covered", "Modern data engineering, Lakehouse architecture, data management")],
    },
    Cert {
        id: "dataviz", kind: Kind::Course, title: "Data Visualization (Internship)", sub: None,
        issuer: "SIES Graduate School of Technology", date: "Jun 2026", hours: None, url: None, no: None, img: None,
        facts: &[("Built", "\"Adoption of Technology by World\" Power BI dashboard")],
    },
];

pub fn by_id(id: &str) -> Option<&'static Cert> {
    CERTS.iter().find(|c| c.id == id)
}
