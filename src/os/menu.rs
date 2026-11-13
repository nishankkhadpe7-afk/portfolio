//! The applications menu (top-left), with fuzzy search.

use super::icons::{Icon, Sigil};
use crate::apps::{AppId, APPS};
use crate::data::profile::ME;
use crate::state::{os, Phase};
use crate::util::fuzzy;
use leptos::*;

#[component]
pub fn Menu() -> impl IntoView {
    let os = os();
    let (q, set_q) = create_signal(String::new());
    let input = create_node_ref::<html::Input>();
    // Focus once the element is actually in the document.
    input.on_load(|el| {
        request_animation_frame(move || {
            let _ = el.focus();
        });
    });

    let items = move || {
        let q = q.get();
        let mut v: Vec<(i32, &'static crate::apps::AppMeta)> = APPS
            .iter()
            .filter_map(|a| fuzzy::score(&q, &format!("{} {} {}", a.label, a.title, a.desc)).map(|s| (s, a)))
            .collect();
        if !q.is_empty() {
            v.sort_by_key(|x| std::cmp::Reverse(x.0));
        }
        v.into_iter().map(|(_, a)| a).collect::<Vec<_>>()
    };
    let first = move || items().first().map(|a| a.id);

    view! {
        <div id="menu" role="menu" on:pointerdown=|e| e.stop_propagation()>
            <div class="m-head">
                <Sigil/>
                <div class="who"><b>{ME.name}</b><span>{ME.role}</span></div>
            </div>
            <label class="m-search">
                <Icon name="search"/>
                <input node_ref=input placeholder="Search applications…" aria-label="Search applications"
                    prop:value=q
                    on:input=move |e| set_q.set(event_target_value(&e))
                    on:keydown=move |e| if e.key() == "Enter" { if let Some(a) = first() { os.open(a, None) } }/>
            </label>
            <div class="m-list">
                {move || items().into_iter().map(|a| {
                    let id: AppId = a.id;
                    view! {
                        <button class="m-item" role="menuitem" on:click=move |_| os.open(id, None)>
                            <Icon name=a.icon/>
                            <span><b>{a.label}</b><small>{a.desc}</small></span>
                        </button>
                    }
                }).collect_view()}
            </div>
            <div class="m-foot">
                <button on:click=move |_| { os.menu.set(false); os.palette.set(true); }><Icon name="command"/>"Palette"<kbd>"Ctrl K"</kbd></button>
                <button on:click=move |_| { os.menu.set(false); os.phase.set(Phase::Locked); }><Icon name="lock"/>"Lock"</button>
                <button on:click=move |_| os.reboot()><Icon name="power"/>"Reboot"</button>
            </div>
        </div>
    }
}
