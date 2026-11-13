//! The Terminal app: a zsh-flavoured shell over a virtual filesystem.

mod commands;
mod fs;
mod markup;

use crate::data::profile::ME;
use crate::os::icons::SIGIL;
use crate::state::{os, Win};
use crate::util::dom;
use commands::{Out, Shell};
use leptos::*;
use markup::{Line, Link};
use std::time::Duration;

const NEOFETCH_ART: &str = "      ▄▄████▄▄\n   ▄██▀▀    ▀▀██▄\n  ██  ██▄    ██  ██\n  ██  ██▀█▄  ██  ██\n  ██  ██  ▀█▄██  ██\n   ▀██▄▄    ▄▄██▀\n      ▀▀████▀▀";

#[component]
pub fn Terminal(win: Win) -> impl IntoView {
    let os = os();
    let lines = create_rw_signal(Vec::<(u32, Line)>::new());
    let counter = store_value(0u32);
    let shell = store_value(Shell { cwd: "~".into(), hist: Vec::new() });
    let (cwd, set_cwd) = create_signal("~".to_string());
    let (input, set_input) = create_signal(String::new());
    let (busy, set_busy) = create_signal(false);
    let hist_i = store_value(0usize);
    let job = store_value(0u32); // bump to cancel a running stream
    let body = create_node_ref::<html::Div>();
    let inp = create_node_ref::<html::Input>();

    let push = move |v: Vec<Line>| {
        lines.update(|ls| {
            for l in v {
                counter.update_value(|c| *c += 1);
                ls.push((counter.get_value(), l));
            }
            let n = ls.len();
            if n > 600 {
                ls.drain(..n - 600);
            }
        });
    };

    let exec = move |raw: String| {
        push(vec![Line::Prompt { cwd: cwd.get_untracked(), cmd: raw.clone() }]);
        let out = shell.try_update_value(|sh| commands::run(sh, os, &raw)).unwrap_or(Out::Nothing);
        hist_i.set_value(shell.with_value(|s| s.hist.len()));
        set_cwd.set(shell.with_value(|s| s.cwd.clone()));
        match out {
            Out::Nothing => {}
            Out::Lines(v) => push(v),
            Out::Clear => lines.set(Vec::new()),
            Out::Exit => {
                push(vec![markup::plain("logout")]);
                set_timeout(move || os.close(win.app), Duration::from_millis(250));
            }
            Out::Stream(v) => {
                set_busy.set(true);
                job.update_value(|j| *j += 1);
                let my = job.get_value();
                let mut at = 0u64;
                let n = v.len();
                for (i, (delay, line)) in v.into_iter().enumerate() {
                    at += delay as u64;
                    set_timeout(
                        move || {
                            if job.get_value() != my {
                                return;
                            }
                            push(vec![line]);
                            if i + 1 == n {
                                set_busy.set(false);
                            }
                        },
                        Duration::from_millis(at),
                    );
                }
            }
        }
    };

    // Keep the newest output in view.
    create_effect(move |_| {
        lines.track();
        busy.track();
        request_animation_frame(move || {
            if let Some(b) = body.get_untracked() {
                b.set_scroll_top(b.scroll_height());
            }
        });
    });

    // Deep links: `run:<command>` from the palette, notifications, etc.
    create_effect(move |_| {
        if let Some(a) = win.arg.get() {
            if let Some(cmd) = a.strip_prefix("run:") {
                exec(cmd.to_string());
            }
            win.arg.set(None);
        }
    });

    // Focus the prompt whenever this window gains focus (not on mobile).
    create_effect(move |_| {
        if os.focused.get() == Some(win.app) && !dom::is_mobile() {
            if let Some(i) = inp.get_untracked() {
                let _ = i.focus();
            }
        }
    });

    push(vec![markup::plain_cls(format!("Last login: {} on pts/0 · guest session", String::from(js_sys::Date::new_0().to_date_string())), "t-m")]);
    if win.arg.get_untracked().is_none() {
        exec("neofetch".into());
    }

    let on_key = move |e: ev::KeyboardEvent| {
        let k = e.key();
        match k.as_str() {
            "Enter" => {
                if busy.get_untracked() {
                    return;
                }
                let v = input.get_untracked();
                set_input.set(String::new());
                exec(v);
            }
            "ArrowUp" | "ArrowDown" => {
                e.prevent_default();
                let n = shell.with_value(|s| s.hist.len());
                let mut i = hist_i.get_value();
                if k == "ArrowUp" && i > 0 {
                    i -= 1;
                } else if k == "ArrowDown" && i < n {
                    i += 1;
                }
                hist_i.set_value(i);
                set_input.set(shell.with_value(|s| s.hist.get(i).cloned().unwrap_or_default()));
            }
            "Tab" => {
                e.prevent_default();
                let v = input.get_untracked();
                let opts = shell.with_value(|sh| commands::complete(sh, &v));
                match opts.len() {
                    0 => {}
                    1 => {
                        let mut parts: Vec<&str> = v.split(' ').collect();
                        parts.pop();
                        let done = &opts[0];
                        let sep = if done.ends_with('/') { "" } else { " " };
                        set_input.set(if parts.is_empty() { format!("{done}{sep}") } else { format!("{} {done}{sep}", parts.join(" ")) });
                    }
                    _ => push(vec![markup::plain_cls(opts.join("  "), "t-m")]),
                }
            }
            "l" | "L" if e.ctrl_key() => {
                e.prevent_default();
                lines.set(Vec::new());
            }
            "c" | "C" if e.ctrl_key() => {
                let sel = window().get_selection().ok().flatten().map(|s| String::from(s.to_string())).unwrap_or_default();
                if sel.is_empty() {
                    e.prevent_default();
                    job.update_value(|j| *j += 1);
                    set_busy.set(false);
                    push(vec![Line::Prompt { cwd: cwd.get_untracked(), cmd: format!("{}^C", input.get_untracked()) }]);
                    set_input.set(String::new());
                }
            }
            _ => {}
        }
    };

    let focus_input = move |_| {
        let has_sel = window().get_selection().ok().flatten().map(|s| !s.is_collapsed()).unwrap_or(false);
        if !has_sel {
            if let Some(i) = inp.get_untracked() {
                let _ = i.focus();
            }
        }
    };

    let render_line = move |l: Line| -> View {
        match l {
            Line::Prompt { cwd, cmd } => view! {
                <div class="t-pr">
                    <span class="t-b">"┌──("</span><span class="t-u">{format!("{}㉿{}", ME.handle, ME.host)}</span>
                    <span class="t-b">")-["</span><span class="t-d">{cwd}</span><span class="t-b">"]"</span>
                    <br/><span class="t-b">"└─"</span><span class="t-u">"$ "</span><span>{cmd}</span>
                </div>
            }
            .into_view(),
            Line::Neofetch(rows) => view! {
                <div class="nf">
                    <pre>{NEOFETCH_ART}</pre>
                    <div>
                        <span class="t-u">{ME.handle}</span>"@"<span class="t-u">{ME.host}</span><br/>
                        <span class="t-m">"──────────────────────────"</span><br/>
                        {rows.into_iter().map(|(k, v)| view! { <span class="t-b">{format!("{k:<9}")}</span>{v}<br/> }).collect_view()}
                        <span class="sw" style="background:var(--void)"></span><span class="sw" style="background:var(--line2)"></span>
                        <span class="sw" style="background:var(--blue)"></span><span class="sw" style="background:var(--ice)"></span>
                        <span class="sw" style="background:var(--ok)"></span><span class="sw" style="background:var(--warn)"></span>
                        <span class="sw" style="background:var(--crit)"></span>
                    </div>
                    <span class="nf-sigil" inner_html=SIGIL></span>
                </div>
            }
            .into_view(),
            Line::Segs(segs) => view! {
                <div>
                    {segs.into_iter().map(|s| match s.link {
                        Link::None => view! { <span class=s.cls>{s.text}</span> }.into_view(),
                        Link::Cmd(c) => view! {
                            <span class="t-link" role="button" tabindex="0" on:click=move |e| { e.stop_propagation(); if !busy.get_untracked() { exec(c.clone()) } }>{s.text}</span>
                        }.into_view(),
                        Link::Url(u) => view! { <a class="t-link" href=u target="_blank" rel="noopener">{s.text}</a> }.into_view(),
                    }).collect_view()}
                    {"\u{200b}"}
                </div>
            }
            .into_view(),
        }
    };

    view! {
        <div class="term" node_ref=body on:click=focus_input>
            <div class="t-out">
                <For each=move || lines.get() key=|l| l.0 children=move |(_, l)| render_line(l)/>
            </div>
            <div class="t-in" class:busy=busy>
                <div class="t-pr">
                    <span class="t-b">"┌──("</span><span class="t-u">{format!("{}㉿{}", ME.handle, ME.host)}</span>
                    <span class="t-b">")-["</span><span class="t-d">{cwd}</span><span class="t-b">"]"</span>
                </div>
                <div class="ln">
                    <span class="t-b">"└─"</span><span class="t-u">"$"</span>
                    <input node_ref=inp aria-label="Terminal input" spellcheck="false" autocomplete="off" autocapitalize="off"
                        prop:value=input on:input=move |e| set_input.set(event_target_value(&e)) on:keydown=on_key/>
                </div>
            </div>
        </div>
    }
}
