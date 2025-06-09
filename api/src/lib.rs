//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In memory list of chat messages.
static CHAT_HISTORY: Lazy<Arc<RwLock<Vec<String>>>> = Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

/// Echo the user input on the server.
#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

/// Store a chat message in memory.
#[server(SendMessage)]
pub async fn send_message(msg: String) -> Result<(), ServerFnError> {
    CHAT_HISTORY.write().await.push(msg);
    Ok(())
}

/// Retrieve all chat messages.
#[server(GetMessages)]
pub async fn get_messages() -> Result<Vec<String>, ServerFnError> {
    Ok(CHAT_HISTORY.read().await.clone())
}
