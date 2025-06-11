use crate::{views::Theme, Route};
use dioxus::prelude::*;

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
fn load_from_storage(_key: &str) -> Option<String> {
    None
}

#[component]
pub fn Settings() -> Element {
    let mut api_key = use_signal(|| load_from_storage("api_key").unwrap_or_default());
    let mut provider =
        use_signal(|| load_from_storage("provider").unwrap_or_else(|| "openai".into()));
    let mut theme = use_context::<Signal<Theme>>();

    rsx! {
        div { class: "p-4 space-y-4 max-w-md mx-auto",
            h1 { class: "text-xl font-bold mb-2", "Settings" }

            div { class: "space-x-2",
                label { "Theme:" }
                select {
                    class: "border border-gray-700 rounded p-1 bg-transparent",
                    value: match theme() {
                        Theme::System => "system",
                        Theme::Light  => "light",
                        Theme::Dark   => "dark",
                    },
                    onchange: move |e| match e.value().as_str() {
                        "light"  => theme.set(Theme::Light),
                        "dark"   => theme.set(Theme::Dark),
                        _        => theme.set(Theme::System),
                    },
                    option { value: "system", "System" }
                    option { value: "light",  "Light"  }
                    option { value: "dark",   "Dark"   }
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
