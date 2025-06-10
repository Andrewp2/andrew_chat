//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::BroadcastStream;
use futures::{stream, StreamExt};
use server_fn::codec::{StreamingText, TextStream};

/// Represents the messages for each conversation.
struct Conversation {
    messages: Vec<String>,
    tx: broadcast::Sender<String>,
}

impl Conversation {
    fn new() -> Self {
        let (tx, _rx) = broadcast::channel(32);
        Self { messages: Vec::new(), tx }
    }
}

type Conversations = Vec<Conversation>;

/// In memory list of conversations used for demo purposes.
/// Each conversation stores messages and broadcasts new ones for streaming.
static CHAT_HISTORY: Lazy<Arc<RwLock<Conversations>>> = Lazy::new(|| {
    Arc::new(RwLock::new(vec![Conversation::new()]))
});

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
    history.push(Conversation::new());
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
        conv.messages.push(msg.clone());
        let _ = conv.tx.send(msg);
    }
    Ok(())
}

/// Retrieve all chat messages for a specific conversation.
#[server(GetMessages)]
pub async fn get_messages(conv_id: usize) -> Result<Vec<String>, ServerFnError> {
    let history = CHAT_HISTORY.read().await;
    Ok(history
        .get(conv_id)
        .map(|c| c.messages.clone())
        .unwrap_or_default())
}

/// Stream messages for a conversation starting at the given index.
#[server(StreamMessages, output = StreamingText)]
pub async fn stream_messages(
    conv_id: usize,
    from: usize,
) -> Result<TextStream, ServerFnError> {
    let (messages, sender) = {
        let history = CHAT_HISTORY.read().await;
        if let Some(conv) = history.get(conv_id) {
            (conv.messages.clone(), conv.tx.clone())
        } else {
            return Ok(TextStream::new(stream::empty()));
        }
    };

    let past = messages.into_iter().skip(from).map(Ok);
    let rx = sender.subscribe();
    let incoming = BroadcastStream::new(rx).filter_map(|msg| async move { msg.ok().map(Ok) });
    let stream = stream::iter(past).chain(incoming);
    Ok(TextStream::new(stream))
}
