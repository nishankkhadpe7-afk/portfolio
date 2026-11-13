//! Projects: a file-explorer style browser over `data::projects`.

use crate::apps::AppId;
use crate::data::projects::PROJECTS;
use crate::os::icons::Icon;
use crate::state::{os, Win};
use leptos::*;

#[component]
pub fn Projects(win: Win) -> impl IntoView {
    let os = os();
    let (sel, set_sel) = create_signal(PROJECTS[0].id);
    create_effect(move |_| {
        if let Some(id) = win.arg.get() {
            if let Some(p) = PROJECTS.iter().find(|p| p.id == id) {
                set_sel.set(p.id);
            }
            win.arg.set(None);
        }
    });
    let current = move || PROJECTS.iter().find(|p| p.id == sel.get()).unwrap_or(&PROJECTS[0]);

    view! {
        <div class="split">
            <aside class="side">
                <h5>"~/projects"</h5>
                {PROJECTS.iter().map(|p| view! {
                    <button class="row-btn" class:sel=move || sel.get() == p.id on:click=move |_| set_sel.set(p.id)>
                        <Icon name=p.icon/>
                        <span><b>{p.name}{p.live.map(|_| view! { <i class="live-dot" title="live"></i> })}</b><small>{format!("{} · {}", p.domain, p.date)}</small></span>
                    </button>
                }).collect_view()}
                <h5>"stats"</h5>
                <div class="pj-mini">
                    <span><b>{PROJECTS.len()}</b>" projects"</span>
                    <span><b>{PROJECTS.iter().filter(|p| p.live.is_some()).count()}</b>" deployed live"</span>
                    <span><b>{PROJECTS.iter().filter(|p| p.app.is_some()).count()}</b>" in-OS demos"</span>
                    <span><b>{PROJECTS.iter().flat_map(|p| p.stack.iter()).collect::<std::collections::BTreeSet<_>>().len()}</b>" technologies"</span>
                </div>
            </aside>
            {move || {
                let p = current();
                view! {
                    <article class="pj-det" key=p.id>
                        <div class="pj-meta">{p.live.map(|_| view! { <span class="live-pill"><i></i>"live"</span> })}<span class="chip">{p.domain}</span><span>{p.date}</span><span>{format!("~/projects/{}.md", p.id)}</span></div>
                        <h2 class="h1">{p.name}</h2>
                        <p class="muted pj-tag">{p.tag}</p>
                        <ul class="pts">{p.pts.iter().map(|t| view! { <li>{*t}</li> }).collect_view()}</ul>
                        <div class="chips">{p.stack.iter().map(|s| view! { <span class="chip">{*s}</span> }).collect_view()}</div>
                        <div class="pj-actions">
                            {p.live.map(|u| view! {
                                <a class="btn pri" href=u target="_blank" rel="noopener"><Icon name="ext"/>"Visit live site"</a>
                            })}
                            {p.app.and_then(AppId::from_key).map(|a| view! {
                                <button class="btn pri" on:click=move |_| os.open(a, None)><Icon name="play"/>"Launch demo"</button>
                            })}
                            {p.blueprint.map(|b| view! {
                                <button class="btn" on:click=move |_| os.open(AppId::Blueprints, Some(b.to_string()))><Icon name="blueprint"/>"System design"</button>
                            })}
                            <button class="btn" on:click=move |_| os.open(AppId::Terminal, Some(format!("run:cat ~/projects/{}.md", p.id)))><Icon name="terminal"/>"cat in shell"</button>
                        </div>
                    </article>
                }
            }}
        </div>
    }
}
