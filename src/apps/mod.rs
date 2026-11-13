//! The application registry. One enum, one metadata table, one render switch.
//! Adding an app = a variant here + a module + a row in `APPS`.

use crate::state::Win;
use leptos::*;

mod about;
mod aviai;
mod blueprints;
mod breach;
mod certs;
mod codelab;
mod contact;
mod cypheron;
mod fintrix;
mod monitor;
mod packages;
mod projects;
mod settings;
pub mod terminal;

pub use contact::vcard as contact_vcard;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AppId {
    Terminal,
    About,
    Files,
    Cypheron,
    Aviai,
    Fintrix,
    Blueprints,
    Packages,
    Certs,
    Contact,
    CodeLab,
    Settings,
    Monitor,
    Breach,
}

pub struct AppMeta {
    pub id: AppId,
    /// Stable string key, used by `open <key>` and deep links.
    pub key: &'static str,
    pub title: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub w: f64,
    pub h: f64,
    pub desc: &'static str,
    pub on_desktop: bool,
}

const fn m(id: AppId, key: &'static str, title: &'static str, label: &'static str, icon: &'static str, w: f64, h: f64, desc: &'static str, on_desktop: bool) -> AppMeta {
    AppMeta { id, key, title, label, icon, w, h, desc, on_desktop }
}

pub const APPS: &[AppMeta] = &[
    m(AppId::About, "about", "whoami", "whoami", "user", 760.0, 600.0, "Profile, focus areas, journey", true),
    m(AppId::Terminal, "terminal", "Terminal", "Terminal", "terminal", 740.0, 470.0, "zsh · try \"help\"", true),
    m(AppId::Files, "files", "Projects", "Projects", "folder", 880.0, 560.0, "Everything Nishank has shipped", true),
    m(AppId::CodeLab, "codelab", "CodeLab · Practice Problems", "CodeLab", "braces", 1100.0, 660.0, "Write and run JavaScript against Rust-graded tests", true),
    m(AppId::Cypheron, "cypheron", "Cypheron · Web Security Scanner", "Cypheron", "shield", 920.0, 600.0, "Run a demo vulnerability scan", true),
    m(AppId::Aviai, "aviai", "AvIAI · ATC Radar", "AvIAI Radar", "radar", 920.0, 580.0, "Airport digital twin, AI controller", true),
    m(AppId::Fintrix, "fintrix", "Fintrix · Compliance RAG", "Fintrix", "scale", 940.0, 600.0, "BM25 + trigram retrieval, in Rust", true),
    m(AppId::Blueprints, "blueprints", "Blueprints · System Design", "Blueprints", "blueprint", 960.0, 620.0, "Architecture of each project", true),
    m(AppId::Packages, "packages", "Cargo · Installed Crates", "Packages", "package", 780.0, 540.0, "Skills, and where they are used", true),
    m(AppId::Certs, "certs", "Certificates", "Certificates", "award", 920.0, 600.0, "Awards and completed courses", true),
    m(AppId::Monitor, "monitor", "System Monitor", "Monitor", "cpu", 820.0, 540.0, "Live processes, real WASM memory", true),
    m(AppId::Breach, "breach", "ICE Breaker", "ICE Breaker", "spark", 760.0, 560.0, "A breach-protocol puzzle · unlocks root", true),
    m(AppId::Contact, "contact", "Contact", "Contact", "mail", 580.0, 560.0, "Email, LinkedIn, GitHub, vCard", true),
    m(AppId::Settings, "settings", "Appearance", "Appearance", "image", 780.0, 600.0, "Themes, wallpapers, effects", true),
];

impl AppId {
    pub fn meta(self) -> &'static AppMeta {
        APPS.iter().find(|a| a.id == self).expect("every AppId has metadata")
    }

    pub fn from_key(key: &str) -> Option<AppId> {
        let key = key.to_lowercase();
        let alias = match key.as_str() {
            "projects" | "explorer" => "files",
            "whoami" | "profile" => "about",
            "top" | "htop" => "monitor",
            "game" | "ice" => "breach",
            "appearance" | "theme" => "settings",
            "skills" | "cargo" => "packages",
            "radar" => "aviai",
            k => k,
        };
        APPS.iter().find(|a| a.key == alias).map(|a| a.id)
    }
}

/// Build the body of an application window.
pub fn render(win: Win) -> View {
    match win.app {
        AppId::Terminal => view! { <terminal::Terminal win/> }.into_view(),
        AppId::About => view! { <about::About/> }.into_view(),
        AppId::Files => view! { <projects::Projects win/> }.into_view(),
        AppId::Cypheron => view! { <cypheron::Cypheron win/> }.into_view(),
        AppId::Aviai => view! { <aviai::Aviai/> }.into_view(),
        AppId::Fintrix => view! { <fintrix::Fintrix/> }.into_view(),
        AppId::Blueprints => view! { <blueprints::Blueprints win/> }.into_view(),
        AppId::Packages => view! { <packages::Packages/> }.into_view(),
        AppId::Certs => view! { <certs::Certs win/> }.into_view(),
        AppId::Contact => view! { <contact::Contact win/> }.into_view(),
        AppId::CodeLab => view! { <codelab::CodeLab win/> }.into_view(),
        AppId::Settings => view! { <settings::Settings/> }.into_view(),
        AppId::Monitor => view! { <monitor::Monitor/> }.into_view(),
        AppId::Breach => view! { <breach::Breach/> }.into_view(),
    }
}
