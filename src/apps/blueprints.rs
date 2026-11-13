//! Blueprints: system-design diagrams generated from data. Edge routing
//! picks the facing sides of each pair of boxes and draws a cubic Bézier;
//! hovering a component highlights everything it talks to.

use crate::data::blueprints::{by_id, Blueprint, NodeKind, BLUEPRINTS, NODE_H};
use crate::state::Win;
use leptos::*;

struct Geo {
    d: String,
    lx: f64,
    ly: f64,
}

fn route(bp: &Blueprint, from: &str, to: &str) -> Option<Geo> {
    let a = bp.nodes.iter().find(|n| n.id == from)?;
    let b = bp.nodes.iter().find(|n| n.id == to)?;
    let (acx, acy) = (a.x + a.w / 2.0, a.y + NODE_H / 2.0);
    let (bcx, bcy) = (b.x + b.w / 2.0, b.y + NODE_H / 2.0);
    let (dx, dy) = (bcx - acx, bcy - acy);
    // Parallel edges in opposite directions get nudged apart.
    let twin = bp.edges.iter().any(|e| e.0 == to && e.1 == from);
    let nudge = if twin { if from < to { -7.0 } else { 7.0 } } else { 0.0 };
    let horizontal = dx.abs() / ((a.w + b.w) / 2.0) > dy.abs() / NODE_H;
    let (p1, c1, p2, c2) = if horizontal {
        let s = dx.signum();
        let p1 = (acx + s * a.w / 2.0, acy + nudge);
        let p2 = (bcx - s * b.w / 2.0, bcy + nudge);
        let off = ((p2.0 - p1.0).abs() * 0.45).max(24.0) * s;
        (p1, (p1.0 + off, p1.1), p2, (p2.0 - off, p2.1))
    } else {
        let s = dy.signum();
        let p1 = (acx + nudge, acy + s * NODE_H / 2.0);
        let p2 = (bcx + nudge, bcy - s * NODE_H / 2.0);
        let off = ((p2.1 - p1.1).abs() * 0.45).max(20.0) * s;
        (p1, (p1.0, p1.1 + off), p2, (p2.0, p2.1 - off))
    };
    // Bézier midpoint for the label.
    let lx = 0.125 * p1.0 + 0.375 * c1.0 + 0.375 * c2.0 + 0.125 * p2.0;
    let ly = 0.125 * p1.1 + 0.375 * c1.1 + 0.375 * c2.1 + 0.125 * p2.1;
    Some(Geo {
        d: format!("M{:.1} {:.1} C{:.1} {:.1} {:.1} {:.1} {:.1} {:.1}", p1.0, p1.1, c1.0, c1.1, c2.0, c2.1, p2.0, p2.1),
        lx,
        ly,
    })
}

#[component]
pub fn Blueprints(win: Win) -> impl IntoView {
    let (cur, set_cur) = create_signal(BLUEPRINTS[0].id);
    let (hover, set_hover) = create_signal(None::<&'static str>);
    create_effect(move |_| {
        if let Some(id) = win.arg.get() {
            if let Some(b) = by_id(&id) {
                set_cur.set(b.id);
            }
            win.arg.set(None);
        }
    });
    let bp = move || by_id(cur.get()).unwrap_or(&BLUEPRINTS[0]);

    view! {
        <div class="bp">
            <div class="tabs" role="tablist">
                {BLUEPRINTS.iter().map(|b| view! {
                    <button class="tab" role="tab" class:on=move || cur.get() == b.id on:click=move |_| { set_cur.set(b.id); set_hover.set(None); }>{b.title}</button>
                }).collect_view()}
            </div>
            {move || {
                let b = bp();
                view! {
                    <div class="bp-body">
                        <div class="diagram" class:hovering=move || hover.get().is_some()>
                            <svg viewBox="-10 -10 790 400" role="img" aria-label=format!("{} architecture", b.title)>
                                <defs>
                                    <marker id="bp-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
                                        <path d="M0 0L10 5L0 10z" class="dg-head"></path>
                                    </marker>
                                </defs>
                                {b.edges.iter().filter_map(|&(f, t, _)| route(b, f, t).map(|g| {
                                    let lit = move || hover.get().is_some_and(|h| h == f || h == t);
                                    view! {
                                        <g class="dg-e" class:lit=lit>
                                            <path class="dg-edge" d=g.d marker-end="url(#bp-arrow)"></path>
                                        </g>
                                    }
                                })).collect_view()}
                                {b.nodes.iter().map(|n| {
                                    let kind = match n.k { NodeKind::Svc => "", NodeKind::Ext => "k-ext", NodeKind::Ai => "k-ai", NodeKind::Db => "k-db" };
                                    let id = n.id;
                                    let near = move || hover.get().is_some_and(|h| h == id || b.edges.iter().any(|e| (e.0 == h && e.1 == id) || (e.1 == h && e.0 == id)));
                                    view! {
                                        <g class=format!("dg-node {kind}") class:near=near
                                            on:mouseenter=move |_| set_hover.set(Some(id)) on:mouseleave=move |_| set_hover.set(None)>
                                            <rect x=n.x y=n.y width=n.w height=NODE_H rx="7"></rect>
                                            <text x=n.x + 12.0 y=n.y + 20.0>{n.l}</text>
                                            <text class="s" x=n.x + 12.0 y=n.y + 35.0>{n.s}</text>
                                        </g>
                                    }
                                }).collect_view()}
                                // Labels last, so boxes never cover them.
                                {b.edges.iter().filter(|e| !e.2.is_empty()).filter_map(|&(f, t, l)| route(b, f, t).map(|g| {
                                    let lit = move || hover.get().is_some_and(|h| h == f || h == t);
                                    view! {
                                        <g class="dg-e" class:lit=lit>
                                            <rect class="dg-lbg" x=g.lx - l.len() as f64 * 2.9 - 4.0 y=g.ly - 8.0 width=l.len() as f64 * 5.8 + 8.0 height="14" rx="3"></rect>
                                            <text class="dg-lbl" x=g.lx y=g.ly + 3.0 text-anchor="middle">{l}</text>
                                        </g>
                                    }
                                })).collect_view()}
                            </svg>
                        </div>
                        <div class="bp-legend">
                            <span><i class="lg-svc"></i>"service"</span><span><i class="lg-ai"></i>"AI / model"</span>
                            <span><i class="lg-db"></i>"data store"</span><span><i class="lg-ext"></i>"external"</span>
                            <em>"hover a component to trace its connections"</em>
                        </div>
                        <div class="notes">
                            {b.notes.iter().map(|(t, d)| view! { <div class="note"><b>{*t}</b><span>{*d}</span></div> }).collect_view()}
                        </div>
                    </div>
                }
            }}
        </div>
    }
}
