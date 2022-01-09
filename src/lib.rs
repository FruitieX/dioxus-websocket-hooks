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
    sender: Arc<RwLock<SplitSink<WebSocket, Message>>>,
    receiver: Arc<RwLock<SplitStream<WebSocket>>>,
}

impl DioxusWs {
    pub fn new(url: &str) -> DioxusWs {
        let ws = WebSocket::open(url).unwrap();

        let (sender, receiver) = ws.split();
        let sender = Arc::new(RwLock::new(sender));
        let receiver = Arc::new(RwLock::new(receiver));

        DioxusWs { sender, receiver }
    }

    pub fn send(&self, msg: Message) {
        let sender = self.sender.clone();

        spawn_local(async move {
            let mut sender = sender.write().await;
            sender.send(msg).await.unwrap()
        });
    }

    pub fn send_text(&self, text: String) {
        let msg = Message::Text(text);
        self.send(msg);
    }

    pub fn send_json<T: Serialize>(&self, value: &T) {
        let json = serde_json::to_string(value).unwrap();
        let msg = Message::Text(json);
        self.send(msg)
    }
}

pub fn use_ws_context_provider(cx: &ScopeState, url: &str, reducer: impl Fn(Message) + 'static) {
    let reducer = Rc::new(reducer);

    cx.use_hook(|_| {
        let ws = cx.provide_context(DioxusWs::new(url));
        let receiver = ws.receiver.clone();

        cx.push_future(async move {
            let mut receiver = receiver.write().await;
            while let Some(msg) = receiver.next().await {
                if let Ok(msg) = msg {
                    reducer(msg);
                }
            }
        })
    });
}

pub fn use_ws_context_provider_json<T>(cx: &ScopeState, url: &str, reducer: impl Fn(T) + 'static)
where
    T: for<'de> Deserialize<'de>,
{
    let reducer = move |msg| match msg {
        Message::Text(text) => {
            let json = serde_json::from_str::<T>(&text);

            match json {
                Ok(json) => reducer(json),

                // TODO: this will likely be suppressed as usage is expected to be in a web browser
                Err(e) => eprintln!("Error while deserializing websocket response: {}", e),
            }
        }
        Message::Bytes(_) => {}
    };

    use_ws_context_provider(cx, url, reducer)
}

pub fn use_ws_context(cx: &ScopeState) -> Rc<DioxusWs> {
    cx.consume_context::<DioxusWs>().unwrap()
}
