//! Lock screen. Any key or click unlocks — it is a portfolio, not a vault.

use super::icons::Sigil;
use crate::data::profile::ME;
use crate::state::{os, Phase};
use crate::util::dom;
use leptos::*;

#[component]
pub fn Lock() -> impl IntoView {
    let os = os();
    let (time, set_time) = create_signal(dom::clock(false));
    let (leaving, set_leaving) = create_signal(false);
    dom::use_interval(1000, move || set_time.set(dom::clock(false)));
    let unlock = move || {
        set_leaving.set(true);
        set_timeout(move || os.phase.set(Phase::Desktop), std::time::Duration::from_millis(380));
    };
    let key = window_event_listener(ev::keydown, move |_| unlock());
    on_cleanup(move || key.remove());
    view! {
        <div id="lock" class:out=leaving on:click=move |_| unlock() role="button" aria-label="Unlock">
            <div class="lk-time">{time}</div>
            <div class="lk-date">{dom::date_long()}</div>
            <div class="lk-user">
                <Sigil class="lk-av"/>
                <b>{ME.name}</b>
                <span class="lk-pass">"••••••••"<i class="lk-caret"></i></span>
            </div>
            <div class="lk-hint">"click or press any key to unlock"</div>
        </div>
    }
}
