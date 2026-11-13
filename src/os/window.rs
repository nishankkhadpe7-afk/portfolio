//! A window frame: title bar, drag, 8-way resize, edge snapping.
//!
//! Geometry lives in four signals on `Win`; dragging writes them and Leptos
//! patches exactly four inline style properties — no re-render of the body.

use super::icons::Icon;
use crate::apps;
use crate::state::{os, Snap, Win, PANEL_H};
use crate::util::dom;
use leptos::ev::PointerEvent;
use leptos::*;
use leptos::leptos_dom::helpers::WindowListenerHandle;
use wasm_bindgen::JsCast;

const MIN_W: f64 = 320.0;
const MIN_H: f64 = 220.0;
const EDGES: [&str; 8] = ["n", "s", "e", "w", "ne", "nw", "se", "sw"];

fn from_button(e: &PointerEvent) -> bool {
    e.target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        .and_then(|el| el.closest("button").ok().flatten())
        .is_some()
}

#[component]
pub fn Window(win: Win) -> impl IntoView {
    let os = os();
    let app = win.app;
    let meta = app.meta();
    let listeners = store_value(Vec::<WindowListenerHandle>::new());
    let (moving, set_moving) = create_signal(false);

    let release = move || {
        listeners.update_value(|v| v.drain(..).for_each(|h| h.remove()));
        set_moving.set(false);
    };
    on_cleanup(move || listeners.update_value(|v| v.drain(..).for_each(|h| h.remove())));

    // ── drag by the title bar ─────────────────────────────────────────
    let start_drag = move |e: PointerEvent| {
        if e.button() != 0 || from_button(&e) || dom::is_mobile() {
            return;
        }
        e.prevent_default();
        os.focus(app);
        let (px, py) = (e.client_x() as f64, e.client_y() as f64);
        if win.snap.get_untracked().is_some() {
            // Un-snap so the cursor keeps the same relative spot on the bar.
            let (dw, _) = os.desk.get_untracked();
            let w = win.w.get_untracked();
            win.snap.set(None);
            win.x.set(px - w * (px / dw));
            win.y.set(py - PANEL_H - 16.0);
        }
        let (x0, y0, w0) = (win.x.get_untracked(), win.y.get_untracked(), win.w.get_untracked());
        set_moving.set(true);
        let mv = window_event_listener(ev::pointermove, move |e| {
            let (cx, cy) = (e.client_x() as f64, e.client_y() as f64);
            let (dw, dh) = os.desk.get_untracked();
            win.x.set((x0 + cx - px).clamp(80.0 - w0, dw - 80.0));
            win.y.set((y0 + cy - py).clamp(0.0, dh - 36.0));
            let dy = cy - PANEL_H;
            os.snap_hint.set(if dy < 4.0 {
                Some(Snap::Max)
            } else if cx < 4.0 {
                Some(Snap::Left)
            } else if cx > dw - 4.0 {
                Some(Snap::Right)
            } else {
                None
            });
        });
        let up = window_event_listener(ev::pointerup, move |_| {
            if let Some(s) = os.snap_hint.get_untracked() {
                win.snap.set(Some(s));
            }
            os.snap_hint.set(None);
            release();
        });
        listeners.update_value(|v| v.extend([mv, up]));
    };

    // ── resize from any edge or corner ────────────────────────────────
    let start_resize = move |dir: &'static str, e: PointerEvent| {
        if e.button() != 0 || win.snap.get_untracked().is_some() {
            return;
        }
        e.prevent_default();
        e.stop_propagation();
        os.focus(app);
        let (px, py) = (e.client_x() as f64, e.client_y() as f64);
        let (x0, y0, w0, h0) = (win.x.get_untracked(), win.y.get_untracked(), win.w.get_untracked(), win.h.get_untracked());
        set_moving.set(true);
        let mv = window_event_listener(ev::pointermove, move |e| {
            let (dx, dy) = (e.client_x() as f64 - px, e.client_y() as f64 - py);
            let (dw, dh) = os.desk.get_untracked();
            if dir.contains('e') {
                win.w.set((w0 + dx).clamp(MIN_W, dw - x0));
            }
            if dir.contains('s') {
                win.h.set((h0 + dy).clamp(MIN_H, dh - y0));
            }
            if dir.contains('w') {
                let w = (w0 - dx).clamp(MIN_W, x0 + w0);
                win.x.set(x0 + w0 - w);
                win.w.set(w);
            }
            if dir.contains('n') {
                let h = (h0 - dy).clamp(MIN_H, y0 + h0);
                win.y.set(y0 + h0 - h);
                win.h.set(h);
            }
        });
        let up = window_event_listener(ev::pointerup, move |_| release());
        listeners.update_value(|v| v.extend([mv, up]));
    };

    view! {
        <section
            class="win"
            class:max=move || win.snap.get() == Some(Snap::Max)
            class:snap-l=move || win.snap.get() == Some(Snap::Left)
            class:snap-r=move || win.snap.get() == Some(Snap::Right)
            class:focused=move || os.focused.get() == Some(app)
            class:min=move || win.min.get()
            class:moving=moving
            style:left=move || format!("{}px", win.x.get())
            style:top=move || format!("{}px", win.y.get())
            style:width=move || format!("{}px", win.w.get())
            style:height=move || format!("{}px", win.h.get())
            style:z-index=move || win.z.get().to_string()
            role="dialog"
            aria-label=meta.title
            on:pointerdown=move |_| os.focus(app)
        >
            <div class="tb" on:pointerdown=start_drag on:dblclick=move |_| os.toggle_max(app)>
                <span class="tb-ic"><Icon name=meta.icon/></span>
                <span class="tb-t">{meta.title}</span>
                <div class="tb-b">
                    <button title="Minimise" aria-label="Minimise" on:click=move |_| os.minimize(app)><Icon name="min"/></button>
                    <button title="Maximise" aria-label="Maximise" on:click=move |_| os.toggle_max(app)>
                        {move || if win.snap.get().is_some() { view! { <Icon name="restore"/> } } else { view! { <Icon name="max"/> } }}
                    </button>
                    <button class="x" title="Close" aria-label="Close" on:click=move |_| os.close(app)><Icon name="close"/></button>
                </div>
            </div>
            <div class="wb">{apps::render(win)}</div>
            {EDGES.into_iter().map(|d| view! {
                <div class=format!("rz rz-{d}") on:pointerdown=move |e| start_resize(d, e)></div>
            }).collect_view()}
        </section>
    }
}
