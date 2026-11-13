//! Certificates: the scanned AgentX certificate, plus generated certificate
//! cards for everything else, each with verification links where they exist.

use crate::data::certs::{Kind, CERTS};
use crate::data::profile::ME;
use crate::os::icons::Icon;
use crate::state::Win;
use crate::util::dom;
use leptos::*;

#[component]
pub fn Certs(win: Win) -> impl IntoView {
    let (sel, set_sel) = create_signal(CERTS[0].id);
    create_effect(move |_| {
        if let Some(id) = win.arg.get() {
            if let Some(c) = CERTS.iter().find(|c| c.id == id) {
                set_sel.set(c.id);
            }
            win.arg.set(None);
        }
    });
    let cur = move || CERTS.iter().find(|c| c.id == sel.get()).unwrap_or(&CERTS[0]);
    let group = |kind: Kind, label: &'static str| {
        view! {
            <h5>{label}</h5>
            {CERTS.iter().filter(move |c| c.kind == kind).map(|c| view! {
                <button class="row-btn" class:sel=move || sel.get() == c.id on:click=move |_| set_sel.set(c.id)>
                    <Icon name=if kind == Kind::Award { "award" } else { "check" }/>
                    <span><b>{c.title}</b><small>{format!("{} · {}", c.issuer, c.date)}</small></span>
                </button>
            }).collect_view()}
        }
    };
    view! {
        <div class="split">
            <aside class="side">{group(Kind::Award, "awards")}{group(Kind::Course, "courses")}</aside>
            {move || {
                let c = cur();
                view! {
                    <div class="ct-view">
                        {match c.img {
                            Some(src) => view! { <div class="ct-img"><img src=src alt=format!("{} certificate", c.title) loading="lazy"/></div> }.into_view(),
                            None => view! {
                                <div class="ct-card" class:award=c.kind == Kind::Award>
                                    <div>
                                        <div class="eyebrow">{if c.kind == Kind::Award { "certificate of achievement" } else { "certificate of completion" }}</div>
                                        <div class="t">{c.title}</div>
                                        {c.sub.map(|s| view! { <div class="muted">{s}</div> })}
                                    </div>
                                    <div><small class="muted">"awarded to"</small><div class="nm">{ME.name}</div></div>
                                    <div class="ft"><span>{c.issuer}</span><span>{c.date}{c.hours.map(|h| format!(" · {h}"))}</span></div>
                                    <span class="ct-seal"><Icon name=if c.kind == Kind::Award { "award" } else { "check" }/></span>
                                </div>
                            }.into_view(),
                        }}
                        <dl class="ct-facts">
                            <dt>"Issuer"</dt><dd>{c.issuer}</dd>
                            <dt>"Date"</dt><dd>{c.date}</dd>
                            {c.hours.map(|h| view! { <dt>"Length"</dt><dd>{h}</dd> })}
                            {c.no.map(|n| view! { <dt>"Cert no."</dt><dd><code>{n}</code></dd> })}
                            {c.facts.iter().map(|(k, v)| view! { <dt>{*k}</dt><dd>{*v}</dd> }).collect_view()}
                        </dl>
                        {c.url.map(|u| view! {
                            <div class="pj-actions">
                                <button class="btn pri" on:click=move |_| dom::open_url(u)><Icon name="ext"/>"Verify on Udemy"</button>
                                <button class="btn" on:click=move |_| dom::copy(u)><Icon name="copy"/>"Copy link"</button>
                            </div>
                        })}
                    </div>
                }
            }}
        </div>
    }
}
