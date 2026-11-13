//! CodeLab: LeetCode-style practice. The editor is a textarea under a
//! Rust-highlighted overlay; code runs in a sandboxed Worker; expected
//! answers come from reference solutions compiled into this wasm binary.

mod highlight;
mod runner;

use crate::data::problems::{by_id, Problem, PROBLEMS};
use crate::os::icons::Icon;
use crate::state::{os, Win};
use crate::util::store;
use leptos::*;
use runner::{RunResult, DEADLINE_MS};
use serde_json::Value;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Desc,
    List,
    Solution,
}

#[derive(Clone, PartialEq)]
struct Case {
    input: String,
    expected: String,
    got: String,
    ok: bool,
    err: Option<String>,
    logs: Vec<String>,
}

#[derive(Clone, PartialEq)]
enum Console {
    Idle,
    Running(bool),
    Compile(String),
    Timeout,
    Done { submit: bool, cases: Vec<Case>, ms: f64 },
}

fn show(v: &Value) -> String {
    let s = v.to_string();
    if s.chars().count() > 140 {
        let n = v.as_array().map(|a| format!(" ({} items)", a.len())).unwrap_or_default();
        format!("{}…{n}", s.chars().take(120).collect::<String>())
    } else {
        s
    }
}

fn args_str(p: &Problem, args: &[Value]) -> String {
    p.params.iter().zip(args).map(|(n, v)| format!("{n} = {}", show(v))).collect::<Vec<_>>().join(", ")
}

fn solved_set() -> Vec<String> {
    store::get("solved").map(|s| s.split(',').filter(|x| !x.is_empty()).map(String::from).collect()).unwrap_or_default()
}

#[component]
pub fn CodeLab(win: Win) -> impl IntoView {
    let os = os();
    let (pid, set_pid) = create_signal(PROBLEMS[0].id);
    let problem = move || by_id(pid.get()).unwrap_or(&PROBLEMS[0]);
    let load = |p: &Problem| store::get(&format!("code.{}", p.id)).unwrap_or_else(|| p.starter());
    let (code, set_code) = create_signal(load(&PROBLEMS[0]));
    let (tab, set_tab) = create_signal(Tab::Desc);
    let (console, set_console) = create_signal(Console::Idle);
    let (case_i, set_case_i) = create_signal(0usize);
    let solved = create_rw_signal(solved_set());
    let ta = create_node_ref::<html::Textarea>();
    let hl = create_node_ref::<html::Pre>();
    let gutter = create_node_ref::<html::Div>();

    let select = move |id: &'static str| {
        set_pid.set(id);
        if let Some(p) = by_id(id) {
            set_code.set(load(p));
        }
        set_tab.set(Tab::Desc);
        set_console.set(Console::Idle);
    };
    create_effect(move |_| {
        if let Some(id) = win.arg.get() {
            if let Some(p) = by_id(&id) {
                select(p.id);
            }
            win.arg.set(None);
        }
    });
    // Persist drafts per problem.
    create_effect(move |_| {
        let c = code.get();
        store::set(&format!("code.{}", pid.get_untracked()), &c);
    });

    let execute = move |submit: bool| {
        if matches!(console.get_untracked(), Console::Running(_)) {
            return;
        }
        let p = problem();
        let all = (p.tests)();
        let tests: Vec<Vec<Value>> = if submit { all } else { all.into_iter().take(p.visible).collect() };
        set_console.set(Console::Running(submit));
        set_case_i.set(0);
        let src = code.get_untracked();
        let tests2 = tests.clone();
        runner::run(&src, p.func, &tests, move |r| match r {
            RunResult::Compile(e) => set_console.set(Console::Compile(e)),
            RunResult::Timeout => set_console.set(Console::Timeout),
            RunResult::Cases(outs) => {
                let mut ms = 0.0;
                let cases: Vec<Case> = tests2
                    .iter()
                    .zip(outs)
                    .map(|(args, o)| {
                        let want = (p.reference)(args);
                        ms += o.ms;
                        Case {
                            input: args_str(p, args),
                            expected: show(&want),
                            got: if o.err.is_some() { "—".into() } else { show(&o.out) },
                            ok: o.err.is_none() && p.judge(&o.out, &want),
                            err: o.err,
                            logs: o.logs,
                        }
                    })
                    .collect();
                let first_bad = cases.iter().position(|c| !c.ok).unwrap_or(0);
                set_case_i.set(first_bad);
                if submit && cases.iter().all(|c| c.ok) {
                    let fresh = solved.with_untracked(|s| !s.iter().any(|x| x == p.id));
                    if fresh {
                        solved.update(|s| s.push(p.id.to_string()));
                        store::set("solved", &solved.get_untracked().join(","));
                        let n = solved.with_untracked(|s| s.len());
                        if n == PROBLEMS.len() {
                            os.toast("award", "All problems solved", "Every CodeLab problem accepted. Nishank would like to interview you.", None);
                        } else {
                            os.toast("check", format!("Accepted · {}", p.title), format!("{n}/{} solved", PROBLEMS.len()), None);
                        }
                    }
                }
                set_console.set(Console::Done { submit, cases, ms });
            }
        });
    };

    // ── editor behaviour ───────────────────────────────────────────────
    let sync_scroll = move || {
        if let (Some(t), Some(h), Some(g)) = (ta.get_untracked(), hl.get_untracked(), gutter.get_untracked()) {
            h.set_scroll_top(t.scroll_top());
            h.set_scroll_left(t.scroll_left());
            g.set_scroll_top(t.scroll_top());
        }
    };
    let insert = move |text: &str| {
        let Some(t) = ta.get_untracked() else { return };
        let (s, e) = (t.selection_start().ok().flatten().unwrap_or(0) as usize, t.selection_end().ok().flatten().unwrap_or(0) as usize);
        let v: Vec<u16> = t.value().encode_utf16().collect();
        let mut nv = String::from_utf16_lossy(&v[..s.min(v.len())]);
        nv.push_str(text);
        nv.push_str(&String::from_utf16_lossy(&v[e.min(v.len())..]));
        t.set_value(&nv);
        let pos = (s + text.encode_utf16().count()) as u32;
        let _ = t.set_selection_range(pos, pos);
        set_code.set(nv);
    };
    let on_key = move |e: ev::KeyboardEvent| {
        let k = e.key();
        if (e.ctrl_key() || e.meta_key()) && k == "Enter" {
            e.prevent_default();
            execute(e.shift_key());
            return;
        }
        match k.as_str() {
            "Tab" => {
                e.prevent_default();
                insert("    ");
            }
            "Enter" => {
                // auto-indent: keep the current indentation, +4 after an opener
                let Some(t) = ta.get_untracked() else { return };
                let s = t.selection_start().ok().flatten().unwrap_or(0) as usize;
                let v: Vec<u16> = t.value().encode_utf16().collect();
                let before = String::from_utf16_lossy(&v[..s.min(v.len())]);
                let line = before.rsplit('\n').next().unwrap_or("");
                let mut indent: String = line.chars().take_while(|c| *c == ' ').collect();
                if line.trim_end().ends_with(['{', '[', '(']) {
                    indent.push_str("    ");
                }
                e.prevent_default();
                insert(&format!("\n{indent}"));
            }
            _ => {}
        }
    };

    let line_count = move || code.with(|c| c.lines().count().max(1) + usize::from(c.ends_with('\n')));

    view! {
        <div class="cl">
            <section class="cl-left">
                <div class="cl-tabs">
                    <button class:on=move || tab.get() == Tab::Desc on:click=move |_| set_tab.set(Tab::Desc)>"Description"</button>
                    <button class:on=move || tab.get() == Tab::List on:click=move |_| set_tab.set(Tab::List)>"Problems"</button>
                    <button class:on=move || tab.get() == Tab::Solution on:click=move |_| set_tab.set(Tab::Solution)>"Solution"</button>
                    <span class="sp">{move || format!("{}/{} solved", solved.get().len(), PROBLEMS.len())}</span>
                </div>
                <div class="cl-lbody">
                    {move || match tab.get() {
                        Tab::Desc => {
                            let p = problem();
                            let tests = (p.tests)();
                            view! {
                                <h3 class="cl-title">{format!("{}. {}", p.n, p.title)}</h3>
                                <div class="cl-meta">
                                    <span class=format!("diff {}", p.diff)>{p.diff}</span>
                                    {p.tags.iter().map(|t| view! { <span class="chip">{*t}</span> }).collect_view()}
                                    {move || solved.get().iter().any(|s| s == p.id).then(|| view! { <span class="sev ok">"solved"</span> })}
                                </div>
                                <div class="cl-desc" inner_html=p.desc></div>
                                {tests.iter().take(p.visible).enumerate().map(|(i, args)| view! {
                                    <div class="ex">
                                        <b>{format!("Example {}", i + 1)}</b>
                                        <pre><span>"Input: "</span>{args_str(p, args)}"\n"<span>"Output: "</span>{show(&(p.reference)(args))}
                                            {p.notes[i].map(|n| view! { "\n"<span>"Explanation: "</span>{n} })}</pre>
                                    </div>
                                }).collect_view()}
                                <h4>"Constraints"</h4>
                                <ul class="cons">{p.cons.iter().map(|c| view! { <li><code>{*c}</code></li> }).collect_view()}</ul>
                                {p.follow.map(|f| view! { <h4>"Follow-up"</h4><p class="muted">{f}</p> })}
                                <p class="cl-rust"><Icon name="cpu"/>{format!("Expected outputs are computed by a Rust reference solution · returns {}", p.ret)}</p>
                            }.into_view()
                        }
                        Tab::List => view! {
                            <div class="pl">
                                {PROBLEMS.iter().map(|p| view! {
                                    <button class:on=move || pid.get() == p.id on:click=move |_| select(p.id)>
                                        {move || if solved.get().iter().any(|s| s == p.id) { view! { <Icon name="check" class="st-ic"/> }.into_view() } else { view! { <i class="st-no"></i> }.into_view() }}
                                        <span>{format!("{}. {}", p.n, p.title)}<small>{p.tags.join(" · ")}</small></span>
                                        <span class=format!("diff {}", p.diff)>{p.diff}</span>
                                    </button>
                                }).collect_view()}
                            </div>
                        }.into_view(),
                        Tab::Solution => {
                            let p = problem();
                            view! {
                                <p class="muted">"Reference approach in JavaScript. Try it yourself first — the judge doesn't mind."</p>
                                <pre class="cl-sol" inner_html=highlight::js(p.solution)></pre>
                                <button class="btn sm" on:click=move |_| set_code.set(p.solution.to_string())><Icon name="copy"/>"Load into editor"</button>
                            }.into_view()
                        }
                    }}
                </div>
            </section>
            <section class="cl-right">
                <div class="cl-tool">
                    <span class="chip">"JavaScript · ES2023"</span>
                    <button class="btn sm" title="Reset to starter code" on:click=move |_| set_code.set(problem().starter())><Icon name="reset"/>"Reset"</button>
                    <span style="flex:1"></span>
                    <button class="btn sm" disabled=move || matches!(console.get(), Console::Running(_)) title="Run visible examples (Ctrl+Enter)" on:click=move |_| execute(false)><Icon name="play"/>"Run"</button>
                    <button class="btn sm sub" disabled=move || matches!(console.get(), Console::Running(_)) title="Submit against all tests (Ctrl+Shift+Enter)" on:click=move |_| execute(true)><Icon name="check"/>"Submit"</button>
                </div>
                <div class="ed">
                    <div class="ed-gut" node_ref=gutter>{move || (1..=line_count()).map(|n| n.to_string()).collect::<Vec<_>>().join("\n")}</div>
                    <div class="ed-wrap">
                        <pre class="ed-hl" node_ref=hl aria-hidden="true" inner_html=move || highlight::js(&code.get())></pre>
                        <textarea class="ed-ta" node_ref=ta spellcheck="false" autocapitalize="off" autocomplete="off" aria-label="Code editor"
                            prop:value=code
                            on:input=move |e| { set_code.set(event_target_value(&e)); sync_scroll(); }
                            on:scroll=move |_| sync_scroll()
                            on:keydown=on_key></textarea>
                    </div>
                </div>
                <div class="cl-con">
                    <div class="cl-tabs"><button class="on">"Console"</button><span class="sp">"Ctrl+Enter run · Ctrl+Shift+Enter submit"</span></div>
                    <div class="cl-cbody">
                        {move || match console.get() {
                            Console::Idle => view! { <p class="muted">"Run your code to see results. Each run gets a fresh, sandboxed Web Worker."</p> }.into_view(),
                            Console::Running(s) => view! { <div class="cl-pending"><span class="spin"></span>{if s { "Judging against all tests…" } else { "Running examples…" }}</div> }.into_view(),
                            Console::Compile(e) => view! { <div class="verdict-h v-bad">"Compile Error"</div><pre class="err-pre">{e}</pre> }.into_view(),
                            Console::Timeout => view! { <div class="verdict-h v-warn">"Time Limit Exceeded"<small>{format!("killed after {} s — check for an infinite loop", DEADLINE_MS / 1000)}</small></div> }.into_view(),
                            Console::Done { submit, cases, ms } => {
                                let passed = cases.iter().filter(|c| c.ok).count();
                                let total = cases.len();
                                let rte = cases.iter().any(|c| c.err.is_some());
                                let (label, cls) = if passed == total { (if submit { "Accepted" } else { "All examples pass" }, "v-ok") } else if rte { ("Runtime Error", "v-bad") } else { ("Wrong Answer", "v-bad") };
                                let cases2 = cases.clone();
                                view! {
                                    <div class=format!("verdict-h {cls}")>{label}<small>{format!("{passed}/{total} passed · {ms:.2} ms")}</small></div>
                                    <div class="cases">
                                        {cases.iter().enumerate().map(|(i, c)| {
                                            let ok = c.ok;
                                            let hidden = submit && i >= problem().visible;
                                            view! {
                                                <button class:on=move || case_i.get() == i on:click=move |_| set_case_i.set(i)>
                                                    <i class:f=!ok></i>{if hidden { format!("Hidden {}", i + 1) } else { format!("Case {}", i + 1) }}
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                    {move || cases2.get(case_i.get()).cloned().map(|c| view! {
                                        <div class="io">
                                            <div><label>"Input"</label><pre>{c.input}</pre></div>
                                            {match c.err {
                                                Some(e) => view! { <div><label>"Error"</label><pre class="err-pre">{e}</pre></div> }.into_view(),
                                                None => view! { <div><label>"Output"</label><pre class:v-bad=!c.ok>{c.got}</pre></div> }.into_view(),
                                            }}
                                            <div><label>"Expected (Rust reference)"</label><pre>{c.expected}</pre></div>
                                            {(!c.logs.is_empty()).then(|| view! { <div><label>"stdout"</label><pre>{c.logs.join("\n")}</pre></div> })}
                                        </div>
                                    })}
                                }.into_view()
                            }
                        }}
                    </div>
                </div>
            </section>
        </div>
    }
}
