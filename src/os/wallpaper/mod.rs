//! The wallpaper engine: a canvas driven by a Rust `requestAnimationFrame`
//! loop. The same component renders the desktop background and the live
//! thumbnails in Appearance (`preview = true`, lower DPR and frame rate).

mod scenes;

use crate::state::{os, Theme, Wallpaper};
use crate::util::dom::{self, FPS, POINTER};
use leptos::*;
use scenes::{Frame, Scene};
use std::cell::RefCell;
use std::rc::Rc;

#[component]
pub fn WallCanvas(#[prop(into)] kind: Signal<Wallpaper>, #[prop(optional)] preview: bool) -> impl IntoView {
    let os = os();
    let canvas = create_node_ref::<html::Canvas>();
    let scene: Rc<RefCell<Option<(Wallpaper, (u32, u32), Theme, Scene)>>> = Rc::new(RefCell::new(None));
    let started = dom::now();
    let mut acc = 0.0;
    let mut fps_avg = 60.0;

    dom::use_raf(move |now, dt| {
            let Some(cv) = canvas.get_untracked() else { return };
            // Thumbnails tick at ~30 fps; a paused desktop renders one frame then idles.
            acc += dt;
            if preview && acc < 1.0 / 30.0 {
                return;
            }
            let step = if preview { acc } else { dt };
            acc = 0.0;
            if !preview && dt > 0.0 {
                fps_avg = fps_avg * 0.95 + (1.0 / dt).min(240.0) * 0.05;
                FPS.with(|f| f.set(fps_avg));
            }
            let k = kind.get_untracked();
            if k == Wallpaper::Custom {
                return;
            }
            let motion = os.motion.get_untracked();
            let (w, h, _) = dom::fit_canvas(&cv, if preview { 1.0 } else { 2.0 });
            if w < 2.0 || h < 2.0 {
                return;
            }
            let Some(cx) = dom::ctx2d(&cv) else { return };
            let size = (w as u32, h as u32);
            let theme = os.theme.get_untracked();
            let mut slot = scene.borrow_mut();
            let fresh = !matches!(&*slot, Some((sk, ss, st, _)) if *sk == k && *ss == size && *st == theme);
            if fresh {
                *slot = Some((k, size, theme, Scene::new(k, w, h, preview)));
            } else if !motion {
                return; // still frame already on screen
            }
            let (acc_c, ice_c) = theme.rgb();
            let pointer = if preview {
                None
            } else {
                let r = cv.get_bounding_client_rect();
                let (px, py) = POINTER.with(|p| p.get());
                Some((px - r.left(), py - r.top()))
            };
            let bg = hex_rgb(theme.swatch()[0]);
            if let Some((_, _, _, sc)) = slot.as_mut() {
                sc.frame(&Frame {
                    cx: &cx,
                    w,
                    h,
                    t: (now - started) / 1000.0,
                    dt: if motion { step } else { 0.0 },
                    acc: acc_c,
                    ice: ice_c,
                    bg,
                    pointer,
                });
            }
        });

    view! { <canvas node_ref=canvas class="wall" class:preview=preview aria-hidden="true"></canvas> }
}

fn hex_rgb(h: &str) -> (u8, u8, u8) {
    let v = u32::from_str_radix(h.trim_start_matches('#'), 16).unwrap_or(0);
    ((v >> 16) as u8, (v >> 8) as u8, v as u8)
}
