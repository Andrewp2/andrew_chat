//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Represents the messages for each conversation.
type Conversations = Vec<Vec<String>>;

/// In memory list of chat messages.
/// In memory list of conversations. Each conversation is a vector of chat messages.
static CHAT_HISTORY: Lazy<Arc<RwLock<Conversations>>> = Lazy::new(|| Arc::new(RwLock::new(vec![Vec::new()])));

/// In-memory store of users where the key is the username and the value is the password.
static USERS: Lazy<Arc<RwLock<HashMap<String, String>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

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

/// Register a new user with a username and password.
#[server(Register)]
pub async fn register(username: String, password: String) -> Result<(), ServerFnError> {
    let mut users = USERS.write().await;
    if users.contains_key(&username) {
        return Err(ServerFnError::new("User already exists"));
    }
    users.insert(username, password);
    Ok(())
}

/// Verify user credentials and return true if they are valid.
#[server(Login)]
pub async fn login(username: String, password: String) -> Result<bool, ServerFnError> {
    let users = USERS.read().await;
    Ok(users.get(&username).map(|p| p == &password).unwrap_or(false))
}
