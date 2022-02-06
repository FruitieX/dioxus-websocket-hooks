use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_websocket_hooks::{use_ws_context, use_ws_context_provider_json};
use fermi::{use_init_atom_root, use_read, use_set, Atom};
use serde::{Deserialize, Serialize};

fn main() {
    dioxus::web::launch(app);
}

// Response and request are identical since we're connecting to an echo server.
#[derive(Deserialize, Serialize, Debug)]
pub enum WsResponse {
    A,
    B,
    C,
}

#[derive(Deserialize, Serialize)]
pub enum WsRequest {
    A,
    B,
    C,
}

pub static WS_RESPONSE_ATOM: Atom<Option<WsResponse>> = |_| None;

fn app(cx: Scope) -> Element {
    use_init_atom_root(&cx);
    let set_response = Rc::clone(use_set(&cx, WS_RESPONSE_ATOM));

    use_ws_context_provider_json(&cx, "wss://echo.websocket.events", move |msg| {
        // Handler for incoming parsed JSON websocket messages.

        // In this example we just set the response atom to the received message.
        // You could also match over the enum and perform different actions here
        // as needed.
        set_response(msg)
    });

    cx.render(rsx!(ResponseDisplay {}))
}

#[allow(non_snake_case)]
fn ResponseDisplay(cx: Scope) -> Element {
    // We can access the latest websocket response from other components:
    let response = use_read(&cx, WS_RESPONSE_ATOM);

    let response = response
        .as_ref()
        .map(|r| format!("{:?}", r))
        .unwrap_or_else(|| String::from("(nothing)"));

    cx.render(rsx! (
        div { "Server sent: {response}" }
        SendA {}
        SendB {}
        SendC {}
    ))
}

// We can also send websocket messages from other components:
#[allow(non_snake_case)]
fn SendA(cx: Scope) -> Element {
    let ws = use_ws_context(&cx);
    let onclick = move |_| ws.send_json(&WsRequest::A);
    cx.render(rsx!(button { onclick: onclick, "A" }))
}

#[allow(non_snake_case)]
fn SendB(cx: Scope) -> Element {
    let ws = use_ws_context(&cx);
    let onclick = move |_| ws.send_json(&WsRequest::B);
    cx.render(rsx!(button { onclick: onclick, "B" }))
}

#[allow(non_snake_case)]
fn SendC(cx: Scope) -> Element {
    let ws = use_ws_context(&cx);
    let onclick = move |_| ws.send_json(&WsRequest::C);
    cx.render(rsx!(button { onclick: onclick, "C" }))
}
