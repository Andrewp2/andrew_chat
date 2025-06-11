use crate::routes::Route;
use crate::speech::{speak, start_stt};
use crate::views::Theme;
use api::model_config::ModelConfig;
use api::{Attachment, ChatMessage, MessageSender};
use dioxus::prelude::*;
use futures_util::StreamExt;
use katex_wasmbind::KaTeXOptions;
use pulldown_cmark::{html, Options, Parser};

#[cfg(feature = "web")]
use {
    base64::{engine::general_purpose::STANDARD as BASE64, Engine as _},
    wasm_bindgen::prelude::*,
    wasm_bindgen_futures::spawn_local,
};

// TODO: what the hell is going on here
#[cfg(feature = "web")]
#[wasm_bindgen(
    inline_js = "export function highlight_all() { if (window.hljs) { window.hljs.highlightAll(); } }"
)]
extern "C" {
    fn highlight_all();
}

// Max attachment size in bytes is 1mb.
pub const MAX_ATTACHMENT_SIZE: u64 = 1024 * 1024;

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
fn load_from_storage(_key: &str) -> Option<String> {
    None
}

fn render_model_selector(
    mut model: Signal<Option<ModelConfig>>,
    all_models: Vec<ModelConfig>,
) -> Element {
    let current_model_name = model.as_ref().map(|m| m.name.clone());
    let selected_index = all_models
        .iter()
        .position(|m| Some(m.name.clone()) == current_model_name)
        .unwrap_or(0);

    rsx! {
        select {
            class: "border border-gray-700 rounded p-1",
            value: "{selected_index}",
            onchange: move |e| {
              if let Ok(index) = e.value().parse::<usize>() {
                  if let Some(selected_model) = all_models.get(index) {
                      let model_clone = selected_model.clone();
                      model.set(Some(model_clone));
                  }
              }
          },
            for (index, model) in all_models.iter().enumerate() {
                        option {
                            key: "{index}",
                            value: "{index}",
                            "{model.name}"
                        }

                }

        }
    }
}

fn render_message_list(messages: &[ChatMessage], katex_opts: &KaTeXOptions) -> Element {
    rsx! {
        div {
            class: if messages.is_empty() {
                "flex-1 border border-gray-700 p-2 overflow-y-auto flex items-center justify-center"
            } else {
                "flex-1 border border-gray-700 p-2 overflow-y-auto"
            },
            if !messages.is_empty() {
                for (idx, msg) in messages.iter().enumerate() {
                    div {
                        class: if idx % 2 == 0 {
                            "flex justify-end"
                        } else {
                            "flex justify-start"
                        },
                        div {
                            class: if idx % 2 == 0 {
                                "dark:bg-gray-700 dark:text-white rounded px-2 py-1 mb-2 max-w-md"
                            } else {
                                "mb-2 max-w-md"
                            },
                            {render_message(msg, katex_opts)}
                        }
                    }
                }
            }
        }
    }
}

fn render_message_input(
    mut input: Signal<String>,
    on_send: Callback<()>,
    attachment: Signal<Option<Attachment>>,
    is_empty: bool,
) -> Element {
    rsx! {
        div {
            class: if is_empty { "flex gap-2 mt-4" } else { "flex gap-2 mt-2" },
            input {
                class: "flex-1 p-1 border border-gray-700 rounded",
                value: "{input}",
                oninput: move |e| input.set(e.value()),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        on_send(());
                    }
                },
                placeholder: "Type a message...",
            }
            input {
                r#type: "file",
                accept: "image/*,application/pdf",
                onchange: move |e| {
                  async move {
                    if let Some(file_engine) = e.files() {
                        let files = file_engine.files();
                        if let Some(file_name) = files.get(0) {
                            let mut attach_sig = attachment.clone();
                              let file = file_engine.read_file_to_string(&(file_name.clone())).await;
                              if let Some(file) = file {
                              let content_type = match file_name.split('.').last() {
                                Some("pdf") => "application/pdf".to_string(),
                                Some("png") => "image/png".to_string(),
                                Some("jpg") => "image/jpeg".to_string(),
                                Some("jpeg") => "image/jpeg".to_string(),
                                Some("gif") => "image/gif".to_string(),
                                Some("webp") => "image/webp".to_string(),
                                _ => "application/octet-stream".to_string(),
                              };
                              attach_sig.set(Some(Attachment {
                                filename: file_name.clone(),
                                content_type,
                                data: file,
                              }));
                              }
                        }
                    }
                  }
                },
                class: "hidden",
                id: "file-upload",
            }
            button {
                class: "px-4 py-1 bg-blue-500 text-white rounded hover:bg-blue-600",
                onclick: move |_| on_send(()),
                "Send"
            }
        }
    }
}

fn render_message(msg: &ChatMessage, opts: &KaTeXOptions) -> Element {
    // Handle text content with optional markdown/math rendering
    let text_content = msg.text.as_ref().map(|text| {
        // Check for math expressions (either $...$ or $$...$$)
        let is_math = (text.starts_with("$$") && text.ends_with("$$"))
            || (text.starts_with('$') && text.ends_with('$'));

        if is_math {
            let expr = if text.starts_with("$$") {
                text[2..text.len() - 2].trim()
            } else {
                text[1..text.len() - 1].trim()
            };

            let str = opts.render(expr);
            rsx! {
              div {dangerous_inner_html: "{str}"}
            }
        } else {
            let html = markdown_to_html(text);
            rsx! { div { dangerous_inner_html: "{html}" } }
        }
    });

    // Handle attachment if present
    let attachment = msg.attachment.as_ref().map(|att| {
        rsx! {
            div {
                class: "attachment",
                "Attachment: ", {att.filename.clone()}
            }
        }
    });

    // Combine text and attachment
    rsx! {
        div { class: "message-content",
            {text_content}
            {attachment}
        }
    }
}

#[component]
fn ChatBase(id: Option<usize>) -> Element {
    // State management
    let mut conversations = use_signal(Vec::<usize>::new);
    let mut current = use_signal(|| id);
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut attachment = use_signal(|| None::<Attachment>);
    let mut input = use_signal(String::new);
    let mut search = use_signal(String::new);
    let model = use_signal(|| Some(ModelConfig::default()));
    let all_models = use_signal(|| ModelConfig::load_models().unwrap_or_default());
    let mut use_web_search = use_signal(|| false);
    let mut use_image_gen = use_signal(|| false);
    let katex_opts = KaTeXOptions::inline_mode();
    let api_key = use_signal(|| load_from_storage("api_key").unwrap_or_default());
    let _theme = use_context::<Signal<Theme>>();
    let mut last_len = use_signal(|| 0usize);

    // Load conversations
    let conv_res = use_resource(move || {
        let mut current = current.clone();
        async move {
            let mut list = api::list_conversations().await.unwrap_or_default();
            if list.is_empty() {
                if let Ok(id) = api::create_conversation().await {
                    list.push(id);
                    current.set(Some(id));
                }
            } else if current().is_none() {
                current.set(list.first().copied());
            }
            list
        }
    });

    // Update conversations state
    use_effect(move || {
        if let Some(list) = &*conv_res.read_unchecked() {
            conversations.set(list.clone());
        }
    });

    // Load messages for current conversation
    let current_id = current();
    let msg_res = use_resource(move || {
        let current_id = current_id;
        async move {
            if let Some(cid) = current_id {
                api::get_messages(cid).await.unwrap_or_default()
            } else {
                Vec::new()
            }
        }
    });

    // Update messages state
    use_effect(move || {
        if let Some(list) = &*msg_res.read_unchecked() {
            messages.set(list.clone());
        }
    });

    // Set the first model as default if none is selected
    use_effect(move || {
        if model.read().is_none() && !all_models.read().is_empty() {
            model.set(Some(all_models.read()[0].clone()));
        }
    });

    // Load models
    use_effect(move || {
        spawn_local(async move {
            if let Ok(loaded_models) = ModelConfig::load_models() {
                all_models.set(loaded_models);
            } else {
                log::error!("Failed to load models, using default");
                all_models.set(vec![ModelConfig::default()]);
            }
        });
    });

    // Update model when models list changes
    use_effect(move || {
        if !all_models().is_empty() && model().is_none() {
            if let Some(first_model) = all_models().first().cloned() {
                model.set(Some(first_model));
            }
        }
    });

    // Stream new messages
    use_effect(move || {
        let current_id = current();
        let messages = messages.clone();

        spawn_local(async move {
            if let Some(cid) = current_id {
                if let Ok(stream) = api::stream_messages(cid, messages().len()).await {
                    let mut inner = stream.into_inner();
                    while let Some(Ok(chunk)) = inner.next().await {
                        messages.with_mut(|msgs| {
                            // Find the last message from AI
                            if let Some(last_msg) = msgs.last_mut() {
                                if last_msg.sender == MessageSender::AI {
                                    // Append to the last AI message
                                    if let Some(text) = &mut last_msg.text {
                                        text.push_str(&chunk);
                                    } else {
                                        last_msg.text = Some(chunk);
                                    }
                                    return;
                                }
                            }
                            // If no AI message exists or last message is from user, create new AI message
                            msgs.push(ChatMessage {
                                text: Some(chunk),
                                attachment: None,
                                sender: MessageSender::AI,
                            });
                        });
                    }
                }
            }
        });
    });

    // Handle message updates
    use_effect(move || {
        if messages().len() > last_len() {
            if let Some(text) = messages().last() {
                speak(text.text.as_ref().unwrap());
            }
            last_len.set(messages().len());
        }
    });

    // Web-specific effects
    #[cfg(feature = "web")]
    use_effect(move || {
        highlight_all();
    });

    let mut on_send = Callback::from(move |_| {
        let text = input().trim().to_string();
        if text.is_empty() {
            return;
        }

        let current_conv = current();
        let current_model = model();
        let current_attachment = attachment.with(|a| a.clone());

        // Clear input and reset states
        input.set(String::new());
        attachment.set(None);
        use_web_search.set(false);
        use_image_gen.set(false); // Reset image gen flag after sending

        // Create and add user message
        let user_message = ChatMessage {
            text: Some(text.clone()),
            attachment: None,
            sender: MessageSender::User,
        };

        // Add user message to local state immediately
        messages.with_mut(|msgs| {
            msgs.push(user_message);
        });

        spawn_local(async move {
            let Some(conv_id) = current_conv else { return };

            // Create and add user message to backend
            let user_message = ChatMessage {
                text: Some(text.clone()),
                attachment: current_attachment,
                sender: MessageSender::User,
            };

            // Always add user message to the conversation first
            if let Err(e) = api::send_message(conv_id, user_message.clone()).await {
                log::error!("Failed to send user message: {}", e);
                return;
            }

            // Update messages list
            if let Ok(all) = api::get_messages(conv_id).await {
                messages.set(all);
            }

            // Get the current model, defaulting to the first one if none selected
            let current_model = match model() {
                Some(m) => m,
                None => {
                    log::error!("No model selected");
                    return;
                }
            };

            if current_model.capabilities.image_generation && *use_image_gen.read() {
                // Handle image generation
                if let Ok(image_url) = api::generate_image(text.clone()).await {
                    let image_message = ChatMessage {
                        text: Some(format!("Generated image for: {}", text)),
                        attachment: Some(Attachment {
                            filename: "generated_image.png".to_string(),
                            content_type: "image/png".to_string(),
                            data: image_url,
                        }),
                        sender: MessageSender::AI,
                    };
                    if let Err(e) = api::send_message(conv_id, image_message).await {
                        log::error!("Failed to send generated image: {}", e);
                    }
                }
            } else {
                // Handle regular chat completion
                let key_sel = api_key();
                if let Ok(ai_response) =
                    api::chat_completion(key_sel, text.clone(), current_model.clone()).await
                {
                    let ai_message = ChatMessage {
                        text: Some(ai_response),
                        attachment: None,
                        sender: MessageSender::AI,
                    };
                    if let Err(e) = api::send_message(conv_id, ai_message).await {
                        log::error!("Failed to send AI response: {}", e);
                    }
                }
            }

            // Refresh messages after all operations
            if let Ok(all) = api::get_messages(conv_id).await {
                messages.set(all);
            }

            // Clear input and attachment
            input.set(String::new());
            attachment.set(None);
        });
    });

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
        .filter(|cid| {
            cid.to_string()
                .to_lowercase()
                .contains(&search().to_lowercase())
        })
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
      div {
            class: "dark:bg-gray-900 dark:text-white bg-white text-black flex flex-col h-screen font-sans p-4",
            div {
                class: "flex flex-1 overflow-hidden",
                {sidebar}
                div {
                    div {
                      {render_model_selector(model.clone(), all_models())}
                        {render_message_list(&messages(), &katex_opts)}
                        {render_message_input(input.clone(), on_send, attachment.clone(), messages().is_empty())}
                    }
                    div { class: "flex items-center gap-4 mt-2",
                        label {
                            class: "flex items-center gap-2 cursor-pointer text-sm text-gray-400",
                            input {
                                r#type: "checkbox",
                                checked: "{use_web_search}",
                                oninput: move |e| {
                                    let value = e.value().parse().unwrap_or(false);
                                    use_web_search.set(value);
                                },
                                class: "rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500"
                            }
                            "Web Search"
                        }
                        if model().map(|m| m.capabilities.image_generation).unwrap_or(false) {
                            label {
                                class: "flex items-center gap-2 cursor-pointer text-sm text-gray-400",
                                input {
                                    r#type: "checkbox",
                                    checked: model().map(|m| m.capabilities.image_generation).unwrap_or(false),
                                    onchange: move |e| {
                                        if let Some(mut m) = model() {
                                            m.capabilities.image_generation = e.value().parse().unwrap_or(false);
                                            model.set(Some(m));
                                        }
                                    },
                                    class: "rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500"
                                }
                                "Image Generation"
                            }
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
    rsx! {
        ChatBase { id: None }
    }
}

#[component]
pub fn ChatShare(id: usize) -> Element {
    rsx!(ChatBase { id: Some(id) })
}
