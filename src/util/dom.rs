//! Glue between Rust and the browser: animation loops that clean themselves
//! up, intervals tied to component lifetimes, clipboard, downloads.

use leptos::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

thread_local! {
    /// Last known pointer position in viewport coordinates. Written by the
    /// desktop on every pointermove, read by wallpaper scenes each frame.
    /// A thread-local (not a signal) because nothing should re-render on it.
    pub static POINTER: Cell<(f64, f64)> = const { Cell::new((-9999.0, -9999.0)) };
    /// Smoothed frames-per-second measured by the wallpaper loop.
    pub static FPS: Cell<f64> = const { Cell::new(60.0) };
}

/// `performance.now()` in milliseconds.
pub fn now() -> f64 {
    window().performance().map(|p| p.now()).unwrap_or_else(js_sys::Date::now)
}

pub fn is_mobile() -> bool {
    window()
        .match_media("(max-width:760px)")
        .ok()
        .flatten()
        .map(|m| m.matches())
        .unwrap_or(false)
}

pub fn prefers_reduced_motion() -> bool {
    window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .map(|m| m.matches())
        .unwrap_or(false)
}

/// Bytes of linear memory currently owned by this wasm module — a real
/// number, shown in the System Monitor and neofetch.
pub fn wasm_memory_bytes() -> f64 {
    let mem: js_sys::WebAssembly::Memory = wasm_bindgen::memory().unchecked_into();
    let buf: js_sys::ArrayBuffer = mem.buffer().unchecked_into();
    buf.byte_length() as f64
}

/// Start a `requestAnimationFrame` loop. The callback gets `(now_ms, dt_s)`
/// with `dt` clamped so a backgrounded tab does not teleport animations.
/// The loop stops (and frees its closure) when the owning reactive scope is
/// disposed — i.e. when the window or component unmounts.
pub fn use_raf(mut f: impl FnMut(f64, f64) + 'static) {
    let alive = Rc::new(Cell::new(true));
    let slot: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let (a2, s2) = (alive.clone(), slot.clone());
    let mut last: Option<f64> = None;
    *slot.borrow_mut() = Some(Closure::new(move |t: f64| {
        if !a2.get() {
            // Never drop a closure while it is executing: defer to a macrotask.
            let s3 = s2.clone();
            set_timeout(move || drop(s3.borrow_mut().take()), Duration::ZERO);
            return;
        }
        let dt = last.map(|l| ((t - l) / 1000.0).clamp(0.0, 0.05)).unwrap_or(1.0 / 60.0);
        last = Some(t);
        f(t, dt);
        if let Some(cb) = s2.borrow().as_ref() {
            let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }));
    if let Some(cb) = slot.borrow().as_ref() {
        let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
    }
    on_cleanup(move || alive.set(false));
}

/// `setInterval` bound to the current reactive scope.
pub fn use_interval(ms: u64, f: impl Fn() + 'static) {
    if let Ok(h) = set_interval_with_handle(f, Duration::from_millis(ms)) {
        on_cleanup(move || h.clear());
    }
}

/// Copy text to the clipboard; resolves silently on failure.
pub fn copy(text: &str) {
    let _ = window().navigator().clipboard().write_text(text);
}

/// Offer a string as a file download (used for the vCard and scan reports).
pub fn download(filename: &str, mime: &str, body: &str) {
    let parts = js_sys::Array::of1(&JsValue::from_str(body));
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(mime);
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts) else { return };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else { return };
    if let Ok(a) = document().create_element("a") {
        let a: web_sys::HtmlAnchorElement = a.unchecked_into();
        a.set_href(&url);
        a.set_download(filename);
        a.click();
    }
    set_timeout(move || { let _ = web_sys::Url::revoke_object_url(&url); }, Duration::from_secs(2));
}

/// Open an external link in a new tab.
pub fn open_url(url: &str) {
    let _ = window().open_with_url_and_target(url, "_blank");
}

/// Local wall-clock time as `HH:MM` (and optionally seconds).
pub fn clock(seconds: bool) -> String {
    let d = js_sys::Date::new_0();
    if seconds {
        format!("{:02}:{:02}:{:02}", d.get_hours(), d.get_minutes(), d.get_seconds())
    } else {
        format!("{:02}:{:02}", d.get_hours(), d.get_minutes())
    }
}

pub fn date_long() -> String {
    let d = js_sys::Date::new_0();
    let opts = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&opts, &"weekday".into(), &"long".into());
    let _ = js_sys::Reflect::set(&opts, &"day".into(), &"numeric".into());
    let _ = js_sys::Reflect::set(&opts, &"month".into(), &"long".into());
    d.to_locale_date_string("en-IN", &opts).into()
}

/// Size a canvas's backing store to its CSS box × devicePixelRatio.
/// Returns the CSS-pixel size and whether a resize happened.
pub fn fit_canvas(cv: &web_sys::HtmlCanvasElement, max_dpr: f64) -> (f64, f64, bool) {
    let dpr = window().device_pixel_ratio().min(max_dpr).max(1.0);
    let (w, h) = (cv.client_width() as f64, cv.client_height() as f64);
    let (bw, bh) = ((w * dpr) as u32, (h * dpr) as u32);
    let changed = cv.width() != bw || cv.height() != bh;
    if changed {
        cv.set_width(bw.max(1));
        cv.set_height(bh.max(1));
    }
    if let Some(cx) = ctx2d(cv) {
        let _ = cx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
    }
    (w, h, changed)
}

pub fn ctx2d(cv: &web_sys::HtmlCanvasElement) -> Option<web_sys::CanvasRenderingContext2d> {
    cv.get_context("2d").ok().flatten().map(|c| c.unchecked_into())
}
