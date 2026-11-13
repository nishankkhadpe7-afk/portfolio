//! Appearance: live wallpaper thumbnails (each one a real, running scene),
//! themes, and effect toggles. Everything persists to localStorage.

use crate::os::icons::Icon;
use crate::os::wallpaper::WallCanvas;
use crate::state::{os, Theme, Wallpaper};
use leptos::*;
use wasm_bindgen::JsCast;

#[component]
fn Toggle(label: &'static str, hint: &'static str, sig: RwSignal<bool>) -> impl IntoView {
    view! {
        <label class="tog">
            <span>{label}<small>{hint}</small></span>
            <input type="checkbox" prop:checked=sig on:change=move |e| sig.set(event_target_checked(&e))/>
        </label>
    }
}

#[component]
pub fn Settings() -> impl IntoView {
    let os = os();
    let upload = move |e: ev::Event| {
        let Some(input) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) else { return };
        let Some(file) = input.files().and_then(|f| f.get(0)) else { return };
        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
            if let Some(old) = os.custom_wall.get_untracked() {
                let _ = web_sys::Url::revoke_object_url(&old);
            }
            os.custom_wall.set(Some(url));
            os.wallpaper.set(Wallpaper::Custom);
        }
    };
    view! {
        <div class="st">
            <section>
                <div class="eyebrow">"wallpaper · rendered live by Rust"</div>
                <div class="wp-grid">
                    {Wallpaper::BUILTIN.map(|w| view! {
                        <button class="wp-tile" class:on=move || os.wallpaper.get() == w on:click=move |_| os.wallpaper.set(w)>
                            <div class="wp-prev"><WallCanvas kind=Signal::derive(move || w) preview=true/></div>
                            <span><span><b>{w.name()}</b><small>{w.blurb()}</small></span><i class="ck"><Icon name="check"/></i></span>
                        </button>
                    }).collect_view()}
                    <label class="wp-tile" class:on=move || os.wallpaper.get() == Wallpaper::Custom>
                        <div class="wp-up" style:background-image=move || os.custom_wall.get().map(|u| format!("url(\"{u}\")")).unwrap_or_default()>
                            <Icon name="upload"/><b>"Your image"</b>"JPG · PNG · WebP"
                        </div>
                        <input type="file" accept="image/*" class="sr" on:change=upload/>
                        <span><span><b>"Custom"</b><small>"stays on your device"</small></span><i class="ck"><Icon name="check"/></i></span>
                    </label>
                </div>
            </section>
            <section>
                <div class="eyebrow">"theme"</div>
                <div class="th-grid">
                    {Theme::ALL.map(|t| {
                        let locked = move || t == Theme::Root && !os.root_unlocked.get();
                        let sw = t.swatch();
                        view! {
                            <button class="th-tile" class:on=move || os.theme.get() == t class:locked=locked
                                disabled=locked
                                title=move || if locked() { "Beat ICE Breaker to unlock".to_string() } else { t.name().to_string() }
                                on:click=move |_| os.theme.set(t)>
                                <span class="th-sw" style:background=sw[0]>
                                    <i style:background=sw[1]></i><i style:background=sw[2]></i>
                                </span>
                                <span>{move || if locked() { "██████ locked".to_string() } else { t.name().to_string() }}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </section>
            <section>
                <div class="eyebrow">"effects"</div>
                <div class="st-rows">
                    <Toggle label="Motion" hint="Animated wallpapers, radar sweep and the walking crab" sig=os.motion/>
                    <Toggle label="CRT scanlines" hint="A subtle phosphor-monitor overlay" sig=os.crt/>
                    <Toggle label="Ferris" hint="The crab who lives at the bottom of the desktop" sig=os.ferris/>
                </div>
            </section>
            <p class="muted st-foot">"Preferences are stored in this browser only. Shortcuts: Ctrl+K palette · F3 overview · Ctrl+Alt+T terminal · Alt+W close window."</p>
        </div>
    }
}
