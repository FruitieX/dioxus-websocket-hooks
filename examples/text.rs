use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_websocket_hooks::{use_ws_context, use_ws_context_provider_text};
use fermi::{use_init_atom_root, use_read, use_set, Atom};

fn main() {
    dioxus::web::launch(app);
}

pub static WS_RESPONSE_ATOM: Atom<String> = |_| Default::default();

fn app(cx: Scope) -> Element {
    use_init_atom_root(&cx);
    let set_response = Rc::clone(use_set(&cx, WS_RESPONSE_ATOM));

    use_ws_context_provider_text(&cx, "wss://echo.websocket.events", move |msg| {
        set_response(msg)
    });

    cx.render(rsx!(ResponseDisplay {}))
}

fn ResponseDisplay(cx: Scope) -> Element {
    let response = use_read(&cx, WS_RESPONSE_ATOM);
    let ws = use_ws_context(&cx);

    let input = use_state(&cx, String::default);
    let submit = move |_| {
        ws.send_text(input.to_string());
        input.set(String::default());
    };

    cx.render(rsx! (
        div { "Server sent: {response}" }
        input { oninput: move |event| input.set(event.value.clone()), "{input}" }
        button { onclick: submit, "Submit" }
    ))
}
