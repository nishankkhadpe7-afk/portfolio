#![allow(clippy::too_many_arguments, clippy::type_complexity)]
//! # NishankOS
//!
//! Nishank Khadpe's portfolio, rebuilt as a Kali-inspired desktop operating
//! system that runs entirely in the browser. Everything you see — the window
//! manager, the wallpapers, the radar, the retrieval engine, the shell — is
//! Rust compiled to `wasm32-unknown-unknown` and rendered with Leptos.
//!
//! ```text
//! main.rs ─► app.rs (root: boot → desktop ⇄ lock, global hotkeys)
//!              ├─ state.rs      the OS kernel: windows, z-order, theme, toasts
//!              ├─ os/           shell chrome: panel, desktop, windows, overlays
//!              ├─ apps/         one module per application
//!              ├─ data/         every fact about Nishank lives here, typed
//!              └─ util/         rng, DOM glue, animation loops, persistence
//! ```

mod app;
mod apps;
mod data;
mod os;
mod state;
mod util;

fn main() {
    console_error_panic_hook::set_once();
    // Remove the static splash that index.html shows while the .wasm downloads.
    if let Some(el) = leptos::document().get_element_by_id("splash") {
        el.remove();
    }
    leptos::mount_to_body(app::App);
}
