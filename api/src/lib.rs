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
