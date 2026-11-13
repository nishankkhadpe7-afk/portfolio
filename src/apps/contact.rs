//! Contact: copyable details, a vCard download, and a message composer that
//! hands off to the visitor's own mail client (no backend, no tracking).

use crate::data::profile::ME;
use crate::os::icons::Icon;
use crate::state::Win;
use crate::util::dom;
use leptos::*;

pub fn vcard() -> String {
    format!(
        "BEGIN:VCARD\nVERSION:3.0\nFN:{}\nN:Khadpe;Nishank;;;\nEMAIL;TYPE=INTERNET:{}\nTEL;TYPE=CELL:{}\nURL:{}\nURL:{}\nADR;TYPE=HOME:;;{};;;;\nNOTE:{}\nEND:VCARD\n",
        ME.name, ME.email, ME.phone, ME.linkedin, ME.github, ME.loc, ME.status
    )
}

const TOPICS: &[(&str, &str)] = &[
    ("Internship", "Hi Nishank,\n\nI came across NishankOS and would love to talk about an internship on our team.\n\n"),
    ("Collaboration", "Hi Nishank,\n\nI'm working on something you might like to build with me:\n\n"),
    ("Hackathon team", "Hi Nishank,\n\nWe're forming a team for an upcoming hackathon and could use your security + AI skills.\n\n"),
    ("Just saying hi", "Hi Nishank,\n\nJust wanted to say the Rust OS portfolio is great.\n\n"),
];

#[component]
fn Row(icon: &'static str, label: &'static str, value: &'static str, #[prop(optional)] href: Option<String>) -> impl IntoView {
    let (copied, set_copied) = create_signal(false);
    view! {
        <div class="cn-row">
            <Icon name=icon/>
            <span>{label}</span>
            <div class="v">{match href {
                Some(h) => view! { <a href=h target="_blank" rel="noopener">{value.trim_start_matches("https://")}</a> }.into_view(),
                None => value.into_view(),
            }}</div>
            <button on:click=move |_| {
                dom::copy(value);
                set_copied.set(true);
                set_timeout(move || set_copied.set(false), std::time::Duration::from_millis(1300));
            }>{move || if copied.get() { "copied ✓" } else { "copy" }}</button>
        </div>
    }
}

#[component]
pub fn Contact(win: Win) -> impl IntoView {
    let (topic, set_topic) = create_signal(0usize);
    let (msg, set_msg) = create_signal(TOPICS[0].1.to_string());
    let (from, set_from) = create_signal(String::new());
    create_effect(move |_| {
        if win.arg.get().as_deref() == Some("hire") {
            set_topic.set(0);
            set_msg.set(TOPICS[0].1.to_string());
            win.arg.set(None);
        }
    });
    let mailto = move || {
        let subject = format!("{} — via NishankOS", TOPICS[topic.get()].0);
        let body = format!("{}{}", msg.get(), if from.get().is_empty() { String::new() } else { format!("\n— {}", from.get()) });
        format!(
            "mailto:{}?subject={}&body={}",
            ME.email,
            String::from(js_sys::encode_uri_component(&subject)),
            String::from(js_sys::encode_uri_component(&body))
        )
    };
    view! {
        <div class="cn">
            <div>
                <div class="eyebrow">"contact"</div>
                <h2 class="h1">"Let's build something."</h2>
                <span class="status-pill"><i></i>{ME.status}</span>
            </div>
            <div class="cn-rows">
                <Row icon="mail" label="email" value=ME.email href=format!("mailto:{}", ME.email)/>
                <Row icon="phone" label="phone" value=ME.phone href=format!("tel:{}", ME.phone.replace(' ', ""))/>
                <Row icon="link" label="linkedin" value=ME.linkedin href=ME.linkedin.to_string()/>
                <Row icon="code" label="github" value=ME.github href=ME.github.to_string()/>
                <Row icon="pin" label="based in" value=ME.loc/>
            </div>
            <div class="cn-compose">
                <div class="eyebrow">"compose"</div>
                <div class="sugg">
                    {TOPICS.iter().enumerate().map(|(i, (t, body))| view! {
                        <button class:on=move || topic.get() == i on:click=move |_| { set_topic.set(i); set_msg.set(body.to_string()); }>{*t}</button>
                    }).collect_view()}
                </div>
                <input class="cn-from" placeholder="Your name (optional)" prop:value=from on:input=move |e| set_from.set(event_target_value(&e)) aria-label="Your name"/>
                <textarea prop:value=msg on:input=move |e| set_msg.set(event_target_value(&e)) aria-label="Message"></textarea>
                <div class="pj-actions">
                    <a class="btn pri" href=mailto><Icon name="mail"/>"Open in mail app"</a>
                    <button class="btn" on:click=move |_| dom::copy(&msg.get_untracked())><Icon name="copy"/>"Copy message"</button>
                    <button class="btn" on:click=move |_| dom::download("nishank-khadpe.vcf", "text/vcard", &vcard())><Icon name="download"/>"vCard"</button>
                </div>
                <small class="muted">"Nothing is sent from this page — your mail client does the sending."</small>
            </div>
        </div>
    }
}
