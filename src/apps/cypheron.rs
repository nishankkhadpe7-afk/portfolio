//! Cypheron demo: a simulated, deterministic web-security scan. The same URL
//! always produces the same findings (seeded by its FNV hash), modules
//! stream results as they "finish", and the report can be exported.

use crate::data::findings::{self, Finding, Sev, FINDINGS, MODS};
use crate::os::icons::Icon;
use crate::state::Win;
use crate::util::{dom, rng::{fnv1a, Rng}};
use leptos::leptos_dom::helpers::IntervalHandle;
use leptos::*;
use std::time::Duration;

fn plan_for(url: &str) -> Vec<usize> {
    let mut r = Rng::new(fnv1a(url));
    let mut v: Vec<usize> = (0..FINDINGS.len()).filter(|_| r.chance(0.55)).collect();
    if !v.iter().any(|&i| FINDINGS[i].sev != Sev::Info) {
        v.push(9);
    }
    v
}

fn triage(fs: &[&Finding], score: u32) -> String {
    let (g, _) = findings::grade(score);
    let count = |s: Sev| fs.iter().filter(|f| f.sev == s).count();
    let (c, h, m, l) = (count(Sev::Critical), count(Sev::High), count(Sev::Medium), count(Sev::Low));
    let mut out = format!("Grade {g} ({score}/100). ");
    if let Some(top) = fs.first() {
        out += &format!("Fix “{}” first — {} ", top.t, top.d.split('.').next().unwrap_or("").to_lowercase());
        out += ". ";
    }
    if c + h > 1 {
        out += &format!("{} more high-impact issue(s) follow; ", c + h - 1);
    }
    if m > 0 {
        out += &format!("{m} medium finding(s) are hardening work for this sprint. ");
    }
    if l > 0 {
        out += &format!("{l} low-severity item(s) are one-line config fixes. ");
    }
    let hours = c * 4 + h * 2 + m + l / 2;
    out += &format!("Estimated remediation effort: ~{} engineer-hour(s).", hours.max(1));
    out
}

#[component]
pub fn Cypheron(win: Win) -> impl IntoView {
    let (url, set_url) = create_signal("https://shop.example".to_string());
    let (running, set_running) = create_signal(false);
    let progress = create_rw_signal(vec![0.0f64; MODS.len()]);
    let shown = create_rw_signal(Vec::<usize>::new());
    let logs = create_rw_signal(Vec::<(String, bool)>::new());
    let (score, set_score) = create_signal(None::<u32>);
    let target = store_value(String::new());
    let timer = store_value(None::<IntervalHandle>);
    on_cleanup(move || {
        if let Some(h) = timer.get_value() {
            h.clear()
        }
    });

    let found = create_memo(move |_| {
        let mut v: Vec<&'static Finding> = shown.get().into_iter().map(|i| &FINDINGS[i]).collect();
        v.sort_by_key(|f| std::cmp::Reverse(f.sev));
        v
    });

    let scan = move |u: String| {
        if let Some(h) = timer.get_value() {
            h.clear();
        }
        let u = if u.starts_with("http") { u } else { format!("https://{u}") };
        set_url.set(u.clone());
        target.set_value(u.clone());
        let plan = plan_for(&u);
        progress.set(vec![0.0; MODS.len()]);
        shown.set(Vec::new());
        set_score.set(None);
        set_running.set(true);
        let host = u.trim_start_matches("https://").trim_start_matches("http://").split('/').next().unwrap_or("").to_string();
        logs.set(vec![(format!("resolving {host} … 203.0.113.{}", fnv1a(&host) % 250 + 2), false), ("TLS handshake ok · starting 7 modules".into(), false)]);
        let rng = std::cell::RefCell::new(Rng::new(fnv1a(&u) ^ 0xA5A5));
        let cur = std::cell::Cell::new(0usize);
        let h = set_interval_with_handle(
            move || {
                let c = cur.get();
                if c >= MODS.len() {
                    return;
                }
                let mut done_mod = false;
                progress.update(|p| {
                    p[c] = (p[c] + rng.borrow_mut().range(7.0, 24.0)).min(100.0);
                    done_mod = p[c] >= 100.0;
                });
                if done_mod {
                    let (key, name) = MODS[c];
                    let hits: Vec<usize> = plan.iter().copied().filter(|&i| FINDINGS[i].m == key).collect();
                    shown.update(|s| s.extend(hits.iter().copied()));
                    logs.update(|l| l.push((format!("[{key}] {name} ✓ {} finding(s)", hits.len()), true)));
                    cur.set(c + 1);
                    if c + 1 == MODS.len() {
                        let fs: Vec<&Finding> = plan.iter().map(|&i| &FINDINGS[i]).collect();
                        set_score.set(Some(findings::score(&fs)));
                        set_running.set(false);
                        logs.update(|l| l.push(("scan complete · report ready".into(), true)));
                        if let Some(h) = timer.get_value() {
                            h.clear();
                        }
                    }
                }
            },
            Duration::from_millis(65),
        )
        .ok();
        timer.set_value(h);
    };

    create_effect(move |_| {
        if let Some(u) = win.arg.get() {
            scan(u);
            win.arg.set(None);
        }
    });
    if win.arg.get_untracked().is_none() {
        scan(url.get_untracked());
    }

    let export = move |_| {
        let fs = found.get_untracked();
        let sc = score.get_untracked().unwrap_or(0);
        let mut md = format!("# Cypheron report — {}\n\nGrade **{}** · score {sc}/100\n\n## AI triage\n\n{}\n\n## Findings\n", target.get_value(), findings::grade(sc).0, triage(&fs, sc));
        for f in &fs {
            md += &format!("\n### [{}] {} ({})\n\n{}\n\n**Fix:** {}\n", f.sev.css().to_uppercase(), f.t, f.cwe, f.d, f.f);
        }
        md += "\n---\nGenerated by the NishankOS Cypheron demo (simulated; no requests were sent).\n";
        dom::download("cypheron-report.md", "text/markdown", &md);
    };

    view! {
        <div class="cy">
            <form class="cy-bar" on:submit=move |e| { e.prevent_default(); scan(url.get_untracked()); }>
                <label class="cy-url"><Icon name="lock"/>
                    <input prop:value=url on:input=move |e| set_url.set(event_target_value(&e)) aria-label="Target URL" spellcheck="false"/>
                </label>
                <button class="btn pri" type="submit" disabled=running><Icon name="play"/>{move || if running.get() { "Scanning…" } else { "Scan" }}</button>
                <button class="btn" type="button" disabled=move || score.get().is_none() on:click=export><Icon name="download"/>"Report"</button>
            </form>
            <div class="cy-main">
                <div class="cy-left">
                    {move || match score.get() {
                        Some(s) => {
                            let (g, col) = findings::grade(s);
                            view! {
                                <div class="grade" style:color=col>
                                    <div class="grade-l">{g}</div>
                                    <div><small>"risk score"</small><b style="color:var(--ink)">{s}"/100"</b></div>
                                </div>
                            }.into_view()
                        }
                        None => view! {
                            <div class="grade scanning">
                                <div class="grade-l"><span class="spin"></span></div>
                                <div><small>"scanning"</small><b>{move || format!("{:.0}%", progress.get().iter().sum::<f64>() / MODS.len() as f64)}</b></div>
                            </div>
                        }.into_view(),
                    }}
                    <div class="counts">
                        {Sev::ALL.map(|s| view! {
                            <div class=format!("c-{}", s.css())><b>{move || found.get().iter().filter(|f| f.sev == s).count()}</b><span>{s.css()}</span></div>
                        }).collect_view()}
                    </div>
                    <div class="mods">
                        {MODS.iter().enumerate().map(|(i, (_, name))| view! {
                            <div class="mod" class:done=move || { progress.get()[i] >= 100.0 } style=move || format!("--p:{}%", progress.get()[i])>
                                <span>{*name}</span>
                                <em>{move || { let p = progress.get()[i]; if p >= 100.0 { "done".to_string() } else if p > 0.0 { format!("{p:.0}%") } else { "queued".into() } }}</em>
                                <i></i>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
                <div class="cy-right">
                    <Show when=move || score.get().is_some()>
                        <div class="ai-box">
                            <div class="eyebrow"><Icon name="spark"/>"AI triage · simulated Claude summary"</div>
                            <p>{move || triage(&found.get(), score.get().unwrap_or(0))}</p>
                        </div>
                    </Show>
                    <For each=move || found.get() key=|f| f.t let:f>
                        <details class="finding">
                            <summary><span class=format!("sev {}", f.sev.css())>{f.sev.css()}</span><b>{f.t}</b><em>{f.cwe}</em></summary>
                            <div class="fd"><span>{f.d}</span><span><b style="color:var(--ink)">"Fix: "</b><code>{f.f}</code></span></div>
                        </details>
                    </For>
                    <div class="cy-log">
                        {move || logs.get().into_iter().map(|(l, g)| view! { <div class:g=g>{format!("› {l}")}</div> }).collect_view()}
                    </div>
                </div>
            </div>
            <p class="demo-note">"Demo mode: results are simulated in WebAssembly and no request leaves this page. The real Cypheron probes targets with HTTPX + BeautifulSoup and streams progress over WebSockets."</p>
        </div>
    }
}
