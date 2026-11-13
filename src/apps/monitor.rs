//! System Monitor: an htop-style view. CPU numbers are simulated, but the
//! memory, DOM-node count, frame rate, window count and uptime are real
//! measurements of this page.

use crate::apps::AppId;
use crate::os::icons::Icon;
use crate::state::os;
use crate::util::dom::{self, FPS};
use crate::util::rng::Rng;
use leptos::*;

const SERVICES: &[(&str, &str, u32)] = &[
    ("root", "nishank-kernel --target wasm32", 1),
    ("root", "leptos-reactive-runtime", 12),
    ("nishank", "wallpaper-engine --raf", 77),
    ("redis", "redis-server *:6379 (aviai)", 412),
    ("nishank", "uvicorn fintrix.app:api --port 8000", 523),
    ("nishank", "cypheron-worker --modules 7", 601),
    ("nishank", "ferris --patrol desktop", 1337),
];

#[derive(Clone, PartialEq)]
struct Proc {
    pid: u32,
    user: &'static str,
    cpu: f64,
    mem: f64,
    cmd: String,
    app: Option<AppId>,
}

fn spark(points: &[f64], w: f64, h: f64) -> String {
    if points.len() < 2 {
        return String::new();
    }
    let step = w / (points.len() - 1) as f64;
    points.iter().enumerate().map(|(i, v)| format!("{}{:.1},{:.1}", if i == 0 { "M" } else { "L" }, i as f64 * step, h - v / 100.0 * h)).collect()
}

#[component]
pub fn Monitor() -> impl IntoView {
    let os = os();
    let (tick, set_tick) = create_signal(0u32);
    let cores = create_rw_signal(vec![12.0f64, 8.0, 21.0, 5.0]);
    let history = create_rw_signal(vec![10.0f64; 60]);
    let rng = store_value(Rng::from_time());
    let (mem, set_mem) = create_signal(dom::wasm_memory_bytes());
    let (nodes, set_nodes) = create_signal(0u32);
    let (fps, set_fps) = create_signal(60.0);
    let (sort_cpu, set_sort_cpu) = create_signal(true);

    let sample = move || {
        rng.update_value(|r| {
            cores.update(|c| {
                for v in c.iter_mut() {
                    *v = (*v + r.range(-9.0, 9.0) + (os.wins.with_untracked(|w| w.len()) as f64 * 1.5 - *v * 0.08)).clamp(1.0, 97.0);
                }
            });
        });
        let avg = cores.with_untracked(|c| c.iter().sum::<f64>() / c.len() as f64);
        history.update(|h| {
            h.remove(0);
            h.push(avg);
        });
        set_mem.set(dom::wasm_memory_bytes());
        set_nodes.set(document().get_elements_by_tag_name("*").length());
        set_fps.set(FPS.with(|f| f.get()));
        set_tick.update(|t| *t += 1);
    };
    sample();
    dom::use_interval(1000, sample);

    let procs = move || {
        let t = tick.get();
        let mut r = Rng::new(t.wrapping_mul(2654435761));
        let mut v: Vec<Proc> = SERVICES
            .iter()
            .map(|(u, c, pid)| Proc { pid: *pid, user: u, cpu: r.range(0.0, 6.0), mem: r.range(0.4, 3.5), cmd: c.to_string(), app: None })
            .collect();
        for (i, w) in os.wins.get().iter().enumerate() {
            let m = w.app.meta();
            let heavy = matches!(w.app, AppId::Aviai | AppId::Settings | AppId::CodeLab);
            v.push(Proc {
                pid: 2000 + i as u32 * 7 + m.key.len() as u32,
                user: "nishank",
                cpu: if w.min.get() { 0.0 } else { r.range(if heavy { 8.0 } else { 0.5 }, if heavy { 24.0 } else { 6.0 }) },
                mem: r.range(1.0, if heavy { 9.0 } else { 4.0 }),
                cmd: format!("nishank-os --app {}", m.key),
                app: Some(w.app),
            });
        }
        if sort_cpu.get() {
            v.sort_by(|a, b| b.cpu.total_cmp(&a.cpu));
        } else {
            v.sort_by_key(|p| p.pid);
        }
        v
    };

    let kill = move |p: Proc| match p.app {
        Some(a) => {
            os.close(a);
            os.toast("close", format!("SIGTERM → {}", p.pid), format!("{} exited cleanly.", p.cmd), None);
        }
        None => os.toast("lock", "kill: Operation not permitted", format!("PID {} belongs to {}. Nice try.", p.pid, p.user), None),
    };

    view! {
        <div class="mon">
            <div class="mon-top">
                <div class="mon-cores">
                    {(0..4).map(|i| view! {
                        <div class="core">
                            <span>{i}</span>
                            <div class="cbar"><i style:width=move || format!("{:.0}%", cores.get()[i])></i></div>
                            <em>{move || format!("{:>4.1}%", cores.get()[i])}</em>
                        </div>
                    }).collect_view()}
                    <svg class="mon-spark" viewBox="0 0 240 48" preserveAspectRatio="none">
                        <path class="sp-fill" d=move || format!("{} L240,48 L0,48 Z", spark(&history.get(), 240.0, 48.0))></path>
                        <path class="sp-line" d=move || spark(&history.get(), 240.0, 48.0)></path>
                    </svg>
                </div>
                <div class="mon-stats">
                    <div><small>"wasm heap"</small><b>{move || format!("{:.2} MiB", mem.get() / 1048576.0)}</b><i class="real">"real"</i></div>
                    <div><small>"DOM nodes"</small><b>{nodes}</b><i class="real">"real"</i></div>
                    <div><small>"frame rate"</small><b>{move || format!("{:.0} fps", fps.get())}</b><i class="real">"real"</i></div>
                    <div><small>"windows"</small><b>{move || os.wins.get().len()}</b><i class="real">"real"</i></div>
                    <div><small>"uptime"</small><b>{move || { tick.track(); os.uptime() }}</b><i class="real">"real"</i></div>
                    <div><small>"load avg"</small><b>{move || { let h = history.get(); format!("{:.2} {:.2}", h[59] / 40.0, h.iter().sum::<f64>() / 60.0 / 40.0) }}</b></div>
                </div>
            </div>
            <div class="mon-tbl">
                <table>
                    <thead><tr>
                        <th class="click" on:click=move |_| set_sort_cpu.set(false)>"PID"</th><th>"USER"</th>
                        <th class="click" on:click=move |_| set_sort_cpu.set(true)>"CPU% ▾"</th><th>"MEM%"</th><th>"COMMAND"</th><th></th>
                    </tr></thead>
                    <tbody>
                        {move || procs().into_iter().map(|p| {
                            let pk = p.clone();
                            view! {
                                <tr class:mine=p.app.is_some()>
                                    <td>{p.pid}</td><td class="u">{p.user}</td>
                                    <td class="n" class:hot={p.cpu > 15.0}>{format!("{:.1}", p.cpu)}</td>
                                    <td class="n">{format!("{:.1}", p.mem)}</td>
                                    <td class="cmd">{p.cmd.clone()}</td>
                                    <td><button class="kill" title="Send SIGTERM" on:click=move |_| kill(pk.clone())><Icon name="close"/></button></td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
            <p class="demo-note">"CPU figures are simulated; heap, DOM nodes, fps, windows and uptime are measured live. Killing a nishank-os process closes its window."</p>
        </div>
    }
}
