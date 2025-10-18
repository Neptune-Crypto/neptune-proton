//=============================================================================
// File: src/components/currency_amount_input.rs
//=============================================================================
use crate::hooks::use_is_touch_device::use_is_touch_device;
use dioxus::prelude::*;

/// A simple, reusable numeric keypad component, now with a "Done" button.
#[component]
pub fn NumericKeypad(
    on_key_press: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let keys = ["1", "2", "3", "4", "5", "6", "7", "8", "9", ".", "0", "⌫"];

    // Local state for flashing
    let mut active_key_local = use_signal::<Option<String>>(|| None);

    // Input handling logic
    let handle_key_down = move |event: Event<KeyboardData>| {
        let event_key_string = event.data.key().to_string();
        let event_key_str = event_key_string.as_str();

        let mapped_key = match event_key_str {
            "Backspace" => Some("⌫"),
            "." | "Decimal" => Some("."),
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => Some(event_key_str),
            _ => None,
        };

        if let Some(mapped_key) = mapped_key {
            active_key_local.set(Some(mapped_key.to_string()));
            on_key_press.call(mapped_key.to_string());
            event.stop_propagation();
        } else if event_key_str.len() == 1 || event_key_str == "Enter" {
            event.stop_propagation();
        }
    };

    let handle_animation_end = move |_: Event<AnimationData>| {
        active_key_local.set(None);
    };

    rsx! {
        style {
            {
                r#"
                .key-flash {
                    animation: keyFlash 0.25s ease-out;
                }
                @keyframes keyFlash {
                    0% { background-color: var(--pico-secondary-border); transform: scale(1.0); }
                    50% {
                        background-color: var(--pico-background-inverse);
                        color: var(--pico-color-inverse);
                        transform: scale(0.96);
                    }
                    100% { background-color: var(--pico-background-color); color: var(--pico-color-text); transform: scale(1.0); }
                }
                "#
            }
        }
        div {
            onmounted: move |mounted| {
                let mounted_data = mounted.data.clone();
                spawn(async move {
                    mounted_data.set_focus(true).await.ok();
                });
            },
            tabindex: "0",
            onkeydown: handle_key_down,
            onclick: move |e| e.stop_propagation(),
            class: "numeric-keypad",
            style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.5rem; background: var(--pico-background-color); padding: 0.75rem; border-radius: var(--pico-border-radius); border: 1.5px solid var(--pico-muted-border-color); box-shadow: 0 4px 12px rgba(0,0,0,0.15); width: 200px; position: relative; z-index: 1001;",

            for key in keys {
                {
                    let key_str = key.to_string();
                    let is_active = active_key_local.read().as_deref() == Some(key);

                    rsx! {
                        button {
                            key: "{key}",
                            class: if is_active { "pico-button key-flash" } else { "pico-button" },
                            style: "font-size: 1.1rem; padding: 0.75rem;",
                            onanimationend: handle_animation_end,
                            onclick: move |_| {
                                active_key_local.set(Some(key_str.clone()));
                                on_key_press.call(key_str.clone());
                            },
                            "{key}"
                        }
                    }
                }
            }
            div {
                style: "grid-column: 1 / -1; margin-top: 0.5rem;",
                button {
                    class: "pico-button pico-button--primary",
                    style: "width: 100%;",
                    onclick: move |_| on_close.call(()),
                    "Done"
                }
            }
        }
    }
}

// --------------------------------------------------------------------------------------------------

#[component]
pub fn CurrencyAmountInput(
    value: Signal<String>,
    on_input: EventHandler<String>,
    on_keypad_press: EventHandler<String>,
    mut popup_state: Signal<Option<Element>>,
    max_integers: u8,
    max_decimals: u8,
    placeholder: String,
) -> Element {
    let is_touch_device = use_is_touch_device();
    let is_popup_visible = use_memo(move || popup_state.read().is_some());

    let handle_new_input = move |new_value: String| {
        let mut sanitized = String::new();
        let mut has_decimal = false;
        let mut integer_digits = 0;
        let mut decimal_digits = 0;

        for ch in new_value.chars() {
            if ch.is_ascii_digit() {
                if has_decimal {
                    if decimal_digits < max_decimals {
                        sanitized.push(ch);
                        decimal_digits += 1;
                    }
                } else {
                    if integer_digits < max_integers {
                        sanitized.push(ch);
                        integer_digits += 1;
                    }
                }
            } else if ch == '.' && !has_decimal {
                sanitized.push(ch);
                has_decimal = true;
            }
        }
        on_input.call(sanitized);
    };

    let handle_input_keydown = move |event: Event<KeyboardData>| {
        if is_popup_visible() {
            event.stop_propagation();
        }
    };

    let open_keypad = move |_| {
        // PANIC FIX: The write to popup_state MUST be deferred to after the render.
        // spawn achieves this by scheduling the work for the next tick of the event loop.
        spawn(async move {
            let keypad_popup = rsx! {
                div {
                    style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.1); z-index: 1000; display: flex; justify-content: center; align-items: center;",
                    onclick: move |_| {
                        // Also defer the closing action.
                        spawn(async move { popup_state.set(None); });
                    },
                    NumericKeypad {
                        on_key_press: move |key: String| {
                            on_keypad_press.call(key);
                        },
                        on_close: move |_| {
                            // Also defer the closing action.
                            spawn(async move { popup_state.set(None); });
                        }
                    }
                }
            };
            popup_state.set(Some(keypad_popup));
        });
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.5rem; width: 100%;",
            div {
                style: "flex-grow: 1; margin-bottom: 0;",
                input {
                    r#type: "text",
                    class: "pico-input",
                    inputmode: "decimal",
                    placeholder: "{placeholder}",
                    value: "{value}",
                    onkeydown: handle_input_keydown,
                    oninput: move |event| {
                        if !is_popup_visible() {
                            handle_new_input(event.value());
                        }
                    },
                    onclick: move |e| {
                        e.stop_propagation();
                        if is_touch_device() {
                            open_keypad(e);
                        }
                    }
                }
            }
            if !is_touch_device() {
                button {
                    class: "pico-button pico-button--secondary",
                    style: "width: 3rem; padding: 0.5rem; margin-bottom: 0; flex-shrink: 0;",
                    onclick: open_keypad,
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        width: "20",
                        height: "20",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        rect { x: "4", y: "2", width: "16", height: "20", rx: "2", ry: "2" }
                        line { x1: "8", y1: "6", x2: "16", y2: "6" }
                        line { x1: "12", y1: "10", x2: "12", y2: "18" }
                        line { x1: "8", y1: "14", x2: "16", y2: "14" }
                    }
                }
            }
        }
    }
}