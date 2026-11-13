//! `localStorage` persistence. Every key is namespaced with `kos.` and every
//! failure (private mode, quota, disabled storage) degrades to "no memory".

fn storage() -> Option<web_sys::Storage> {
    leptos::window().local_storage().ok().flatten()
}

pub fn get(key: &str) -> Option<String> {
    storage()?.get_item(&format!("kos.{key}")).ok().flatten()
}

pub fn set(key: &str, val: &str) {
    if let Some(s) = storage() {
        let _ = s.set_item(&format!("kos.{key}"), val);
    }
}

pub fn get_bool(key: &str, default: bool) -> bool {
    get(key).map(|v| v == "1").unwrap_or(default)
}

pub fn set_bool(key: &str, v: bool) {
    set(key, if v { "1" } else { "0" });
}
