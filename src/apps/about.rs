//! whoami: the profile — hero, stats, a skill radar computed in Rust,
//! focus cards that deep-link into apps, the journey, education.

use crate::apps::AppId;
use crate::data::profile::{EDUCATION, FOCUS, ME, RADAR_EXTRA, TIMELINE};
use crate::os::icons::{Icon, Sigil};
use crate::state::os;
use leptos::*;
use std::f64::consts::{FRAC_PI_2, TAU};

fn radar() -> impl IntoView {
    let axes: Vec<(&str, f64)> = FOCUS.iter().map(|f| (f.title, f.level)).chain(RADAR_EXTRA.iter().copied()).collect();
    let n = axes.len() as f64;
    let (cx, cy, r) = (150.0, 128.0, 92.0);
    let pt = |i: usize, v: f64| {
        let a = -FRAC_PI_2 + TAU * i as f64 / n;
        (cx + a.cos() * r * v, cy + a.sin() * r * v)
    };
    let ring = |v: f64| (0..axes.len()).map(|i| { let (x, y) = pt(i, v); format!("{x:.1},{y:.1}") }).collect::<Vec<_>>().join(" ");
    let shape = axes.iter().enumerate().map(|(i, (_, l))| { let (x, y) = pt(i, l / 100.0); format!("{x:.1},{y:.1}") }).collect::<Vec<_>>().join(" ");
    view! {
        <svg class="radar" viewBox="-46 0 392 256" role="img" aria-label="Skill radar">
            {[0.25, 0.5, 0.75, 1.0].map(|v| view! { <polygon class="rg" points=ring(v)></polygon> }).collect_view()}
            {(0..axes.len()).map(|i| { let (x, y) = pt(i, 1.0); view! { <line class="rx" x1=cx y1=cy x2=x y2=y></line> } }).collect_view()}
            <polygon class="rs" points=shape></polygon>
            {axes.iter().enumerate().map(|(i, (name, lvl))| {
                let (x, y) = pt(i, 1.0);
                let (lx, ly) = pt(i, 1.2);
                let anchor = if (lx - cx).abs() < 8.0 { "middle" } else if lx > cx { "start" } else { "end" };
                let (dx, dy) = pt(i, lvl / 100.0);
                view! {
                    <circle class="rd" cx=dx cy=dy r="3"></circle>
                    <text class="rl" x=lx y=ly + 4.0 text-anchor=anchor>{*name}</text>
                    <title>{format!("{name}: {lvl}/100")}</title>
                    <circle cx=x cy=y r="1.5" class="rx-dot"></circle>
                }
            }).collect_view()}
        </svg>
    }
}

#[component]
pub fn About() -> impl IntoView {
    let os = os();
    let go = move |k: &'static str| move |_| { if let Some(a) = AppId::from_key(k) { os.open(a, None) } };
    view! {
        <div class="pad about">
            <div class="ab-head">
                <Sigil/>
                <div class="ab-id">
                    <div class="eyebrow">"whoami"</div>
                    <h1 class="h1 glitch" data-text=ME.name>{ME.name}</h1>
                    <div class="muted">{ME.role}</div>
                    <span class="status-pill"><i></i>{ME.status}</span>
                </div>
            </div>
            <div class="ab-grid">
                <div>
                    <div class="stats">
                        <div><b>{ME.cgpa}</b><span>"CGPA / 10"</span></div>
                        <div><b>"2nd"</b><span>"AgentX 2026"</span></div>
                        <div><b>"Top 80"</b><span>"of 500+ teams"</span></div>
                        <div><b>"4"</b><span>"systems shipped"</span></div>
                    </div>
                    <div class="focus">
                        {FOCUS.iter().map(|f| view! {
                            <button class="fcard" on:click=go(f.app)>
                                <Icon name=f.icon/><b>{f.title}</b><span>{f.body}</span><em>{f.cta}</em>
                            </button>
                        }).collect_view()}
                    </div>
                </div>
                <figure class="ab-radar">{radar()}<figcaption>"self-assessed depth · rendered from typed data"</figcaption></figure>
            </div>

            <div class="eyebrow">"journey · git log --oneline"</div>
            <ol class="timeline">
                {TIMELINE.iter().map(|m| view! {
                    <li class=format!("tl-{}", m.kind)>
                        <code>{m.hash}</code>
                        <time>{m.date}</time>
                        <span><b>{format!("{}({})", m.kind, m.scope)}</b>": "{m.msg}</span>
                    </li>
                }).collect_view()}
            </ol>

            <div class="eyebrow">"education"</div>
            <div class="edu">
                {EDUCATION.iter().map(|e| view! {
                    <div><span class="yr">{e.years}</span><span><b>{e.title}</b><br/><span class="muted">{e.place}</span></span><span class="sc">{e.score}</span></div>
                }).collect_view()}
            </div>
            <div class="pj-actions" style="margin-top:16px">
                <button class="btn pri" on:click=go("files")><Icon name="folder"/>"Browse projects"</button>
                <button class="btn" on:click=go("certs")><Icon name="award"/>"Certificates"</button>
                <button class="btn" on:click=go("contact")><Icon name="mail"/>"Contact"</button>
                <button class="btn" on:click=go("terminal")><Icon name="terminal"/>"Open a shell"</button>
            </div>
        </div>
    }
}
