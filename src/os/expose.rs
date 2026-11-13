//! Overview of every open window (F3). Cards are mini schematic previews
//! drawn at each window's real relative position and size.

use super::icons::Icon;
use crate::state::os;
use leptos::*;

#[component]
pub fn Expose() -> impl IntoView {
    let os = os();
    view! {
        <div class="expose" on:pointerdown=move |_| os.expose.set(false)>
            <div class="ex-head"><b>"Overview"</b><span>{move || format!("{} window(s) · Esc to close", os.wins.get().len())}</span></div>
            <div class="ex-grid">
                <For each=move || os.wins.get() key=|w| w.app let:w>
                    {
                        let m = w.app.meta();
                        view! {
                            <button class="ex-card" class:on=move || os.focused.get() == Some(w.app)
                                on:pointerdown=|e| e.stop_propagation()
                                on:click=move |_| { w.min.set(false); os.focus(w.app); os.expose.set(false); }>
                                <div class="ex-mini">
                                    <i style:left=move || { let (dw, _) = os.desk.get(); format!("{}%", (w.x.get() / dw * 100.0).clamp(0.0, 80.0)) }
                                       style:top=move || { let (_, dh) = os.desk.get(); format!("{}%", (w.y.get() / dh * 100.0).clamp(0.0, 80.0)) }
                                       style:width=move || { let (dw, _) = os.desk.get(); format!("{}%", (w.w.get() / dw * 100.0).clamp(15.0, 100.0)) }
                                       style:height=move || { let (_, dh) = os.desk.get(); format!("{}%", (w.h.get() / dh * 100.0).clamp(15.0, 100.0)) }>
                                        <Icon name=m.icon/>
                                    </i>
                                </div>
                                <span><Icon name=m.icon/>{m.label}{move || if w.min.get() { " · minimised" } else { "" }}</span>
                                <em role="button" aria-label="Close window" on:click=move |e| { e.stop_propagation(); os.close(w.app); }><Icon name="close"/></em>
                            </button>
                        }
                    }
                </For>
            </div>
            <Show when=move || os.wins.get().is_empty()>
                <p class="ex-empty">"Nothing open. Double-click an icon, or press Ctrl+K."</p>
            </Show>
        </div>
    }
}
