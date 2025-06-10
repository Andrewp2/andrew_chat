//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use base64::Engine;

/// Represents a message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMessage {
    Text(String),
    /// Base64 encoded data uri of an image
    Image(String),
}

/// Represents the messages for each conversation.
type Conversations = Vec<Vec<ChatMessage>>;

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
pub async fn send_message(conv_id: usize, msg: ChatMessage) -> Result<(), ServerFnError> {
    let mut history = CHAT_HISTORY.write().await;
    if let Some(conv) = history.get_mut(conv_id) {
        conv.push(msg);
    }
    Ok(())
}

/// Retrieve all chat messages for a specific conversation.
#[server(GetMessages)]
pub async fn get_messages(conv_id: usize) -> Result<Vec<ChatMessage>, ServerFnError> {
    let history = CHAT_HISTORY.read().await;
    Ok(history.get(conv_id).cloned().unwrap_or_default())
}

/// Generate an image from a prompt and return it as a data URI.
#[server(GenerateImage)]
pub async fn generate_image(prompt: String) -> Result<String, ServerFnError> {
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='256' height='256'>\
<rect width='100%' height='100%' fill='blue'/>\
<text x='50%' y='50%' dominant-baseline='middle' text-anchor='middle' font-size='20' fill='white'>{}</text></svg>",
        prompt
    );
    let encoded = base64::engine::general_purpose::STANDARD.encode(svg);
    Ok(format!("data:image/svg+xml;base64,{}", encoded))
}
