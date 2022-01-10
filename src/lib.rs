use std::{rc::Rc, sync::Arc};

use async_rwlock::RwLock;
use dioxus::prelude::*;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use reqwasm::websocket::{futures::WebSocket, Message};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

pub struct DioxusWs {
    url: String,
    sender: Arc<RwLock<SplitSink<WebSocket, Message>>>,
    receiver: Arc<RwLock<SplitStream<WebSocket>>>,
}

impl DioxusWs {
    pub fn new(url: &str) -> DioxusWs {
        let ws = WebSocket::open(url).unwrap();

        let (sender, receiver) = ws.split();
        let sender = Arc::new(RwLock::new(sender));
        let receiver = Arc::new(RwLock::new(receiver));

        DioxusWs { url: url.to_string(), sender, receiver }
    }

    /// Sends a reqwasm Message
    pub fn send(&self, msg: Message) {
        let sender = self.sender.clone();

        spawn_local(async move {
            let mut sender = sender.write().await;
            sender.send(msg).await.unwrap()
        });
    }

    /// Sends a plaintext string
    pub fn send_text(&self, text: String) {
        let msg = Message::Text(text);
        self.send(msg);
    }

    /// Sends data that implements Serialize as JSON
    pub fn send_json<T: Serialize>(&self, value: &T) {
        let json = serde_json::to_string(value).unwrap();
        let msg = Message::Text(json);
        self.send(msg)
    }
}

/// Provide websocket context with a handler for incoming reqwasm Messages
pub fn use_ws_context_provider(cx: &ScopeState, url: &str, handler: impl Fn(Message) + 'static) {
    let handler = Rc::new(handler);

    cx.use_hook(|_| {
        let ws = cx.provide_context(DioxusWs::new(url));
        let receiver = ws.receiver.clone();

        cx.push_future(async move {
            let mut receiver = receiver.write().await;
            while let Some(msg) = receiver.next().await {
                // TODO: Reconnect on error
                if let Ok(msg) = msg {
                    handler(msg);
                }
            }
        })
    });
}

/// Provide websocket context with a handler for incoming plaintext messages
pub fn use_ws_context_provider_text(
    cx: &ScopeState,
    url: &str,
    handler: impl Fn(String) + 'static,
) {
    let handler = move |msg| {
        if let Message::Text(text) = msg {
            handler(text)
        }
    };

    use_ws_context_provider(cx, url, handler)
}

/// Provide websocket context with a handler for incoming JSON messages.
/// Note that the message type T must implement Deserialize.
pub fn use_ws_context_provider_json<T>(cx: &ScopeState, url: &str, handler: impl Fn(T) + 'static)
where
    T: for<'de> Deserialize<'de>,
{
    let handler = move |msg| match msg {
        Message::Text(text) => {
            let json = serde_json::from_str::<T>(&text);

            match json {
                Ok(json) => handler(json),

                // TODO: this will likely be suppressed as usage is expected to be in a web browser
                Err(e) => eprintln!("Error while deserializing websocket response: {}", e),
            }
        }
        Message::Bytes(_) => {}
    };

    use_ws_context_provider(cx, url, handler)
}

pub fn use_ws_context(cx: &ScopeState) -> Rc<DioxusWs> {
    cx.consume_context::<DioxusWs>().unwrap()
}
