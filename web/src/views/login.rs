use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn Login() -> Element {
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let navigator = use_navigator();

    let on_login = move |_| {
        let user = username().clone();
        let pass = password().clone();
        let nav = navigator.clone();
        async move {
            if api::login(user, pass).await.unwrap_or(false) {
                nav.push(Route::Chat {});
            }
        }
    };

    let on_register = move |_| {
        let user = username().clone();
        let pass = password().clone();
        async move {
            api::register(user, pass).await.ok();
        }
    };

    rsx! {
        div { class: "flex flex-col space-y-2 p-4",
            h1 { class: "text-xl font-bold", "Login" }
            input {
                class: "border p-1",
                placeholder: "Username",
                value: "{username}",
                oninput: move |e| username.set(e.value()),
            }
            input {
                class: "border p-1",
                r#type: "password",
                placeholder: "Password",
                value: "{password}",
                oninput: move |e| password.set(e.value()),
            }
            button {
                class: "bg-blue-500 text-white p-1",
                onclick: move |_| {
                    spawn(on_login(()));
                },
                "Login"
            }
            button {
                class: "bg-gray-700 text-white p-1",
                onclick: move |_| {
                    spawn(on_register(()));
                },
                "Register"
            }
        }
    }
}
