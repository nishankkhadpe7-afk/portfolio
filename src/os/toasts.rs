//! Notification stack, top-right. Click a toast to open its app.

use super::icons::Icon;
use crate::state::os;
use leptos::*;

#[component]
pub fn Toasts() -> impl IntoView {
    let os = os();
    view! {
        <div class="toasts" aria-live="polite">
            <For each=move || os.toasts.get() key=|t| t.id let:t>
                {
                    let id = t.id;
                    let open = t.open;
                    view! {
                        <div class="toast" role="status" on:click=move |_| {
                            os.dismiss_toast(id);
                            if let Some(a) = open { os.open(a, None) }
                        }>
                            <Icon name=t.icon/>
                            <div><b>{t.title.clone()}</b><span>{t.body.clone()}</span></div>
                            <i class="toast-life"></i>
                        </div>
                    }
                }
            </For>
        </div>
    }
}
