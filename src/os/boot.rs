//! Cold boot: a UEFI-style POST, then systemd-style service lines. The wasm
//! size printed here is measured live, not hard-coded.

use crate::data::profile::ME;
use crate::state::{os, Phase};
use crate::util::{dom, store};
use leptos::*;

const OK: &str = r#"[<span class="ok">  OK  </span>]"#;

fn script() -> Vec<String> {
    let mem = dom::wasm_memory_bytes() / 1024.0 / 1024.0;
    let mut v = vec![
        r#"<span class="hl">NishankOS UEFI</span> v2026.3 · Copyright (C) 2026 Nishank Khadpe"#.to_string(),
        "CPU0: Curiosity Core™ @ 9.24 GHz (CGPA boost enabled)".into(),
        format!("Memory: {mem:.1} MiB linear memory mapped · wasm32-unknown-unknown"),
        "Secure Boot: enabled · signature: rustc 1.x + wasm-bindgen".into(),
        String::new(),
        format!("NishankOS 2026.3 (kali-inspired) — kernel 6.x-{}", ME.handle),
    ];
    for s in [
        "Mounted <span class=\"hl\">/home/nishank</span>",
        "Loaded <span class=\"hl\">nishank_os_bg.wasm</span> into the browser sandbox",
        "Started Leptos reactive runtime (no virtual DOM)",
        "Started FastAPI gateway on :8000",
        "Started Redis real-time state (AvIAI)",
        "Indexed BM25 + trigram corpus (Fintrix, in-process)",
        "Started Cypheron scanner service",
        "Spawned CodeLab worker pool (sandboxed)",
        "Reached target <span class=\"hl\">Cyber Security</span>",
        "Reached target <span class=\"hl\">System Design</span>",
        "Reached target <span class=\"hl\">Web Development</span>",
        "Reached target <span class=\"hl\">ML Development</span>",
        "Verified certificate: AgentX 2026 · 2nd Prize",
        "Started Xfce-style desktop session",
    ] {
        v.push(format!("{OK} {s}"));
    }
    v.push(String::new());
    v.push("Welcome, guest. Starting graphical session…".into());
    v
}

#[component]
pub fn Boot() -> impl IntoView {
    let os = os();
    let lines = script();
    let total = lines.len();
    let (shown, set_shown) = create_signal(0usize);
    let (out, set_out) = create_signal(false);
    let lines = store_value(lines);

    let finish = move || {
        if out.get_untracked() {
            return;
        }
        set_shown.set(total);
        set_out.set(true);
        store::set_bool("booted", true);
        set_timeout(
            move || {
                os.phase.set(Phase::Desktop);
                os.toast("spark", "Welcome to NishankOS", "Now running on Rust + WebAssembly. Press Ctrl+K for the command palette.", None);
            },
            std::time::Duration::from_millis(480),
        );
    };

    dom::use_interval(85, move || {
        let n = shown.get_untracked();
        if n < total {
            set_shown.set(n + 1);
        } else {
            finish();
        }
    });
    let key = window_event_listener(ev::keydown, move |_| finish());
    on_cleanup(move || key.remove());

    view! {
        <div id="boot" class:out=out on:click=move |_| finish() aria-hidden="true">
            <div id="bootlog">
                {move || lines.with_value(|l| l[..shown.get().min(total)].iter().map(|s| view! { <div inner_html=s.clone()></div> }).collect_view())}
            </div>
            <div class="boot-bar"><i style:width=move || format!("{}%", shown.get() * 100 / total)></i></div>
            <span class="skip">"click or press any key to skip"</span>
        </div>
    }
}
