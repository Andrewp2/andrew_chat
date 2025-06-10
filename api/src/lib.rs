//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

/// Query an AI model using the provided API key.
///
/// `provider` should be either `openai` or `anthropic`. The API key will be
/// sent directly to the upstream provider for the request.
#[server(ChatCompletion)]
pub async fn chat_completion(
    provider: String,
    api_key: String,
    prompt: String,
    model: String,
) -> Result<String, ServerFnError> {
    match provider.as_str() {
        "openai" => {
            let client = reqwest::Client::new();
            let body = serde_json::json!({
                "model": model,
                "messages": [OpenAiMessage { role: "user", content: &prompt }],
            });
            let res = client
                .post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(api_key)
                .json(&body)
                .send()
                .await
                .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
            let json: serde_json::Value = res
                .json()
                .await
                .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
            if let Some(reply) = json["choices"][0]["message"]["content"].as_str() {
                Ok(reply.to_string())
            } else {
                Err(ServerFnError::ServerError("invalid response".into()))
            }
        }
        "anthropic" => {
            let client = reqwest::Client::new();
            let body = serde_json::json!({
                "model": model,
                "max_tokens": 1024,
                "messages": [{"role": "user", "content": prompt}],
            });
            let res = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send()
                .await
                .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
            let json: serde_json::Value = res
                .json()
                .await
                .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
            if let Some(reply) = json["content"][0]["text"].as_str() {
                Ok(reply.to_string())
            } else {
                Err(ServerFnError::ServerError("invalid response".into()))
            }
        }
        _ => Err(ServerFnError::ServerError("unknown provider".into())),
    }
}
