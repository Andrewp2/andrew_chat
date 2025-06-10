use dioxus::prelude::*;
use crate::Route;
use api::{Attachment, ChatMessage};
#[cfg(feature = "web")]
use base64::engine::general_purpose::STANDARD as BASE64;
#[cfg(feature = "web")]
use base64::Engine;
#[cfg(feature = "web")]
use gloo_file::{futures::read_as_bytes, File};
use crate::speech::{speak, start_stt};
use api::ChatMessage;
use pulldown_cmark::{html, Options, Parser};
#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "web")]
#[wasm_bindgen(inline_js = "export function highlight_all() { if (window.hljs) { window.hljs.highlightAll(); } }")]
extern "C" {
    fn highlight_all();
}

fn markdown_to_html(text: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(text, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[cfg(feature = "web")]
fn load_from_storage(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
}

#[cfg(not(feature = "web"))]
fn load_from_storage(_key: &str) -> Option<String> { None }

#[component]
fn ChatBase(id: Option<usize>) -> Element {
    let mut conversations = use_signal(|| Vec::<usize>::new());
    let mut current = use_signal(|| None::<usize>);
    let mut messages = use_signal(|| Vec::<ChatMessage>::new());
    let attachment = use_signal(|| None::<Attachment>);
    let mut input = use_signal(|| String::new());
    let mut search = use_signal(|| String::new());
    let mut model = use_signal(|| String::from("gpt-3.5"));
    let theme = use_signal(|| String::from("system"));
    let provider = use_signal(|| load_from_storage("provider").unwrap_or_else(|| "openai".into()));
    let api_key = use_signal(|| load_from_storage("api_key").unwrap_or_default());

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
        let init = id;
        spawn(async move {
            let mut list = api::list_conversations().await.unwrap_or_default();
            if list.is_empty() {
                if let Ok(id) = api::create_conversation().await {
                    list.push(id);
                }
            }
            if let Some(cid) = init {
                while list.len() <= cid {
                    if let Ok(new_id) = api::create_conversation().await {
                        list.push(new_id);
                    } else {
                        break;
                    }
                }
                current.set(Some(cid));
            } else {
                current.set(list.first().cloned());
            }
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

    let mut last_len = use_signal(|| 0usize);
    use_effect(move || {
        if messages().len() > last_len() {
            if let Some(text) = messages().last() {
                speak(text);
            }
            last_len.set(messages().len());
        }
    #[cfg(feature = "web")]
    use_effect(move || {
        messages();
        highlight_all();
        ()
    });

    let on_send = move |_| {
        let text = input().clone();
        let attach = attachment();
        let mut msgs: Signal<Vec<ChatMessage>> = messages.clone();
        let mut input_signal = input.clone();
        let mut attachment_signal = attachment.clone();
        let conv = current().unwrap_or(0);
        let selected_model = model().clone();
        async move {
            if !text.is_empty() {
                // store the user's text
                api::send_message(conv, ChatMessage::Text(text.clone())).await.ok();

                // if the selected model supports images, generate one
                if selected_model == "dall-e" {
                    if let Ok(img) = api::generate_image(text.clone()).await {
                        api::send_message(conv, ChatMessage::Image(img)).await.ok();
                    }
                }

        let model_sel = model();
        let provider_sel = provider();
        let key_sel = api_key();
        async move {
            if !text.is_empty() || attach.is_some() {
                api::send_message(
                    conv,
                    ChatMessage { text: if text.is_empty() { None } else { Some(text) }, attachment: attach },
                )
                .await
                .ok();
            if !text.is_empty() {
                let user_msg = text.clone();
                api::send_message(conv, user_msg.clone()).await.ok();
                if let Ok(all) = api::get_messages(conv).await {
                    msgs.set(all);
                }
                if let Ok(resp) = api::chat_completion(provider_sel, key_sel, user_msg, model_sel).await {
                    api::send_message(conv, resp).await.ok();
                    if let Ok(all) = api::get_messages(conv).await {
                        msgs.set(all);
                    }
                }
                input_signal.set(String::new());
                attachment_signal.set(None);
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
                                if let Some(text) = &msg.text {
                                    p { "{text}" }
                                }
                                if let Some(att) = &msg.attachment {
                                    if att.content_type.starts_with("image/") {
                                        img {
                                            src: "data:{att.content_type};base64,{att.data}",
                                            class: "max-w-xs",
                                        }
                                    } else if att.content_type == "application/pdf" {
                                        a {
                                            href: "data:{att.content_type};base64,{att.data}",
                                            target: "_blank",
                                            "View PDF"
                                        }

                            for (idx , msg) in messages().iter().enumerate() {
                                div { class: if idx % 2 == 0 { "flex justify-end" } else { "flex justify-center" },
                                    p {
                                        class: if idx % 2 == 0 { if is_dark() {
                                            "bg-gray-700 text-white rounded px-2 py-1 mb-2 max-w-md"
                                        } else {
                                            "bg-gray-200 text-black rounded px-2 py-1 mb-2 max-w-md"
                                        } } else { "mb-2 max-w-md" },
                                        match msg {
                                    ChatMessage::Text(t) => rsx! {
                                        p { dangerous_inner_html: "{markdown_to_html(msg)}" }
                                    },
                                    ChatMessage::Image(data) => rsx! {
                                        img { src: "{data}" }
                                    },
                                        
                                    }
                                }
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
                        input {
                            r#type: "file",
                            accept: "image/*,application/pdf",
                            onchange: move |_e| {
                                #[cfg(feature = "web")]
                                if let Some(list) = _e.files() {
                                    if let Some(file) = list.first() {
                                        let gfile: File = file.clone().into();
                                        let name = gfile.name().to_string();
                                        let mime = gfile.raw_mime_type();
                                        let mut attach_sig = attachment.clone();
                                        spawn(async move {
                                            if let Ok(bytes) = read_as_bytes(&gfile).await {
                                                let data = BASE64.encode(&bytes);
                                                attach_sig
                                                    .set(
                                                        Some(Attachment {
                                                            filename: name,
                                                            content_type: mime,
                                                            data,
                                                        }),
                                                    );
                                            }
                                        });
                                    }
                                }
                            },
                        }
                        button {
                            class: "bg-gray-700 text-white rounded px-2",
                            onclick: move |_| {
                                start_stt({
                                    let mut input = input.clone();
                                    move |t| input.set(t)
                                });
                            },
                            "🎤"
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
                            option { value: "dall-e", "DALL-E" }
                            option { value: "gpt-o3", "GPT-0.3" }
                            option { value: "gpt-4o", "GPT-4o" }
                            option { value: "gpt-4o-mini", "GPT-4o Mini" }
                            option { value: "gpt-04-mini", "GPT-04 Mini" }
                            option { value: "gpt-4.1", "GPT-4.1" }
                            option { value: "gpt-4.1-mini", "GPT-4.1 Mini" }
                            option { value: "gpt-4.1-nano", "GPT-4.1 Nano" }
                            option { value: "gpt-4.5", "GPT-4.5" }
                            option { value: "gemini-2.5-flash", "Gemini 2.5 Flash" }
                            option { value: "gemini-2.5-pro", "Gemini 2.5 Pro" }
                            option { value: "claude-4-sonnet", "Claude 4 Sonnet" }
                            option { value: "claude-4-sonnet-reasoning", "Claude 4 Sonnet Reasoning" }
                            option { value: "deepseek-r1", "Deepseek r1" }
                            option { value: "deepseek-v3", "Deepseek v3" }
                            option { value: "llama-4-scout", "Llama 4 Scout" }
                            option { value: "qwen-2.5-32b", "Qwen 2.5 32B" }
                            option { value: "grok-3", "Grok 3" }
                            option { value: "grok-3-mini", "Grok 3 Mini" }
                        }
                        Link {
                            to: Route::ChatShare {
                                id: current().unwrap_or(0),
                            },
                            class: "underline text-sm",
                            "Share"
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

#[component]
pub fn Chat() -> Element {
    rsx!(
        ChatBase { id: None }
    )
}

#[component]
pub fn ChatShare(id: usize) -> Element {
    rsx!(
        ChatBase { id: Some(id) }
    )
}
