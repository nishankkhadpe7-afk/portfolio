//! A small crab who patrols the bottom of the desktop. Rust's unofficial
//! mascot is public domain; this one is drawn from scratch in SVG.
//! Click for a tip; he changes his mind about where to walk now and then.

use crate::apps::AppId;
use crate::state::os;
use crate::util::{dom, rng::Rng};
use leptos::*;

const TIPS: &[(&str, Option<&str>)] = &[
    ("Ctrl+K opens the command palette. It searches apps, projects, themes — everything.", None),
    ("Drag a window to the top edge to maximise it, or to a side to tile it.", None),
    ("Try `sudo hire-me` in the Terminal.", Some("terminal")),
    ("Beat ICE Breaker to unlock the root theme. Or know a certain old cheat code…", Some("breach")),
    ("Everything you see is Rust compiled to WebAssembly. Even me.", Some("monitor")),
    ("Fintrix's BM25 retriever runs in WASM — type a question and watch the scores.", Some("fintrix")),
    ("The Game of Life wallpaper lets you draw cells with your cursor.", Some("settings")),
    ("`git log` in the Terminal is Nishank's journey, one commit at a time.", Some("terminal")),
    ("CodeLab grades your JavaScript against reference answers written in Rust.", Some("codelab")),
];

#[component]
pub fn Ferris() -> impl IntoView {
    let os = os();
    let (x, set_x) = create_signal(30.0f64); // percent of desk width
    let (dir, set_dir) = create_signal(1.0f64);
    let (tip, set_tip) = create_signal(None::<usize>);
    let (hop, set_hop) = create_signal(false);
    let rng = store_value(Rng::from_time());
    let paused = create_memo(move |_| tip.get().is_some());

    dom::use_raf(move |_, dt| {
        if paused.get_untracked() || !os.motion.get_untracked() {
            return;
        }
        let mut nx = x.get_untracked() + dir.get_untracked() * dt * 2.2;
        let mut flip = !(4.0..=92.0).contains(&nx);
        rng.update_value(|r| flip |= r.chance(dt * 0.06));
        if flip {
            set_dir.update(|d| *d = -*d);
            nx = nx.clamp(4.0, 92.0);
        }
        set_x.set(nx);
    });

    let poke = move |_| {
        set_hop.set(true);
        set_timeout(move || set_hop.set(false), std::time::Duration::from_millis(420));
        let i = rng.try_update_value(|r| r.int(TIPS.len())).unwrap_or(0);
        set_tip.set(Some(i));
        set_timeout(move || set_tip.update(|t| if *t == Some(i) { *t = None }), std::time::Duration::from_millis(6500));
    };

    view! {
        <div class="ferris" class:hop=hop class:walking=move || !paused.get()
            style:left=move || format!("{:.2}%", x.get())
            style:transform=move || format!("scaleX({})", if dir.get() < 0.0 { -1 } else { 1 })>
            {move || tip.get().map(|i| {
                let (text, app) = TIPS[i];
                view! {
                    <div class="f-bubble" style:transform=move || format!("scaleX({})", if dir.get() < 0.0 { -1 } else { 1 })>
                        {text}
                        {app.and_then(AppId::from_key).map(|a| view! {
                            <button on:click=move |e| { e.stop_propagation(); set_tip.set(None); os.open(a, None); }>"show me →"</button>
                        })}
                    </div>
                }
            })}
            <button class="f-body" aria-label="Ferris the crab — click for a tip" on:click=poke>
                <svg viewBox="0 0 64 40" aria-hidden="true">
                    <g class="f-legs" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" fill="none">
                        <path d="M16 28l-7 7M22 30l-4 8M42 30l4 8M48 28l7 7"/>
                    </g>
                    <path class="f-claw l" d="M10 16c-6-1-8-8-3-11 1 3 4 4 6 3-1 3 0 5 2 6z"/>
                    <path class="f-claw r" d="M54 16c6-1 8-8 3-11-1 3-4 4-6 3 1 3 0 5-2 6z"/>
                    <path class="f-shell" d="M8 24c0-8 11-13 24-13s24 5 24 13c0 6-11 8-24 8S8 30 8 24z"/>
                    <path class="f-spikes" d="M12 15l3 3 3-5 3 4 4-6 3 5 4-6 4 6 3-5 4 6 3-4 3 5 3-3"/>
                    <circle cx="26" cy="20" r="3.2" fill="#fff"/><circle cx="38" cy="20" r="3.2" fill="#fff"/>
                    <circle class="f-eye" cx="26.8" cy="20.4" r="1.6"/><circle class="f-eye" cx="38.8" cy="20.4" r="1.6"/>
                    <path d="M28 26q4 3 8 0" stroke="#1a0b05" stroke-width="1.6" fill="none" stroke-linecap="round"/>
                </svg>
            </button>
        </div>
    }
}
