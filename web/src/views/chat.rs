use dioxus::prelude::*;

const CHAT_CSS: Asset = asset!("/assets/chat.css");

#[component]
pub fn Chat() -> Element {
    let mut conversations = use_signal(|| Vec::<usize>::new());
    let mut current = use_signal(|| None::<usize>);
    let mut messages = use_signal(|| Vec::<String>::new());
    let mut input = use_signal(|| String::new());
    let mut search = use_signal(|| String::new());
    let mut model = use_signal(|| String::from("gpt-3.5"));
    let mut dark = use_signal(|| false);

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

    let show_sidebar = !(messages().is_empty() && conversations().len() <= 1);
    let filtered: Vec<usize> = conversations()
        .into_iter()
        .filter(|cid| cid.to_string().contains(&search()))
        .collect();

    let sidebar = if show_sidebar {
        rsx! {
            div { id: "sidebar",
                input {
                    r#type: "text",
                    placeholder: "Search...",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
                button {
                    onclick: move |_| {
                        spawn(on_new_conv(()));
                    },
                    "New"
                }
                ul { class: "conversations",
                    for cid in filtered.iter().cloned() {
                        li {
                            class: if Some(cid) == current() { "active" } else { "" },
                            onclick: move |_| current.set(Some(cid)),
                            "Conversation {cid}"
                        }
                    }
                }
                div { class: "account", "Logged in" }
            }
        }
    } else {
        rsx!(div {})
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CHAT_CSS }
        div { id: "chat-container", class: if dark() { "dark" } else { "" },
            div { id: "top-bar",
                div { class: "model-select",
                    select {
                        value: "{model}",
                        onchange: move |e| model.set(e.value()),
                        option { value: "gpt-3.5", "GPT-3.5" }
                        option { value: "gpt-4", "GPT-4" }
                    }
                }
                div { class: "settings",
                    button { "Settings" }
                    label {
                        "Dark"
                        input {
                            r#type: "checkbox",
                            checked: dark(),
                            onchange: move |_| dark.set(!dark()),
                        }
                    }
                }
            }
            div { id: "main-area",
                {sidebar}
                div { id: "chat",
                    div { id: "messages",
                        for msg in messages().iter() {
                            p { "{msg}" }
                        }
                    }
                    div { id: "input-area",
                    div { class: "input-container",
                        input {
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
                            onclick: move |_| {
                                spawn(on_send(()));
                            },
                            "Send"
                        }
                    }
                }
              }

            }
        }
    }
}
