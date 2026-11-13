//! AvIAI: a 60 NM approach radar. Aircraft kinematics, the sweep, short-term
//! conflict alerts (STCA) and the AI controller's clearances all run in Rust;
//! the canvas is repainted every animation frame.

use crate::os::icons::Icon;
use crate::state::os;
use crate::util::{dom, rng::Rng, thousands};
use leptos::*;
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::f64::consts::{PI, TAU};
use std::rc::Rc;

const AIRLINES: &[&str] = &["IGO", "AIC", "AKJ", "AXB", "UAE", "QTR", "SIA", "BAW", "VTI", "DLH"];
const RANGE_NM: f64 = 60.0;

#[derive(Clone)]
struct Plane {
    cs: String,
    x: f64,
    y: f64,
    hdg: f64,
    spd: f64,
    alt: f64,
    dep: bool,
    lit: f64,
    trail: VecDeque<(f64, f64)>,
}

impl Plane {
    fn spawn(r: &mut Rng, dep: bool) -> Plane {
        let ang = r.f() * TAU;
        let rad = if dep { r.range(2.0, 6.0) } else { r.range(44.0, 59.0) };
        let to_c = (-ang.sin()).atan2(-ang.cos());
        Plane {
            cs: format!("{}{}", r.pick(AIRLINES), 100 + r.int(8900)),
            x: ang.cos() * rad,
            y: ang.sin() * rad,
            hdg: if dep { ang + r.range(-0.3, 0.3) } else { to_c + r.range(-0.25, 0.25) },
            spd: if dep { r.range(180.0, 270.0) } else { r.range(230.0, 290.0) },
            alt: if dep { 3000.0 + r.int(5) as f64 * 1000.0 } else { 12000.0 + r.int(14) as f64 * 1000.0 },
            dep,
            lit: 0.3,
            trail: VecDeque::new(),
        }
    }
    fn dist(&self) -> f64 {
        self.x.hypot(self.y)
    }
    fn fl(&self) -> String {
        format!("{:03}", (self.alt / 100.0).round() as i64)
    }
}

/// Compass heading (1–360) from a math angle.
fn to_hdg(a: f64) -> i64 {
    let d = ((90.0 - a.to_degrees()).round() as i64).rem_euclid(360);
    if d == 0 { 360 } else { d }
}

#[derive(Clone, PartialEq)]
struct Strip {
    cs: String,
    dep: bool,
    nm: i64,
    fl: String,
    spd: i64,
}

#[derive(Clone, PartialEq)]
struct Clearance {
    id: u32,
    time: String,
    cs: String,
    msg: String,
    alert: bool,
}

#[component]
pub fn Aviai() -> impl IntoView {
    let os = os();
    let rng = Rc::new(RefCell::new(Rng::new(42)));
    let planes: Rc<RefCell<Vec<Plane>>> = Rc::new(RefCell::new((0..10).map(|i| Plane::spawn(&mut rng.borrow_mut(), i % 3 == 0)).collect()));
    let strips = create_rw_signal(Vec::<Strip>::new());
    let log = create_rw_signal(Vec::<Clearance>::new());
    let log_n = store_value(0u32);
    let (selected, set_selected) = create_signal(None::<String>);
    let (conflicts, set_conflicts) = create_signal(0usize);
    let canvas = create_node_ref::<html::Canvas>();

    let push_log = move |cs: String, msg: String, alert: bool| {
        log_n.update_value(|n| *n += 1);
        let id = log_n.get_value();
        log.update(|l| {
            l.insert(0, Clearance { id, time: dom::clock(true), cs, msg, alert });
            l.truncate(16);
        });
    };

    let refresh_strips = {
        let planes = planes.clone();
        move || {
            let mut v: Vec<Strip> = planes
                .borrow()
                .iter()
                .map(|p| Strip { cs: p.cs.clone(), dep: p.dep, nm: p.dist().round() as i64, fl: p.fl(), spd: p.spd.round() as i64 })
                .collect();
            v.sort_by_key(|s| s.nm);
            strips.set(v);
        }
    };

    let clearance = {
        let (planes, rng) = (planes.clone(), rng.clone());
        move || {
            let mut ps = planes.borrow_mut();
            let mut r = rng.borrow_mut();
            let i = r.int(ps.len());
            let p = &mut ps[i];
            let turn = if r.chance(0.5) { "left" } else { "right" };
            let msg = if p.dep {
                p.alt = (p.alt + 4000.0).min(36000.0);
                format!("climb and maintain FL{}, turn {turn} heading {:03}, resume own navigation", p.fl(), to_hdg(p.hdg))
            } else {
                p.alt = (p.alt - 3000.0 - r.int(3) as f64 * 1000.0).max(3000.0);
                p.spd = (p.spd - 20.0).max(170.0);
                let lvl = if p.alt >= 10000.0 { format!("FL{}", p.fl()) } else { format!("{} ft", thousands(p.alt as i64)) };
                format!("descend and maintain {lvl}, turn {turn} heading {:03}, reduce speed {} kt", to_hdg((-p.y).atan2(-p.x)), p.spd.round())
            };
            let cs = p.cs.clone();
            drop(ps);
            drop(r);
            push_log(cs, msg, false);
        }
    };

    refresh_strips();
    for _ in 0..3 {
        clearance();
    }
    dom::use_interval(2800, clearance);
    dom::use_interval(1500, refresh_strips);

    // ── the scope ─────────────────────────────────────────────────────
    let alerted: Rc<RefCell<HashSet<(String, String)>>> = Rc::new(RefCell::new(HashSet::new()));
    let sel_cs: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    {
        let (planes, rng, sel_cs) = (planes.clone(), rng.clone(), sel_cs.clone());
        let mut sweep = 0.0f64;
        let mut trail_t = 0.0f64;
        dom::use_raf(move |_, dt| {
            let Some(cv) = canvas.get_untracked() else { return };
            let dt = if os.motion.get_untracked() { dt } else { 0.0 };
            let (w, h, _) = dom::fit_canvas(&cv, 2.0);
            let Some(cx) = dom::ctx2d(&cv) else { return };
            let (acc, ice) = os.theme.get_untracked().rgb();
            let c = |(r, g, b): (u8, u8, u8), a: f64| format!("rgba({r},{g},{b},{a:.3})");
            let (ox, oy) = (w / 2.0, h / 2.0);
            let rad = w.min(h) / 2.0 - 20.0;
            let k = rad / RANGE_NM;
            sweep = (sweep + dt * 1.1) % TAU;
            trail_t += dt;
            let drop_trail = trail_t > 0.5;
            if drop_trail {
                trail_t = 0.0;
            }

            // kinematics
            let mut ps = planes.borrow_mut();
            for p in ps.iter_mut() {
                let v = p.spd / 3600.0 * 40.0;
                p.x += p.hdg.cos() * v * dt;
                p.y += p.hdg.sin() * v * dt;
                if !p.dep {
                    let to_c = (-p.y).atan2(-p.x);
                    let d = (to_c - p.hdg).sin().atan2((to_c - p.hdg).cos());
                    p.hdg += d * dt * 0.08;
                }
                if drop_trail {
                    p.trail.push_front((p.x, p.y));
                    p.trail.truncate(6);
                }
                if (!p.dep && p.dist() < 2.5) || (p.dep && p.dist() > RANGE_NM) {
                    *p = Plane::spawn(&mut rng.borrow_mut(), !p.dep);
                }
                let a = p.y.atan2(p.x).rem_euclid(TAU);
                // sweep rotates clockwise on screen = negative math angle
                let diff = (sweep - (TAU - a)).rem_euclid(TAU);
                p.lit = if diff < 0.07 { 1.0 } else { (p.lit - dt * 0.22).max(0.25) };
            }

            // STCA: < 5 NM laterally and < 1000 ft vertically
            let mut pairs = Vec::new();
            for i in 0..ps.len() {
                for j in (i + 1)..ps.len() {
                    let (a, b) = (&ps[i], &ps[j]);
                    if (a.x - b.x).hypot(a.y - b.y) < 5.0 && (a.alt - b.alt).abs() < 1000.0 {
                        pairs.push((i, j));
                    }
                }
            }
            if pairs.len() != conflicts.get_untracked() {
                set_conflicts.set(pairs.len());
            }
            for &(i, j) in &pairs {
                let key = (ps[i].cs.clone(), ps[j].cs.clone());
                if alerted.borrow_mut().insert(key.clone()) {
                    // Resolve: the controller vectors one aircraft away.
                    ps[j].alt += 2000.0;
                    let msg = format!("traffic alert — climb immediately FL{}, conflict with {}", ps[j].fl(), key.0);
                    push_log(key.1, msg, true);
                }
            }

            // paint
            cx.clear_rect(0.0, 0.0, w, h);
            cx.set_line_width(1.0);
            cx.set_font("10px 'IBM Plex Mono', monospace");
            for n in [15.0, 30.0, 45.0, 60.0] {
                cx.set_stroke_style_str(&c(acc, 0.28));
                cx.begin_path();
                let _ = cx.arc(ox, oy, n * k, 0.0, TAU);
                cx.stroke();
                cx.set_fill_style_str(&c(ice, 0.55));
                let _ = cx.fill_text(&format!("{n}NM"), ox + n * k * 0.71 + 3.0, oy - n * k * 0.71 - 3.0);
            }
            cx.set_stroke_style_str(&c(acc, 0.14));
            cx.begin_path();
            cx.move_to(ox - rad, oy);
            cx.line_to(ox + rad, oy);
            cx.move_to(ox, oy - rad);
            cx.line_to(ox, oy + rad);
            cx.stroke();
            cx.set_fill_style_str(&c(ice, 0.55));
            for d in (0..360).step_by(30) {
                let a = (90.0 - d as f64).to_radians();
                let _ = cx.fill_text(&format!("{d:03}"), ox + a.cos() * (rad + 6.0) - 9.0, oy - a.sin() * (rad + 6.0) + 4.0);
            }
            // runway 09/27 (VABB-ish)
            cx.set_stroke_style_str(&c(ice, 1.0));
            cx.set_line_width(3.0);
            cx.begin_path();
            cx.move_to(ox - 8.0, oy + 3.0);
            cx.line_to(ox + 8.0, oy - 3.0);
            cx.stroke();
            cx.set_line_width(1.0);
            // sweep wedge
            if let Ok(g) = cx.create_radial_gradient(ox, oy, 0.0, ox, oy, rad) {
                let _ = g.add_color_stop(0.0, &c(acc, 0.0));
                let _ = g.add_color_stop(1.0, &c(acc, 0.24));
                cx.save();
                cx.begin_path();
                cx.move_to(ox, oy);
                let _ = cx.arc(ox, oy, rad, sweep - 0.5, sweep);
                cx.close_path();
                cx.set_fill_style_canvas_gradient(&g);
                cx.fill();
                cx.restore();
            }
            cx.set_stroke_style_str(&c(ice, 0.85));
            cx.set_line_width(1.5);
            cx.begin_path();
            cx.move_to(ox, oy);
            cx.line_to(ox + sweep.cos() * rad, oy + sweep.sin() * rad);
            cx.stroke();
            cx.set_line_width(1.0);

            let sel = sel_cs.borrow().clone();
            for &(i, j) in &pairs {
                let (a, b) = (&ps[i], &ps[j]);
                cx.set_stroke_style_str("rgba(255,92,112,.9)");
                cx.begin_path();
                cx.move_to(ox + a.x * k, oy - a.y * k);
                cx.line_to(ox + b.x * k, oy - b.y * k);
                cx.stroke();
                for p in [a, b] {
                    cx.begin_path();
                    let _ = cx.arc(ox + p.x * k, oy - p.y * k, 11.0, 0.0, TAU);
                    cx.stroke();
                }
            }
            for p in ps.iter() {
                let (x, y) = (ox + p.x * k, oy - p.y * k);
                let col = if p.dep { (69, 216, 147) } else { ice };
                for (n, (tx, ty)) in p.trail.iter().enumerate() {
                    cx.set_fill_style_str(&c(col, 0.35 - n as f64 * 0.05));
                    cx.fill_rect(ox + tx * k - 1.0, oy - ty * k - 1.0, 2.0, 2.0);
                }
                cx.set_fill_style_str(&c(col, p.lit));
                cx.fill_rect(x - 3.0, y - 3.0, 6.0, 6.0);
                cx.set_stroke_style_str(&c(col, p.lit * 0.6));
                cx.begin_path();
                cx.move_to(x, y);
                cx.line_to(x + p.hdg.cos() * 16.0, y - p.hdg.sin() * 16.0);
                cx.stroke();
                if sel.as_deref() == Some(p.cs.as_str()) {
                    cx.set_stroke_style_str(&c(ice, 1.0));
                    cx.stroke_rect(x - 8.0, y - 8.0, 16.0, 16.0);
                }
                cx.set_fill_style_str(&c((220, 232, 248), 0.35 + p.lit * 0.6));
                let _ = cx.fill_text(&p.cs, x + 8.0, y - 6.0);
                cx.set_fill_style_str(&c((139, 162, 195), 0.3 + p.lit * 0.5));
                let _ = cx.fill_text(&format!("{} {}", p.fl(), p.spd.round()), x + 8.0, y + 5.0);
            }
            let _ = PI;
        });
    }

    let pick = {
        let (planes, sel_cs) = (planes.clone(), sel_cs.clone());
        move |e: ev::MouseEvent| {
            let Some(cv) = canvas.get_untracked() else { return };
            let r = cv.get_bounding_client_rect();
            let (mx, my) = (e.client_x() as f64 - r.left(), e.client_y() as f64 - r.top());
            let (w, h) = (r.width(), r.height());
            let k = (w.min(h) / 2.0 - 20.0) / RANGE_NM;
            let best = planes
                .borrow()
                .iter()
                .map(|p| (((w / 2.0 + p.x * k) - mx).hypot((h / 2.0 - p.y * k) - my), p.cs.clone()))
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .filter(|b| b.0 < 24.0)
                .map(|b| b.1);
            *sel_cs.borrow_mut() = best.clone();
            set_selected.set(best);
        }
    };

    view! {
        <div class="av">
            <div class="av-scope">
                <canvas node_ref=canvas on:click=pick aria-label="Radar scope with simulated aircraft — click a target to select it"></canvas>
                <div class="av-hud">
                    "VABB · MUMBAI APP · 60 NM · "{move || format!("{} tracks", strips.get().len())}
                    <Show when=move || { conflicts.get() > 0 }>
                        <span class="stca">{move || format!(" · STCA ×{}", conflicts.get())}</span>
                    </Show>
                </div>
                {move || selected.get().and_then(|cs| strips.get().into_iter().find(|s| s.cs == cs)).map(|s| view! {
                    <div class="av-sel">
                        <b>{s.cs.clone()}</b>
                        <span>{if s.dep { "DEPARTURE" } else { "ARRIVAL" }}</span>
                        <span>{format!("FL{} · {} kt · {} NM", s.fl, s.spd, s.nm)}</span>
                    </div>
                })}
            </div>
            <aside class="av-side">
                <h4>"Traffic strips"</h4>
                <div class="av-strips">
                    <For each=move || strips.get() key=|s| s.cs.clone() let:s>
                        <div class="strip" class:dep=s.dep class:sel={let cs = s.cs.clone(); move || selected.get().as_deref() == Some(cs.as_str())}>
                            <b>{s.cs.clone()}</b><span>{format!("{} {}NM", if s.dep { "DEP" } else { "ARR" }, s.nm)}</span><span>{format!("FL{}", s.fl)}</span>
                        </div>
                    </For>
                </div>
                <h4><Icon name="spark"/>"AI-ATC clearances"</h4>
                <ol class="av-log">
                    <For each=move || log.get() key=|c| c.id let:c>
                        <li class:alert=c.alert>
                            <time>{format!("{}Z · {}", c.time, if c.alert { "stca-guard" } else { "mistral-agent" })}</time>
                            <b>{c.cs.clone()}</b>", "{c.msg.clone()}"."
                        </li>
                    </For>
                </ol>
            </aside>
        </div>
        <p class="demo-note">"Simulated in Rust/WASM · click a target to inspect it. Production: live OpenSky → Redis, clearances phrased by Mistral."</p>
    }
}
