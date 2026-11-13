//! Packages: skills as a `cargo`-style crate table, filterable and searchable.

use crate::data::packages::{version, PKGS, SECTIONS};
use crate::os::icons::Icon;
use crate::util::fuzzy;
use leptos::*;

#[component]
pub fn Packages() -> impl IntoView {
    let (q, set_q) = create_signal(String::new());
    let (sec, set_sec) = create_signal(None::<&'static str>);
    let rows = move || {
        let q = q.get();
        let s = sec.get();
        PKGS.iter()
            .filter(|p| s.is_none_or(|s| p.1 == s))
            .filter(|p| fuzzy::score(&q, &format!("{} {}", p.0, p.2)).is_some())
            .collect::<Vec<_>>()
    };
    view! {
        <div class="pk">
            <div class="pk-bar">
                <label class="m-search"><Icon name="search"/>
                    <input placeholder="Search crates…" prop:value=q on:input=move |e| set_q.set(event_target_value(&e)) aria-label="Search packages"/>
                </label>
                <div class="flt">
                    <button class:on=move || sec.get().is_none() on:click=move |_| set_sec.set(None)>"all"</button>
                    {SECTIONS.iter().map(|s| view! {
                        <button class:on=move || sec.get() == Some(*s) on:click=move |_| set_sec.set(Some(*s))>{*s}</button>
                    }).collect_view()}
                </div>
            </div>
            <div class="pk-tbl">
                <table>
                    <thead><tr><th>"crate"</th><th>"version"</th><th>"section"</th><th>"used in"</th></tr></thead>
                    <tbody>
                        {move || rows().into_iter().map(|(n, s, u)| view! {
                            <tr><td>{*n}</td><td class="ver">{version(n)}</td><td class="sec">{*s}</td><td class="use">{*u}</td></tr>
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
            <div class="pk-foot">
                {move || format!("    Finished · {} of {} crates shown · 0 vulnerabilities (cargo audit)", rows().len(), PKGS.len())}
            </div>
        </div>
    }
}
