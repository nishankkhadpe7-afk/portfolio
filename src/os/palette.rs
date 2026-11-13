//! Command palette (Ctrl+K): one fuzzy box over apps, projects, certificates,
//! blueprints, practice problems, themes, wallpapers, actions and shell
//! commands. Arrow keys move, Enter runs, Esc closes.

use super::icons::Icon;
use crate::apps::{AppId, APPS};
use crate::data::{blueprints::BLUEPRINTS, certs::CERTS, problems::PROBLEMS, projects::PROJECTS};
use crate::state::{os, Os, Phase, Theme, Wallpaper};
use crate::util::{dom, fuzzy};
use leptos::*;

#[derive(Clone, PartialEq)]
enum Act {
    Open(AppId, Option<String>),
    Theme(Theme),
    Wall(Wallpaper),
    Run(&'static str),
    Lock,
    Reboot,
    Crt,
    Ferris,
    Motion,
    Url(&'static str),
}

#[derive(Clone, PartialEq)]
struct Item {
    icon: &'static str,
    title: String,
    sub: String,
    group: &'static str,
    act: Act,
}

fn catalogue(os: &Os) -> Vec<Item> {
    let mut v = Vec::new();
    let it = |icon, title: &str, sub: &str, group, act| Item { icon, title: title.into(), sub: sub.into(), group, act };
    for a in APPS {
        v.push(it(a.icon, a.label, a.desc, "App", Act::Open(a.id, None)));
    }
    for p in PROJECTS {
        v.push(it(p.icon, p.name, p.tag, "Project", Act::Open(AppId::Files, Some(p.id.into()))));
    }
    for p in crate::data::projects::live() {
        v.push(it("ext", &format!("{} (live site)", p.name), p.live.unwrap_or(""), "Link", Act::Url(p.live.unwrap_or(""))));
    }
    for b in BLUEPRINTS {
        v.push(it("blueprint", &format!("{} blueprint", b.title), "System design diagram", "Blueprint", Act::Open(AppId::Blueprints, Some(b.id.into()))));
    }
    for c in CERTS {
        v.push(it("award", c.title, c.issuer, "Certificate", Act::Open(AppId::Certs, Some(c.id.into()))));
    }
    for p in PROBLEMS {
        v.push(it("braces", &format!("{}. {}", p.n, p.title), &format!("{} · {}", p.diff, p.tags.join(", ")), "Problem", Act::Open(AppId::CodeLab, Some(p.id.into()))));
    }
    let unlocked = os.root_unlocked.get_untracked();
    for t in Theme::ALL.into_iter().filter(|t| *t != Theme::Root || unlocked) {
        v.push(it("droplet", &format!("Theme: {}", t.name()), "Switch colour scheme", "Theme", Act::Theme(t)));
    }
    for w in Wallpaper::BUILTIN {
        v.push(it("image", &format!("Wallpaper: {}", w.name()), w.blurb(), "Wallpaper", Act::Wall(w)));
    }
    for (cmd, sub) in [
        ("neofetch", "system summary"),
        ("nmap -sV localhost", "scan this portfolio's open services"),
        ("git log", "the journey as a commit history"),
        ("cargo run", "build NishankOS (again)"),
        ("cargo test", "run the candidate's test suite"),
        ("sudo hire-me", "escalate privileges"),
        ("fortune", "a random truth"),
        ("tree", "the virtual filesystem"),
    ] {
        v.push(it("terminal", cmd, sub, "Command", Act::Run(cmd)));
    }
    v.push(it("lock", "Lock screen", "", "Action", Act::Lock));
    v.push(it("power", "Reboot", "replay the boot sequence", "Action", Act::Reboot));
    v.push(it("spark", "Toggle CRT scanlines", "retro monitor effect", "Action", Act::Crt));
    v.push(it("crab", "Toggle Ferris", "the desktop crab", "Action", Act::Ferris));
    v.push(it("bolt", "Toggle motion", "pause every animation", "Action", Act::Motion));
    v.push(it("ext", "GitHub", crate::data::profile::ME.github, "Link", Act::Url(crate::data::profile::ME.github)));
    v.push(it("ext", "LinkedIn", crate::data::profile::ME.linkedin, "Link", Act::Url(crate::data::profile::ME.linkedin)));
    v
}

fn run(os: Os, a: Act) {
    os.palette.set(false);
    match a {
        Act::Open(app, arg) => os.open(app, arg),
        Act::Theme(t) => os.theme.set(t),
        Act::Wall(w) => os.wallpaper.set(w),
        Act::Run(cmd) => os.open(AppId::Terminal, Some(format!("run:{cmd}"))),
        Act::Lock => os.phase.set(Phase::Locked),
        Act::Reboot => os.reboot(),
        Act::Crt => os.crt.update(|c| *c = !*c),
        Act::Ferris => os.ferris.update(|c| *c = !*c),
        Act::Motion => os.motion.update(|c| *c = !*c),
        Act::Url(u) => dom::open_url(u),
    }
}

#[component]
pub fn Palette() -> impl IntoView {
    let os = os();
    let all = store_value(catalogue(&os));
    let (q, set_q) = create_signal(String::new());
    let (sel, set_sel) = create_signal(0usize);
    let input = create_node_ref::<html::Input>();
    // Focus once the element is actually in the document.
    input.on_load(|el| {
        request_animation_frame(move || {
            let _ = el.focus();
        });
    });

    let results = create_memo(move |_| {
        let q = q.get();
        all.with_value(|all| {
            let mut v: Vec<(i32, Item)> = all
                .iter()
                .filter_map(|i| fuzzy::score(&q, &format!("{} {}", i.title, i.group)).map(|s| (s, i.clone())))
                .collect();
            if q.is_empty() {
                v.retain(|(_, i)| matches!(i.group, "App" | "Project" | "Command"));
            } else {
                v.sort_by_key(|x| std::cmp::Reverse(x.0));
                // Drop weak, scattered subsequence matches once strong ones exist.
                if let Some(best) = v.first().map(|x| x.0) {
                    v.retain(|x| x.0 * 2 >= best);
                }
            }
            v.into_iter().take(40).map(|(_, i)| i).collect::<Vec<_>>()
        })
    });

    let key = move |e: ev::KeyboardEvent| {
        // The global Esc handler may already have closed (disposed) the
        // palette by the time this delegated handler runs — bail quietly.
        let Some(n) = results.try_with_untracked(|r| r.len()) else { return };
        match e.key().as_str() {
            "ArrowDown" => {
                e.prevent_default();
                set_sel.update(|s| *s = (*s + 1) % n.max(1));
            }
            "ArrowUp" => {
                e.prevent_default();
                set_sel.update(|s| *s = (*s + n.max(1) - 1) % n.max(1));
            }
            "Enter" => {
                if let Some(i) = results.try_with_untracked(|r| r.get(sel.get_untracked()).cloned()).flatten() {
                    run(os, i.act);
                }
            }
            "Escape" => os.palette.set(false),
            _ => {}
        }
    };

    view! {
        <div class="pal-scrim" on:pointerdown=move |_| os.palette.set(false)>
            <div class="palette" role="dialog" aria-label="Command palette" on:pointerdown=|e| e.stop_propagation()>
                <label class="pal-in">
                    <Icon name="command"/>
                    <input node_ref=input placeholder="Type a command, app, project, theme…" aria-label="Command"
                        prop:value=q on:input=move |e| { set_q.set(event_target_value(&e)); set_sel.set(0); } on:keydown=key/>
                    <kbd>"esc"</kbd>
                </label>
                <div class="pal-list" role="listbox">
                    {move || {
                        let s = sel.get();
                        results.get().into_iter().enumerate().map(|(i, it)| {
                            let act = it.act.clone();
                            view! {
                                <button class="pal-item" class:sel=i == s role="option" aria-selected=(i == s).to_string()
                                    on:mouseenter=move |_| set_sel.set(i)
                                    on:click=move |_| run(os, act.clone())>
                                    <Icon name=it.icon/>
                                    <span class="pi-t"><b>{it.title.clone()}</b><small>{it.sub.clone()}</small></span>
                                    <em>{it.group}</em>
                                </button>
                            }
                        }).collect_view()
                    }}
                    <Show when=move || results.with(|r| r.is_empty())>
                        <p class="pal-empty">"No matches. Try “radar”, “root”, or “hire”."</p>
                    </Show>
                </div>
                <div class="pal-foot"><span><kbd>"↑↓"</kbd>" navigate"</span><span><kbd>"↵"</kbd>" run"</span><span>{move || format!("{} results", results.with(|r| r.len()))}</span></div>
            </div>
        </div>
    }
}
