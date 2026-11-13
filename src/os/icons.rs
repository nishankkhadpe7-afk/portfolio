//! The icon set: 24×24 stroke icons, emitted once as an SVG sprite and
//! referenced everywhere with `<use>` — one DOM node per icon instance.

use leptos::*;

pub const ICONS: &[(&str, &str)] = &[
    ("terminal", r#"<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 9l3 3-3 3M12.5 15h4.5"/>"#),
    ("user", r#"<circle cx="12" cy="8" r="4"/><path d="M4 20c1.5-4 4.5-6 8-6s6.5 2 8 6"/>"#),
    ("folder", r#"<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>"#),
    ("shield", r#"<path d="M12 3l8 3v6c0 4.5-3.4 8-8 9-4.6-1-8-4.5-8-9V6z"/><path d="M9 12l2 2 4-4"/>"#),
    ("radar", r#"<circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="4.5"/><path d="M12 12l6.3-6.3"/><circle cx="15.5" cy="15" r=".9" fill="currentColor"/>"#),
    ("scale", r#"<path d="M12 4v16M7 20h10M5 8h14M5 8l-2.5 6a3 3 0 0 0 5 0zM19 8l-2.5 6a3 3 0 0 0 5 0z"/>"#),
    ("blueprint", r#"<rect x="3" y="4" width="6" height="5" rx="1"/><rect x="15" y="4" width="6" height="5" rx="1"/><rect x="9" y="15" width="6" height="5" rx="1"/><path d="M6 9v3h12V9M12 12v3"/>"#),
    ("package", r#"<path d="M12 3l8 4.5v9L12 21l-8-4.5v-9z"/><path d="M4 7.5l8 4.5 8-4.5M12 12v9"/>"#),
    ("award", r#"<circle cx="12" cy="9" r="5.5"/><path d="M9 14l-1.2 7L12 19l4.2 2-1.2-7"/>"#),
    ("mail", r#"<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M3 7l9 6 9-6"/>"#),
    ("phone", r#"<path d="M5 4h4l2 5-2.5 1.5a11 11 0 0 0 5 5L15 13l5 2v4a2 2 0 0 1-2 2A16 16 0 0 1 3 6a2 2 0 0 1 2-2"/>"#),
    ("link", r#"<path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"/>"#),
    ("code", r#"<path d="M8 7l-5 5 5 5M16 7l5 5-5 5M13.5 4l-3 16"/>"#),
    ("pin", r#"<path d="M12 21s7-6.2 7-12a7 7 0 0 0-14 0c0 5.8 7 12 7 12z"/><circle cx="12" cy="9" r="2.5"/>"#),
    ("power", r#"<path d="M12 3v8M7 6.3a8 8 0 1 0 10 0"/>"#),
    ("search", r#"<circle cx="11" cy="11" r="6"/><path d="M20 20l-4.5-4.5"/>"#),
    ("lock", r#"<rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/>"#),
    ("min", r#"<path d="M6 12h12"/>"#),
    ("max", r#"<rect x="6" y="6" width="12" height="12" rx="1"/>"#),
    ("restore", r#"<rect x="5" y="9" width="10" height="10" rx="1"/><path d="M9 9V6a1 1 0 0 1 1-1h8a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1h-3"/>"#),
    ("close", r#"<path d="M7 7l10 10M17 7L7 17"/>"#),
    ("wifi", r#"<path d="M2.5 9a14 14 0 0 1 19 0M5.5 12.5a9.5 9.5 0 0 1 13 0M8.8 16a5 5 0 0 1 6.4 0"/><circle cx="12" cy="19.2" r=".9" fill="currentColor"/>"#),
    ("cpu", r#"<rect x="6" y="6" width="12" height="12" rx="2"/><path d="M9 3v3M15 3v3M9 18v3M15 18v3M3 9h3M3 15h3M18 9h3M18 15h3"/><rect x="9.5" y="9.5" width="5" height="5"/>"#),
    ("globe", r#"<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"/>"#),
    ("brain", r#"<path d="M9 4a3 3 0 0 0-3 3 3 3 0 0 0-2 5 3 3 0 0 0 2 5 3 3 0 0 0 6 1V5a2 2 0 0 0-3-1zM15 4a3 3 0 0 1 3 3 3 3 0 0 1 2 5 3 3 0 0 1-2 5 3 3 0 0 1-6 1"/>"#),
    ("play", r#"<path d="M7 5l12 7-12 7z"/>"#),
    ("spark", r#"<path d="M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8zM19 16l.8 2.2L22 19l-2.2.8L19 22l-.8-2.2L16 19l2.2-.8z"/>"#),
    ("chart", r#"<path d="M4 4v16h16"/><path d="M8 15l3.5-4 3 2.5L20 7"/>"#),
    ("check", r#"<path d="M5 12.5l4.5 4.5L19 7"/>"#),
    ("reset", r#"<path d="M4 12a8 8 0 1 0 2.4-5.7M4 4v4h4"/>"#),
    ("upload", r#"<path d="M12 16V4M7 9l5-5 5 5M5 20h14"/>"#),
    ("download", r#"<path d="M12 4v12M7 11l5 5 5-5M5 20h14"/>"#),
    ("image", r#"<rect x="3" y="5" width="18" height="14" rx="2"/><circle cx="9" cy="10" r="1.8"/><path d="M21 16l-5-5-9 8"/>"#),
    ("braces", r#"<path d="M8 4H7a2 2 0 0 0-2 2v4l-2 2 2 2v4a2 2 0 0 0 2 2h1M16 4h1a2 2 0 0 1 2 2v4l2 2-2 2v4a2 2 0 0 1-2 2h-1"/><path d="M10 15l4-6"/>"#),
    ("ext", r#"<path d="M14 4h6v6M20 4l-9 9M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5"/>"#),
    ("grid", r#"<rect x="4" y="4" width="7" height="7" rx="1.5"/><rect x="13" y="4" width="7" height="7" rx="1.5"/><rect x="4" y="13" width="7" height="7" rx="1.5"/><rect x="13" y="13" width="7" height="7" rx="1.5"/>"#),
    ("command", r#"<path d="M9 6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3z"/>"#),
    ("droplet", r#"<path d="M12 3.5s6 6.4 6 10.5a6 6 0 0 1-12 0c0-4.1 6-10.5 6-10.5z"/>"#),
    ("copy", r#"<rect x="8" y="8" width="12" height="12" rx="2"/><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/>"#),
    ("flag", r#"<path d="M5 21V4M5 4h11l-2 4 2 4H5"/>"#),
    ("git", r#"<circle cx="6" cy="6" r="2"/><circle cx="6" cy="18" r="2"/><circle cx="18" cy="12" r="2"/><path d="M6 8v8M8 6c6 0 10 2 10 4"/>"#),
    ("bolt", r#"<path d="M13 3L5 14h6l-1 7 8-11h-6z"/>"#),
    ("crab", r#"<path d="M6 13c0-3 2.7-5 6-5s6 2 6 5-2.7 4-6 4-6-1-6-4z"/><path d="M4 9l2 2M20 9l-2 2M4 6l1 3M20 6l-1 3M7 17l-2 3M17 17l2 3M10 17l-1 3M14 17l1 3"/>"#),
];

/// Render once, near the root.
#[component]
pub fn IconSprite() -> impl IntoView {
    let symbols: String = ICONS
        .iter()
        .map(|(id, body)| format!(r#"<symbol id="i-{id}" viewBox="0 0 24 24">{body}</symbol>"#))
        .collect();
    view! { <svg class="sprite" aria-hidden="true" inner_html=symbols></svg> }
}

#[component]
pub fn Icon(name: &'static str, #[prop(optional)] class: &'static str) -> impl IntoView {
    let cls = if class.is_empty() { "ic".to_string() } else { format!("ic {class}") };
    view! {
        <svg class=cls aria-hidden="true" inner_html=format!(r##"<use href="#i-{name}"></use>"##)></svg>
    }
}

/// The NishankOS hexagon sigil (an N inside a hex).
pub const SIGIL: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" stroke-linecap="round"><path d="M12 1.8l9 5.1v10.2l-9 5.1-9-5.1V6.9z"/><path d="M8.2 16.2V7.8l7.6 8.4V7.8"/><path d="M12 1.8v2.4M12 19.8v2.4" opacity=".55"/></svg>"#;

#[component]
pub fn Sigil(#[prop(optional)] class: &'static str) -> impl IntoView {
    let cls = if class.is_empty() { "av-hex".to_string() } else { format!("av-hex {class}") };
    view! { <span class=cls inner_html=SIGIL></span> }
}
