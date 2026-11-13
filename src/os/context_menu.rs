//! Right-click on the desktop.

use crate::apps::AppId;
use crate::state::{os, Phase};
use leptos::*;

#[component]
pub fn ContextMenu() -> impl IntoView {
    let os = os();
    let act = move |f: Box<dyn Fn()>| {
        move |_| {
            os.ctx.set(None);
            f();
        }
    };
    view! {
        <Show when=move || os.ctx.get().is_some()>
            <div id="ctx" role="menu"
                style:left=move || format!("{}px", os.ctx.get().map(|c| c.0).unwrap_or(0.0))
                style:top=move || format!("{}px", os.ctx.get().map(|c| c.1).unwrap_or(0.0))
                on:pointerdown=|e| e.stop_propagation()>
                <button on:click=act(Box::new(move || os.open(AppId::Terminal, None)))>"Open Terminal"<kbd>"Ctrl Alt T"</kbd></button>
                <button on:click=act(Box::new(move || os.palette.set(true)))>"Command palette"<kbd>"Ctrl K"</kbd></button>
                <button on:click=act(Box::new(move || os.expose.set(true)))>"Show all windows"<kbd>"F3"</kbd></button>
                <hr/>
                <button on:click=act(Box::new(move || os.cycle_wallpaper()))>"Next wallpaper"<kbd>{move || os.wallpaper.get().name()}</kbd></button>
                <button on:click=act(Box::new(move || os.cycle_theme()))>"Next theme"<kbd>{move || os.theme.get().name()}</kbd></button>
                <button on:click=act(Box::new(move || os.crt.update(|c| *c = !*c)))>"CRT scanlines"<kbd>{move || if os.crt.get() { "on" } else { "off" }}</kbd></button>
                <button on:click=act(Box::new(move || os.open(AppId::Settings, None)))>"Appearance…"</button>
                <hr/>
                <button on:click=act(Box::new(move || os.toast("cpu", "NishankOS 2026.3", "Rust → wasm32-unknown-unknown · Leptos · Canvas 2D · zero JS frameworks.", Some(AppId::Monitor))))>"About NishankOS"</button>
                <button on:click=act(Box::new(move || os.phase.set(Phase::Locked)))>"Lock screen"</button>
            </div>
        </Show>
    }
}
