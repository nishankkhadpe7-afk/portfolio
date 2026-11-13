//! ICE Breaker: a breach-protocol puzzle. Pick codes alternately along the
//! active row and column to assemble the daemon sequences in your buffer.
//! Every puzzle is generated from a hidden solution path, so all three
//! daemons are always solvable. Upload ROOT_SHELL to unlock the root theme.

use crate::os::icons::Icon;
use crate::state::{os, Theme};
use crate::util::{dom, rng::Rng};
use leptos::*;

const N: usize = 6;
const BUF: usize = 7;
const CODES: &[&str] = &["1C", "55", "BD", "E9", "7A", "FF"];
const DAEMONS: &[(&str, &str)] = &[
    ("DATAMINE_V1", "exfiltrates a fun fact"),
    ("ICEPICK", "lowers the firewall"),
    ("ROOT_SHELL", "grants uid=0 — unlocks the root theme"),
];
const TIME: f64 = 45.0;

#[derive(Clone, Copy, PartialEq)]
enum Axis {
    Row(usize),
    Col(usize),
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Ready,
    Running,
    Over,
}

#[derive(Clone, PartialEq)]
struct Puzzle {
    grid: Vec<Vec<&'static str>>,
    daemons: Vec<Vec<&'static str>>,
}

fn generate(seed: u32) -> Puzzle {
    let mut r = Rng::new(seed);
    let mut grid: Vec<Vec<&str>> = (0..N).map(|_| (0..N).map(|_| *r.pick(CODES)).collect()).collect();
    // Walk a valid path: row 0 → column → row → …, never reusing a cell.
    let mut path: Vec<(usize, usize)> = Vec::new();
    let (mut row, mut col) = (0usize, 0usize);
    for step in 0..BUF {
        loop {
            let k = r.int(N);
            let cell = if step % 2 == 0 { (row, k) } else { (k, col) };
            if !path.contains(&cell) {
                path.push(cell);
                row = cell.0;
                col = cell.1;
                break;
            }
        }
    }
    let codes: Vec<&str> = path.iter().map(|_| *r.pick(CODES)).collect();
    for (&(pr, pc), &c) in path.iter().zip(&codes) {
        grid[pr][pc] = c;
    }
    let daemons = [2usize, 3, 4]
        .iter()
        .map(|&len| {
            let off = r.int(BUF - len + 1);
            codes[off..off + len].to_vec()
        })
        .collect();
    Puzzle { grid, daemons }
}

fn contains(hay: &[&str], needle: &[&str]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

#[component]
pub fn Breach() -> impl IntoView {
    let os = os();
    let seed = create_rw_signal(js_sys::Date::now() as u32);
    let puzzle = create_memo(move |_| generate(seed.get()));
    let picks = create_rw_signal(Vec::<(usize, usize)>::new());
    let (axis, set_axis) = create_signal(Axis::Row(0));
    let (phase, set_phase) = create_signal(Phase::Ready);
    let (left, set_left) = create_signal(TIME);
    let (hover, set_hover) = create_signal(None::<(usize, usize)>);
    let rewarded = store_value(false);

    let buffer = move || -> Vec<&'static str> {
        let p = puzzle.get();
        picks.get().iter().map(|&(r, c)| p.grid[r][c]).collect()
    };
    let solved = move |i: usize| contains(&buffer(), &puzzle.with(|p| p.daemons[i].clone()));

    let finish = move || {
        if phase.get_untracked() == Phase::Over {
            return;
        }
        set_phase.set(Phase::Over);
        if rewarded.get_value() {
            return;
        }
        rewarded.set_value(true);
        let got: Vec<usize> = (0..3).filter(|&i| untrack(|| solved(i))).collect();
        if got.contains(&2) {
            os.unlock_root();
            os.theme.set(Theme::Root);
        }
        if got.is_empty() {
            os.toast("shield", "Breach failed", "The ICE held. Try again — every puzzle is solvable.", None);
        } else {
            let names: Vec<&str> = got.iter().map(|&i| DAEMONS[i].0).collect();
            os.toast("spark", format!("Breach successful · {}/3", got.len()), format!("Uploaded {}.", names.join(", ")), None);
        }
    };

    dom::use_interval(100, move || {
        if phase.get_untracked() == Phase::Running {
            let l = (left.get_untracked() - 0.1).max(0.0);
            set_left.set(l);
            if l <= 0.0 {
                finish();
            }
        }
    });

    let reset = move |new: bool| {
        if new {
            seed.update(|s| *s = s.wrapping_mul(1664525).wrapping_add(1013904223));
        }
        picks.set(Vec::new());
        set_axis.set(Axis::Row(0));
        set_phase.set(Phase::Ready);
        set_left.set(TIME);
        rewarded.set_value(false);
    };

    let click = move |r: usize, c: usize| {
        if phase.get_untracked() == Phase::Over || picks.with_untracked(|p| p.contains(&(r, c))) {
            return;
        }
        let ok = match axis.get_untracked() {
            Axis::Row(i) => i == r,
            Axis::Col(j) => j == c,
        };
        if !ok {
            return;
        }
        set_phase.set(Phase::Running);
        picks.update(|p| p.push((r, c)));
        set_axis.set(match axis.get_untracked() {
            Axis::Row(_) => Axis::Col(c),
            Axis::Col(_) => Axis::Row(r),
        });
        let all = (0..3).all(|i| untrack(|| solved(i)));
        if picks.with_untracked(|p| p.len()) >= BUF || all {
            finish();
        }
    };

    let in_axis = move |r: usize, c: usize| match axis.get() {
        Axis::Row(i) => i == r,
        Axis::Col(j) => j == c,
    };
    let preview = move |r: usize, c: usize| {
        hover.get().is_some_and(|(hr, hc)| in_axis(hr, hc) && match axis.get() { Axis::Row(_) => hc == c, Axis::Col(_) => hr == r })
    };

    view! {
        <div class="br">
            <div class="br-head">
                <div>
                    <div class="eyebrow">"breach protocol"</div>
                    <b class="br-title">"ICE Breaker"</b>
                </div>
                <div class="br-timer" class:low=move || { left.get() < 10.0 }>
                    <small>"breach time remaining"</small>
                    <b>{move || format!("{:05.2}", left.get())}</b>
                    <i style:width=move || format!("{}%", left.get() / TIME * 100.0)></i>
                </div>
            </div>
            <div class="br-main">
                <div class="br-matrix" role="grid" aria-label="Code matrix">
                    {move || {
                        let p = puzzle.get();
                        (0..N).map(|r| view! {
                            <div class="br-row" role="row">
                                {(0..N).map(|c| {
                                    let code = p.grid[r][c];
                                    view! {
                                        <button role="gridcell" class="br-cell"
                                            class:axis=move || in_axis(r, c) && phase.get() != Phase::Over
                                            class:used=move || picks.get().contains(&(r, c))
                                            class:preview=move || preview(r, c)
                                            on:mouseenter=move |_| set_hover.set(Some((r, c)))
                                            on:mouseleave=move |_| set_hover.set(None)
                                            on:click=move |_| click(r, c)>
                                            {move || if picks.get().contains(&(r, c)) { "[ ]" } else { code }}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        }).collect_view()
                    }}
                </div>
                <div class="br-side">
                    <div class="eyebrow">"buffer"</div>
                    <div class="br-buf">
                        {move || {
                            let b = buffer();
                            (0..BUF).map(|i| view! { <span class:filled=i < b.len()>{b.get(i).copied().unwrap_or("")}</span> }).collect_view()
                        }}
                    </div>
                    <div class="eyebrow">"sequences to upload"</div>
                    <div class="br-daemons">
                        {(0..3).map(|i| view! {
                            <div class="br-d" class:ok=move || solved(i) class:fail=move || phase.get() == Phase::Over && !solved(i)>
                                <div class="br-seq">{move || puzzle.get().daemons[i].iter().map(|c| view! { <code>{*c}</code> }).collect_view()}</div>
                                <div><b>{DAEMONS[i].0}</b><small>{DAEMONS[i].1}</small></div>
                                <em>{move || if solved(i) { "UPLOADED" } else if phase.get() == Phase::Over { "FAILED" } else { "" }}</em>
                            </div>
                        }).collect_view()}
                    </div>
                    <div class="pj-actions">
                        <button class="btn pri" on:click=move |_| reset(true)><Icon name="reset"/>"New puzzle"</button>
                        <button class="btn" on:click=move |_| reset(false)>"Retry"</button>
                    </div>
                    <p class="br-help">"Start in the highlighted top row. Each pick switches the active line: row → column → row. Sequences can overlap, and every puzzle has a path that uploads all three."</p>
                </div>
            </div>
        </div>
    }
}
