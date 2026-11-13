//! The top panel: launcher, quick-launch, task list and tray.

use super::icons::{Icon, Sigil};
use crate::apps::AppId;
use crate::state::{os, Phase};
use crate::util::dom;
use leptos::*;

#[component]
pub fn Panel() -> impl IntoView {
    let os = os();
    let (clock, set_clock) = create_signal(dom::clock(false));
    let (mem, set_mem) = create_signal(dom::wasm_memory_bytes());
    dom::use_interval(1000, move || {
        set_clock.set(dom::clock(false));
        set_mem.set(dom::wasm_memory_bytes());
    });

    let quick = [AppId::Terminal, AppId::Files, AppId::CodeLab, AppId::Cypheron];

    view! {
        <header class="panel" role="toolbar" aria-label="Panel">
            <button class="pbtn brand" class:on=move || os.menu.get() aria-haspopup="true" aria-label="Applications menu"
                on:click=move |e| { e.stop_propagation(); os.menu.update(|m| *m = !*m); }>
                <Sigil class="sm"/><b>"NishankOS"</b>
            </button>
            <span class="psep"></span>
            {quick.into_iter().map(|a| {
                let m = a.meta();
                view! {
                    <button class="pbtn" class:hide-sm=a == AppId::Cypheron title=m.title aria-label=m.label on:click=move |_| os.open(a, None)>
                        <Icon name=m.icon/>
                    </button>
                }
            }).collect_view()}
            <span class="psep"></span>
            <nav class="tasks" aria-label="Open windows">
                <For each=move || os.wins.get() key=|w| w.app let:w>
                    {
                        let m = w.app.meta();
                        view! {
                            <button class="task"
                                class:on=move || os.focused.get() == Some(w.app) && !w.min.get()
                                class:min=move || w.min.get()
                                title=m.title
                                on:click=move |_| os.task_click(w.app)>
                                <Icon name=m.icon/><span>{m.label}</span>
                            </button>
                        }
                    }
                </For>
            </nav>
            <div class="tray">
                <button class="pbtn hide-sm" title="Command palette (Ctrl+K)" aria-label="Command palette" on:click=move |_| os.palette.set(true)>
                    <Icon name="command"/>
                </button>
                <button class="pbtn hide-sm" title="Overview (F3)" aria-label="Window overview" on:click=move |_| os.expose.update(|x| *x = !*x)>
                    <Icon name="grid"/>
                </button>
                <button class="pbtn hide-sm" title="Cycle theme" aria-label="Cycle theme" on:click=move |_| os.cycle_theme()>
                    <Icon name="droplet"/>
                </button>
                <button class="meter-chip hide-sm" title="WebAssembly linear memory · open System Monitor" on:click=move |_| os.open(AppId::Monitor, None)>
                    <Icon name="cpu"/>{move || format!("{:.1} MiB", mem.get() / 1048576.0)}
                </button>
                <span class="pbtn hide-sm" title="Connected · wlan0" aria-label="Network connected"><Icon name="wifi"/></span>
                <span class="clock">{clock}</span>
                <button class="pbtn" title="Lock screen" aria-label="Lock screen" on:click=move |_| os.phase.set(Phase::Locked)>
                    <Icon name="lock"/>
                </button>
            </div>
        </header>
    }
}
