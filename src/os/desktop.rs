//! The desktop surface: wallpaper, icons, windows and every overlay.

use super::context_menu::ContextMenu;
use super::expose::Expose;
use super::ferris::Ferris;
use super::icons::{Icon, Sigil};
use super::menu::Menu;
use super::palette::Palette;
use super::panel::Panel;
use super::toasts::Toasts;
use super::wallpaper::WallCanvas;
use super::window::Window;
use crate::apps::APPS;
use crate::state::{os, Snap, Wallpaper};
use crate::util::dom::{self, POINTER};
use leptos::*;

#[component]
pub fn Shell() -> impl IntoView {
    let os = os();
    let desk = create_node_ref::<html::Main>();

    let measure = move || {
        if let Some(el) = desk.get_untracked() {
            let r = el.get_bounding_client_rect();
            os.desk.set((r.width(), r.height()));
        }
    };
    desk.on_load(move |_| request_animation_frame(measure));
    let rs = window_event_listener(ev::resize, move |_| measure());
    on_cleanup(move || rs.remove());

    let custom_bg = move || match (os.wallpaper.get(), os.custom_wall.get()) {
        (Wallpaper::Custom, Some(url)) => format!("url(\"{url}\")"),
        _ => String::new(),
    };

    view! {
        <div id="os">
            <Panel/>
            <main id="desk" node_ref=desk
                class:custom=move || os.wallpaper.get() == Wallpaper::Custom
                style:background-image=custom_bg
                on:pointermove=move |e| POINTER.with(|p| p.set((e.client_x() as f64, e.client_y() as f64)))
                on:pointerleave=move |_| POINTER.with(|p| p.set((-9999.0, -9999.0)))
                on:pointerdown=move |_| { os.menu.set(false); os.ctx.set(None); }
                on:contextmenu=move |e| {
                    e.prevent_default();
                    let (dw, dh) = os.desk.get_untracked();
                    let x = (e.client_x() as f64).min(dw - 230.0);
                    let y = (e.client_y() as f64 - crate::state::PANEL_H).min(dh - 300.0);
                    os.ctx.set(Some((x.max(4.0), y.max(4.0))));
                }
            >
                <WallCanvas kind=os.wallpaper/>
                <Sigil class="sigil-bg"/>
                <div class="wall-tag" aria-hidden="true">
                    <b>"NISHANK OS"</b>"the quieter you become, the more you are able to hear"
                </div>
                <DesktopIcons/>
                <For each=move || os.wins.get() key=|w| w.app let:w>
                    <Window win=w/>
                </For>
                <div class=move || match os.snap_hint.get() {
                    Some(Snap::Max) => "snap-ghost on max",
                    Some(Snap::Left) => "snap-ghost on l",
                    Some(Snap::Right) => "snap-ghost on r",
                    None => "snap-ghost",
                }></div>
                <Show when=move || os.ferris.get()><Ferris/></Show>
                <Toasts/>
                <Show when=move || os.menu.get()><Menu/></Show>
                <ContextMenu/>
                <Show when=move || os.expose.get()><Expose/></Show>
                <Show when=move || os.palette.get()><Palette/></Show>
            </main>
        </div>
    }
}

#[component]
fn DesktopIcons() -> impl IntoView {
    let os = os();
    view! {
        <div class="icons" role="list">
            {APPS.iter().filter(|a| a.on_desktop).map(|a| {
                let id = a.id;
                view! {
                    <button class="dicon" role="listitem" title=a.desc
                        on:dblclick=move |_| os.open(id, None)
                        on:click=move |_| if dom::is_mobile() { os.open(id, None) }
                        on:keydown=move |e| if e.key() == "Enter" { os.open(id, None) }>
                        <Icon name=a.icon/>
                        <span>{a.label}</span>
                    </button>
                }
            }).collect_view()}
        </div>
    }
}
