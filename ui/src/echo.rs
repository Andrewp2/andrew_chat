use dioxus::prelude::*;

/// Echo component that demonstrates fullstack server functions.
#[component]
pub fn Echo() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        div { class: "w-90 mx-auto mt-12 bg-gray-800 p-5 rounded",
            h4 { "ServerFn Echo" }
            input {
                class: "block w-full bg-transparent text-white border-b border-white focus:border-blue-300 outline-none",
                placeholder: "Type here to echo...",
                oninput: move |event| async move {
                    let data = api::echo(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}
