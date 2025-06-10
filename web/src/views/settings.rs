use dioxus::prelude::*;
use crate::Route;

#[cfg(feature = "web")]
fn save_to_storage(key: &str, value: &str) {
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.set_item(key, value);
        }
    }
}

#[cfg(not(feature = "web"))]
fn save_to_storage(_key: &str, _value: &str) {}

#[cfg(feature = "web")]
fn load_from_storage(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
}

#[cfg(not(feature = "web"))]
fn load_from_storage(_key: &str) -> Option<String> { None }

#[component]
pub fn Settings() -> Element {
    let mut theme = use_signal(|| String::from("system"));
    let mut api_key = use_signal(|| load_from_storage("api_key").unwrap_or_default());
    let mut provider = use_signal(|| load_from_storage("provider").unwrap_or_else(|| "openai".into()));
    rsx! {
        div { class: "p-4 space-y-4",
            h1 { class: "text-xl font-bold", "Settings" }
            div {
                label { class: "mr-2", "Theme:" }
                select { value: "{theme}", onchange: move |e| theme.set(e.value()),
                    option { value: "system", "System" }
                    option { value: "light", "Light" }
                    option { value: "dark", "Dark" }
                }
            }
            div {
                label { class: "mr-2", "Provider:" }
                select {
                    value: "{provider}",
                    onchange: move |e| {
                        save_to_storage("provider", &e.value());
                        provider.set(e.value());
                    },
                    option { value: "openai", "OpenAI" }
                    option { value: "anthropic", "Anthropic" }
                }
            }
            div {
                label { class: "mr-2", "API Key:" }
                input {
                    r#type: "text",
                    value: "{api_key}",
                    oninput: move |e| {
                        save_to_storage("api_key", &e.value());
                        api_key.set(e.value());
                    },
                }
            }
            Link { to: Route::Chat {}, class: "text-blue-500 underline", "Back" }
        }
    }
}
