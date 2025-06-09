use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn Settings() -> Element {
    let mut theme = use_signal(|| String::from("system"));
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
            Link { to: Route::Chat {}, class: "text-blue-500 underline", "Back" }
        }
    }
}
