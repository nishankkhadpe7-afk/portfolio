//! Fintrix: ask a compliance question, watch the retrieval pipeline light
//! up stage by stage, inspect the score table, and stress-test the LRS
//! limit in the What-If panel.

mod retrieval;

use crate::data::corpus::{CORPUS, SUGGESTIONS};
use crate::os::icons::Icon;
use crate::util::thousands;
use leptos::*;
use retrieval::{retrieve, Ranking, ALPHA, MIN_COVERAGE};
use std::time::Duration;

const STAGES: &[&str] = &["query", "tokenise", "BM25", "trigram ∠", "fuse α=0.55", "ground-check", "answer"];
const LRS: i64 = 250_000;

#[component]
pub fn Fintrix() -> impl IntoView {
    let (q, set_q) = create_signal(SUGGESTIONS[0].to_string());
    let (stage, set_stage) = create_signal(STAGES.len());
    let result = create_rw_signal(None::<(String, Ranking)>);
    let gen = store_value(0u32);
    let (elapsed, set_elapsed) = create_signal(0.0f64);

    let ask = move |text: String| {
        let text = text.trim().to_string();
        if text.is_empty() {
            return;
        }
        set_q.set(text.clone());
        gen.update_value(|g| *g += 1);
        let my = gen.get_value();
        let t0 = crate::util::dom::now();
        let mut r = Some(retrieve(&text));
        set_elapsed.set(crate::util::dom::now() - t0);
        set_stage.set(0);
        for i in 1..=STAGES.len() {
            let r = if i == STAGES.len() { r.take().map(|r| (text.clone(), r)) } else { None };
            set_timeout(
                move || {
                    if gen.get_value() != my {
                        return;
                    }
                    set_stage.set(i);
                    if let Some(r) = r {
                        result.set(Some(r));
                    }
                },
                Duration::from_millis(110 * i as u64),
            );
        }
    };
    ask(q.get_untracked());

    // What-If
    let (used, set_used) = create_signal(120_000i64);
    let (new, set_new) = create_signal(60_000i64);
    let (purpose, set_purpose) = create_signal("edu".to_string());
    let total = move || used.get() + new.get();
    let pct = move || total() as f64 / LRS as f64;
    let usd = |n: i64| format!("${}", thousands(n));

    view! {
        <div class="fx">
            <div class="fx-main">
                <form class="fx-q" on:submit=move |e| { e.prevent_default(); ask(q.get_untracked()); }>
                    <input prop:value=q on:input=move |e| set_q.set(event_target_value(&e)) aria-label="Ask a compliance question" placeholder="Ask about LRS, FEMA penalties, KYC…"/>
                    <button class="btn pri" type="submit"><Icon name="search"/>"Ask"</button>
                </form>
                <div class="sugg">
                    {SUGGESTIONS.iter().map(|s| view! { <button on:click=move |_| ask(s.to_string())>{*s}</button> }).collect_view()}
                </div>
                <div class="pipe">
                    {STAGES.iter().enumerate().map(|(i, s)| view! {
                        <span class:on=move || { stage.get() > i } class:cur=move || { stage.get() == i }>{*s}</span>
                        {(i + 1 < STAGES.len()).then(|| view! { <i>"→"</i> })}
                    }).collect_view()}
                    <em>{move || format!("retrieval {:.2} ms in WASM", elapsed.get())}</em>
                </div>
                {move || result.with(|r| r.as_ref().map(|(text, rk)| {
                    let top = &rk.hits[0];
                    let c = &CORPUS[top.doc];
                    let terms = rk.terms.join(" · ");
                    let body = if rk.grounded {
                        view! {
                            <div class="ans">
                                <div class="eyebrow">"answer · grounded"</div>
                                <p>{c.ans}</p>
                                <div class="src">"source › "{c.src}</div>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <div class="ans miss">
                                <div class="eyebrow" style="color:var(--warn)">"no grounded answer"</div>
                                <p>"None of the indexed RBI / FEMA chunks cover “"{text.clone()}"”, so Fintrix declines instead of guessing. This guard is what keeps hallucinations out."</p>
                            </div>
                        }.into_view()
                    };
                    view! {
                        {body}
                        <div class="fx-terms">
                            <span class="eyebrow">"query terms"</span><code>{if terms.is_empty() { "∅ (all stop-words)".to_string() } else { terms }}</code>
                            <span class="eyebrow">"coverage"</span>
                            <code class:low={rk.coverage < MIN_COVERAGE}>{format!("{:.0}% (needs ≥ {:.0}%)", rk.coverage * 100.0, MIN_COVERAGE * 100.0)}</code>
                        </div>
                        <table class="ret">
                            <thead><tr><th>"chunk"</th><th>"BM25"</th><th>"trigram"</th><th>{format!("hybrid ({ALPHA}·bm + {:.2}·tri)", 1.0 - ALPHA)}</th></tr></thead>
                            <tbody>
                                {rk.hits.iter().take(4).enumerate().map(|(j, h)| view! {
                                    <tr class:top=j == 0 && rk.grounded>
                                        <td><b style="font-weight:500">{CORPUS[h.doc].src}</b><br/><span class="muted">{CORPUS[h.doc].text}</span></td>
                                        <td class="n">{format!("{:.2}", h.bm25)}</td>
                                        <td class="n">{format!("{:.2}", h.dense)}</td>
                                        <td class="n">{format!("{:.2}", h.hybrid)}<div class="bar"><i style:width=format!("{}%", (h.hybrid * 100.0).round())></i></div></td>
                                    </tr>
                                }).collect_view()}
                            </tbody>
                        </table>
                    }
                }))}
            </div>
            <aside class="fx-side">
                <div class="eyebrow">"what-if · LRS headroom"</div>
                <div class="fld">
                    <label>"Already remitted this FY"<output>{move || usd(used.get())}</output></label>
                    <input type="range" min="0" max="250000" step="5000" prop:value=move || used.get().to_string()
                        on:input=move |e| set_used.set(event_target_value(&e).parse().unwrap_or(0))/>
                </div>
                <div class="fld">
                    <label>"New remittance"<output>{move || usd(new.get())}</output></label>
                    <input type="range" min="0" max="200000" step="5000" prop:value=move || new.get().to_string()
                        on:input=move |e| set_new.set(event_target_value(&e).parse().unwrap_or(0))/>
                </div>
                <div class="fld">
                    <label>"Purpose"</label>
                    <select on:change=move |e| set_purpose.set(event_target_value(&e))>
                        <option value="edu">"Education abroad"</option>
                        <option value="travel">"Travel"</option>
                        <option value="gift">"Gift to a relative"</option>
                        <option value="inv">"Overseas investment"</option>
                    </select>
                </div>
                <div class="meter"><i style:width=move || format!("{}%", (pct() * 100.0).min(100.0))
                    style:background=move || { if pct() > 1.0 { "var(--crit)" } else if pct() > 0.9 { "var(--warn)" } else { "var(--ok)" } }></i></div>
                {move || {
                    let (cls, label, tag) = if pct() > 1.0 { ("critical", "Breach: exceeds LRS limit", "risk") } else if pct() > 0.9 { ("medium", "Near the limit", "warn") } else { ("ok", "Within limit", "ok") };
                    let extra = match purpose.get().as_str() {
                        "edu" => "Education remittances still count toward the LRS limit.",
                        "travel" => "Forex for travel must be bought through an authorised person.",
                        "gift" => "Gifts to relatives abroad count toward the same limit.",
                        _ => "Overseas portfolio investment also follows the OI Rules, 2022.",
                    };
                    let t = total();
                    let tail = if t > LRS { format!("Over by {}; this needs prior RBI approval.", usd(t - LRS)) } else { format!("Headroom left: {}.", usd(LRS - t)) };
                    view! {
                        <div class="verdict">
                            <b><span class=format!("sev {cls}")>{tag}</span>{label}</b>
                            "Total this FY: "<code>{usd(t)}</code>" of "{usd(LRS)}". "{tail}" "{extra}
                        </div>
                    }
                }}
                <p class="fx-note">"Rules, not the LLM, decide risk — so verdicts are auditable."</p>
            </aside>
        </div>
        <p class="demo-note">"The BM25 + trigram retriever above is real and runs in your browser. Production Fintrix uses FAISS + SentenceTransformers with Mistral generating the grounded answer."</p>
    }
}
