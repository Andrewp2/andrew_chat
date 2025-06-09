use dioxus::prelude::*;

const CHAT_CSS: Asset = asset!("/assets/chat.css");

#[component]
pub fn Chat() -> Element {
    let mut messages = use_signal(|| Vec::<String>::new());
    let mut input = use_signal(|| String::new());

    use_effect(move || {
        spawn(async move {
            if let Ok(msgs) = api::get_messages().await {
                messages.set(msgs);
            }
        });
        ()
    });

    let on_send = move |_| {
        let text = input().clone();
        let mut msgs = messages.clone();
        let mut input_signal = input.clone();
        async move {
            if !text.is_empty() {
                api::send_message(text).await.ok();
                if let Ok(all) = api::get_messages().await {
                    msgs.set(all);
                }
                input_signal.set(String::new());
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CHAT_CSS }
        div { id: "chat", 
            div { id: "messages", 
                for msg in messages().iter() {
                    p { "{msg}" }
                }
            }
            div { id: "input-area",
                input { 
                    value: "{input}",
                    oninput: move |e| input.set(e.value()),
                    placeholder: "Type a message...",
                }
                button { onclick: on_send, "Send" }
            }
        }
    }
}
