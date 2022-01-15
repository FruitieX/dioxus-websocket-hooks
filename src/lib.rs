use std::{rc::Rc, sync::Arc, time::Duration};

use async_std::sync::RwLock;
use dioxus::prelude::*;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use reqwasm::websocket::{futures::WebSocket, Message};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

pub struct DioxusWs {
    url: String,
    sender: Arc<RwLock<SplitSink<WebSocket, Message>>>,
    receiver: Arc<RwLock<SplitStream<WebSocket>>>,
    is_open: Arc<RwLock<bool>>,
}

impl DioxusWs {
    pub fn new(url: &str) -> DioxusWs {
        let ws = WebSocket::open(url).unwrap();

        let (sender, receiver) = ws.split();
        let sender = Arc::new(RwLock::new(sender));
        let receiver = Arc::new(RwLock::new(receiver));

        DioxusWs {
            url: url.to_string(),
            sender,
            receiver,
            is_open: Arc::new(RwLock::new(false)),
        }
    }

    /// Sends a reqwasm Message
    pub fn send(&self, msg: Message) {
        let sender = self.sender.clone();
        let is_open = self.is_open.clone();

        spawn_local(async move {
            let is_open = *is_open.read().await;

            if is_open {
                let mut sender = sender.write().await;
                sender.send(msg).await.ok();
            }
        });
    }

    pub fn set_open(&self, open: bool) {
        let is_open = self.is_open.clone();
        let sender = self.sender.clone();

        spawn_local(async move {
            let mut is_open = is_open.write().await;
            *is_open = open;

            let mut sender = sender.write().await;
            sender.close().await.ok();
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
        self.send(msg);
    }

    pub async fn reconnect(&self) {
        let ws = WebSocket::open(&self.url).unwrap();

        let (sender, receiver) = ws.split();

        {
            let mut self_sender = self.sender.write().await;
            *self_sender = sender;
        }

        {
            let mut self_receiver = self.receiver.write().await;
            *self_receiver = receiver;
        }
    }
}

fn log_err(s: &str) {
    web_sys::console::error_1(&JsValue::from_str(s));
}

/// Provide websocket context with a handler for incoming reqwasm Messages
pub fn use_ws_context_provider(cx: &ScopeState, url: &str, handler: impl Fn(Message) + 'static) {
    let handler = Rc::new(handler);

    cx.use_hook(|_| {
        let ws = cx.provide_context(DioxusWs::new(url));
        let receiver = ws.receiver.clone();

        cx.push_future(async move {
            loop {
                let mut err = None;

                {
                    let mut receiver = receiver.write().await;
                    while let Some(msg) = receiver.next().await {
                        match msg {
                            Ok(msg) => {
                                ws.set_open(true);
                                handler(msg)
                            },
                            Err(e) => {
                                err = Some(e);
                            }
                        }
                    }
                }

                if let Some(err) = err {
                    ws.set_open(false);

                    log_err(&format!(
                        "Error while trying to receive message over websocket, reconnecting in 1s...\n{:?}", err
                    ));

                    async_std::task::sleep(Duration::from_millis(1000)).await;

                    ws.reconnect().await;
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
                Err(e) => log_err(&format!(
                    "Error while deserializing websocket response: {}",
                    e
                )),
            }
        }
        Message::Bytes(_) => {}
    };

    use_ws_context_provider(cx, url, handler)
}

/// Consumes WebSocket context. Useful for sending messages over the WebSocket
/// connection.
///
/// NOTE: Currently the server is expected to send a message when the connection
/// opens. You will not be able to send websocket messages from the client
/// before a message has been received from the server. This is a limitation
/// in the current reconnection logic.
pub fn use_ws_context(cx: &ScopeState) -> Rc<DioxusWs> {
    cx.consume_context::<DioxusWs>().unwrap()
}
