//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents the messages for each conversation.
type Conversations = Vec<Vec<String>>;

/// In memory list of chat messages.
/// In memory list of conversations. Each conversation is a vector of chat messages.
static CHAT_HISTORY: Lazy<Arc<RwLock<Conversations>>> = Lazy::new(|| Arc::new(RwLock::new(vec![Vec::new()])));

/// Echo the user input on the server.
#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

/// Create a new conversation and return its id.
#[server(CreateConversation)]
pub async fn create_conversation() -> Result<usize, ServerFnError> {
    let mut history = CHAT_HISTORY.write().await;
    let id = history.len();
    history.push(Vec::new());
    Ok(id)
}

/// List all conversation ids.
#[server(ListConversations)]
pub async fn list_conversations() -> Result<Vec<usize>, ServerFnError> {
    let history = CHAT_HISTORY.read().await;
    Ok((0..history.len()).collect())
}

/// Store a chat message in memory for a specific conversation.
#[server(SendMessage)]
pub async fn send_message(conv_id: usize, msg: String) -> Result<(), ServerFnError> {
    let mut history = CHAT_HISTORY.write().await;
    if let Some(conv) = history.get_mut(conv_id) {
        conv.push(msg);
    }
    Ok(())
}

/// Retrieve all chat messages for a specific conversation.
#[server(GetMessages)]
pub async fn get_messages(conv_id: usize) -> Result<Vec<String>, ServerFnError> {
    let history = CHAT_HISTORY.read().await;
    Ok(history.get(conv_id).cloned().unwrap_or_default())
}

/// Perform a web search using DuckDuckGo and return a summary of the top results.
#[server(WebSearch)]
pub async fn web_search(query: String) -> Result<String, ServerFnError> {
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_redirect=1&no_html=1",
        urlencoding::encode(&query)
    );
    let resp = reqwest::get(url)
        .await
        .map_err(|e| server_fn::error::ServerFnError::<server_fn::error::NoCustomError>::ServerError(e.to_string()))?;
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| server_fn::error::ServerFnError::<server_fn::error::NoCustomError>::ServerError(e.to_string()))?;

    let mut results = Vec::new();
    if let Some(topics) = json.get("RelatedTopics").and_then(|v| v.as_array()) {
        for topic in topics.iter().take(3) {
            if let Some(text) = topic.get("Text").and_then(|t| t.as_str()) {
                results.push(text.to_string());
            }
        }
    }

    Ok(results.join("\n"))
}
