# dioxus-websocket-hooks

Dioxus hooks for websocket connections

```rust
use dioxus_websocket_hooks::{use_ws_context, use_ws_context_provider_json};

fn app(cx: Scope) -> Element {
    use_ws_context_provider_json(&cx, "wss://echo.websocket.events", move |msg| {
        // Handle incoming ws message, e.g. store it in shared state
    });

    ...
}

fn ExampleComponent(cx: Scope) -> Element {
    let ws = use_ws_context(&cx);

    cx.render(rsx! (
        button { onclick: move |_| ws.send_json(&"hello"), "Submit" }
    ))
}
```

## Examples

See [cargo examples](/examples)

Samples make use of [fermi](https://github.com/DioxusLabs/fermi) for state management.