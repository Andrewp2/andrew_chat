use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn Chat() -> Element {
    let mut conversations = use_signal(|| Vec::<usize>::new());
    let mut current = use_signal(|| None::<usize>);
    let mut messages = use_signal(|| Vec::<String>::new());
    let mut input = use_signal(|| String::new());
    let mut search = use_signal(|| String::new());
    let mut model = use_signal(|| String::from("gpt-3.5"));
    let mut theme = use_signal(|| String::from("system"));

    let is_dark = move || {
        match theme().as_str() {
            "dark" => true,
            "system" => {
                #[cfg(feature = "web")]
                {
                    web_sys::window()
                        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
                        .map(|m| m.matches())
                        .unwrap_or(false)
                }
                #[cfg(not(feature = "web"))]
                {
                    false
                }
            }
            _ => false,
        }
    };

    // Load available conversations on mount
    use_effect(move || {
        spawn(async move {
            let mut list = api::list_conversations().await.unwrap_or_default();
            if list.is_empty() {
                if let Ok(id) = api::create_conversation().await {
                    list.push(id);
                }
            }
            current.set(list.first().cloned());
            conversations.set(list);
        });
        ()
    });

    // Load messages whenever the current conversation changes
    use_effect(move || {
        let id = current();
        spawn(async move {
            if let Some(cid) = id {
                if let Ok(msgs) = api::get_messages(cid).await {
                    messages.set(msgs);
                }
            }
        });
        ()
    });

    let on_send = move |_| {
        let text = input().clone();
        let mut msgs: Signal<Vec<String>> = messages.clone();
        let mut input_signal = input.clone();
        let conv = current().unwrap_or(0);
        async move {
            if !text.is_empty() {
                api::send_message(conv, text).await.ok();
                if let Ok(all) = api::get_messages(conv).await {
                    msgs.set(all);
                }
                input_signal.set(String::new());
            }
        }
    };

    let on_new_conv = move |_| {
        let mut convs = conversations.clone();
        let mut cur = current.clone();
        async move {
            if let Ok(id) = api::create_conversation().await {
                let mut list = convs();
                list.push(id);
                convs.set(list);
                cur.set(Some(id));
            }
        }
    };

    let filtered: Vec<usize> = conversations()
        .into_iter()
        .filter(|cid| cid.to_string().to_lowercase().contains(&search().to_lowercase()))
        .collect();

    let sidebar = rsx! {
        div { class: "w-48 border-r border-gray-700 p-2 flex flex-col h-full",
            button {
                class: "mb-2 p-1 bg-gray-700 text-white rounded",
                onclick: move |_| {
                    spawn(on_new_conv(()));
                },
                "New Chat"
            }
            input {
                class: "mb-2 pb-1 border-b border-gray-300 bg-transparent outline-none",
                r#type: "text",
                placeholder: "Search...",
                value: "{search}",
                oninput: move |e| search.set(e.value()),
            }
            ul { class: "flex-1 overflow-y-auto list-none p-0",
                for cid in filtered.iter().cloned() {
                    li {
                        class: if Some(cid) == current() { "bg-gray-800 p-1" } else { "p-1" },
                        onclick: move |_| current.set(Some(cid)),
                        "Conversation {cid}"
                    }
                }
            }
            Link { to: Route::Settings {}, class: "mt-2 text-left text-sm", "Account" }
        }
    };

    rsx! {
        div { class: if is_dark() { "dark bg-gray-900 text-white flex flex-col h-screen font-sans p-4" } else { "bg-white text-black flex flex-col h-screen font-sans p-4" },
            div { class: "flex flex-1 overflow-hidden",
                {sidebar}
                div { class: "flex flex-col flex-1 pl-4",
                    div { class: if messages().is_empty() { "flex-1 border border-gray-700 p-2 overflow-y-auto flex items-center justify-center" } else { "flex-1 border border-gray-700 p-2 overflow-y-auto" },
                        if !messages().is_empty() {
                            for msg in messages().iter() {
                                p { "{msg}" }
                            }
                        }
                    }
                    div { class: if messages().is_empty() { "flex gap-2 mt-4" } else { "flex gap-2 mt-2" },
                        input {
                            class: "flex-1 p-1 border border-gray-700 rounded",
                            value: "{input}",
                            oninput: move |e| input.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    spawn(on_send(()));
                                }
                            },
                            placeholder: "Type a message...",
                        }
                        button {
                            class: "bg-gray-700 text-white rounded px-2",
                            onclick: move |_| {
                                spawn(on_send(()));
                            },
                            "Send"
                        }
                        select {
                            class: "border border-gray-700 rounded p-1",
                            value: "{model}",
                            onchange: move |e| model.set(e.value()),
                            option { value: "gpt-3.5", "GPT-3.5" }
                            option { value: "gpt-4", "GPT-4" }
                        }
                        Link {
                            to: Route::Settings {},
                            class: "underline text-sm",
                            "Settings"
                        }
                    }
                }
            }
        }
    }
}
