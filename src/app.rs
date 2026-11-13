//! Root component: provides the kernel, drives the boot → desktop ⇄ lock
//! state machine, and owns the global keyboard shortcuts.

use crate::apps::AppId;
use crate::os::{boot::Boot, desktop::Shell, icons::IconSprite, lock::Lock};
use crate::state::{Os, Phase, Theme};
use leptos::*;

const KONAMI: [&str; 10] = [
    "ArrowUp", "ArrowUp", "ArrowDown", "ArrowDown", "ArrowLeft", "ArrowRight", "ArrowLeft", "ArrowRight", "b", "a",
];

#[component]
pub fn App() -> impl IntoView {
    let os = Os::new();
    provide_context(os);

    // ── global hotkeys ───────────────────────────────────────────────
    let seq = store_value(Vec::<String>::new());
    let keys = window_event_listener(ev::keydown, move |e| {
        if os.phase.get_untracked() != Phase::Desktop {
            return;
        }
        let k = e.key();
        let (ctrl, alt) = (e.ctrl_key() || e.meta_key(), e.alt_key());

        seq.update_value(|s| {
            s.push(k.clone());
            if s.len() > KONAMI.len() {
                s.remove(0);
            }
        });
        if seq.with_value(|s| s.iter().map(String::as_str).eq(KONAMI.iter().copied())) {
            seq.update_value(Vec::clear);
            os.unlock_root();
            os.theme.set(Theme::Root);
            os.toast("bolt", "↑↑↓↓←→←→BA", "Thirty lives granted. Root theme engaged.", None);
        }

        match (ctrl, alt, k.as_str()) {
            (true, _, "k") | (true, _, "K") | (true, _, " ") => {
                e.prevent_default();
                os.palette.update(|p| *p = !*p);
            }
            (true, true, "t") | (true, true, "T") => {
                e.prevent_default();
                os.open(AppId::Terminal, None);
            }
            (_, _, "F3") | (true, false, "ArrowUp") => {
                e.prevent_default();
                os.expose.update(|x| *x = !*x);
            }
            (false, true, "w") | (false, true, "W") => {
                e.prevent_default();
                os.close_focused();
            }
            (false, true, "`") => {
                // cycle focus through open windows
                let next = os.wins.with_untracked(|v| {
                    let mut v: Vec<_> = v.iter().map(|w| (w.z.get_untracked(), w.app)).collect();
                    v.sort_by_key(|p| p.0);
                    v.first().map(|p| p.1)
                });
                if let Some(a) = next {
                    os.open(a, None);
                }
            }
            (_, _, "Escape") => {
                os.palette.set(false);
                os.menu.set(false);
                os.expose.set(false);
                os.ctx.set(None);
            }
            _ => {}
        }
    });
    on_cleanup(move || keys.remove());

    view! {
        <IconSprite/>
        <Show when=move || os.phase.get() == Phase::Boot><Boot/></Show>
        <Show when=move || os.phase.get() != Phase::Boot><Shell/></Show>
        <Show when=move || os.phase.get() == Phase::Locked><Lock/></Show>
        <div class="crt" class:on=move || os.crt.get() aria-hidden="true"></div>
    }
}
