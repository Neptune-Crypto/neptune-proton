// ui/src/components/currency_amount_input.rs
use dioxus::prelude::*;

use crate::components::pico::Button;
use crate::components::pico::ButtonType;
use crate::hooks::use_is_touch_device::use_is_touch_device;

// The NumericKeypad component is unchanged.
#[component]
pub fn NumericKeypad(on_key_press: EventHandler<String>, on_close: EventHandler<()>) -> Element {
    let keys = [
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        ".",
        "0",
        "BACKSPACE",
    ];
    let mut active_key_local = use_signal::<Option<String>>(|| None);

    let handle_key_down = move |event: Event<KeyboardData>| {
        let event_key_string = event.data.key().to_string();
        let event_key_str = event_key_string.as_str();

        if event_key_str == "Escape" || event_key_str == "Enter" {
            on_close.call(());
            event.stop_propagation();
            return;
        }

        let mapped_key = match event_key_str {
            "Backspace" => Some("BACKSPACE"),
            "." | "Decimal" => Some("."),
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => Some(event_key_str),
            _ => None,
        };

        if let Some(mapped_key) = mapped_key {
            active_key_local.set(Some(mapped_key.to_string()));
            on_key_press.call(mapped_key.to_string());
            event.stop_propagation();
        } else if event_key_str.len() == 1 {
            event.stop_propagation();
        }
    };

    let handle_animation_end = move |_: Event<AnimationData>| {
        active_key_local.set(None);
    };

    rsx! {
        style {
            {
                r#" .key-flash { animation: keyFlash 0.25s ease-out; } @keyframes keyFlash { 0% { background-color: var(--pico-secondary-border); transform: scale(1.0); } 50% { background-color: var(--pico-background-inverse); color: var(--pico-color-inverse); transform: scale(0.96); } 100% { background-color: var(--pico-background-color); color: var(--pico-color-text); transform: scale(1.0); } } "#
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
                            class: if is_active { "pico-button secondary outline key-flash" } else { "pico-button secondary outline" },
                            style: "font-size: 1.1rem; padding: 0.75rem; display: flex; justify-content: center; align-items: center;",
                            onanimationend: handle_animation_end,
                            onclick: move |_| {
                                active_key_local.set(Some(key_str.clone()));
                                on_key_press.call(key_str.clone());
                            },
                            if key == "BACKSPACE" {
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    width: "24",
                                    height: "24",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    path {
                                        d: "M21 4H8l-7 8 7 8h13a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2z",
                                    }
                                    line {
                                        x1: "18",
                                        y1: "9",
                                        x2: "12",
                                        y2: "15",
                                    }
                                    line {
                                        x1: "12",
                                        y1: "9",
                                        x2: "18",
                                        y2: "15",
                                    }
                                }
                            } else {
                                "{key}"
                            }
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
    value: String,
    on_input: EventHandler<String>,
    mut popup_state: Signal<Option<Element>>,
    max_integers: u8,
    max_decimals: u8,
    placeholder: String,
) -> Element {
    let is_touch_device = use_is_touch_device();
    let is_popup_visible = use_memo(move || popup_state.read().is_some());

    let is_numerically_zero = value.trim().parse::<f64>() == Ok(0.0);

    let mut value_signal = use_signal(|| value.clone());

    // Sync signal with prop
    use_effect({
        let value = value.clone();
        move || {
            if *value_signal.read() != value {
                value_signal.set(value.clone());
            }
        }
    });

    let mut handle_new_input = move |new_value: String| {
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
                } else if integer_digits < max_integers {
                    sanitized.push(ch);
                    integer_digits += 1;
                }
            } else if ch == '.' && !has_decimal {
                sanitized.push(ch);
                has_decimal = true;
            }
        }
        on_input.call(sanitized.clone());

        // Instantly update the mirror, breaking the race condition.
        value_signal.set(sanitized);
    };
    let mut handle_new_input_clone = handle_new_input;

    let handle_input_keydown = move |event: Event<KeyboardData>| {
        if is_popup_visible() {
            event.stop_propagation();
        }
    };

    let mut handle_interaction = move || {
        if is_numerically_zero {
            on_input.call("".to_string());
            value_signal.set("".to_string());
        }
    };
    let mut handle_interaction_clone = handle_interaction;
    let mut handle_interaction_click = handle_interaction.clone();

    let open_keypad = {
        let value = value.clone();
        move |_| {
            // Fix for issue 17
            // https://github.com/Neptune-Crypto/neptune-proton/issues/17
            // Ensure internal signal matches the current prop before starting the interaction.
            // This fixes an issue where toggling currency leaves the keypad logic
            // holding the old value.
            if *value_signal.read() != value {
                value_signal.set(value.clone());
            }

            handle_interaction();

            spawn(async move {
                let handle_keypad_press = move |key: String| {
                    let current_val = value_signal.read().clone();
                    let new_val = if key == "BACKSPACE" {
                        let mut chars = current_val.chars();
                        chars.next_back();
                        chars.as_str().to_string()
                    } else {
                        current_val + &key
                    };
                    handle_new_input_clone(new_val);
                };

                let keypad_popup = rsx! {
                    div {
                        style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.1); z-index: 1000; display: flex; justify-content: center; align-items: center;",
                        onclick: move |_| {
                            spawn(async move {
                                popup_state.set(None);
                            });
                        },
                        NumericKeypad {
                            on_key_press: handle_keypad_press,
                            on_close: move |_| {
                                spawn(async move {
                                    popup_state.set(None);
                                });
                            },
                        }
                    }
                };
                popup_state.set(Some(keypad_popup));
            });
        }
    };

    let mut open_keypad_clone = open_keypad.clone();

    let show_placeholder = value.is_empty();
    let display_value = if show_placeholder { "" } else { &value };

    let focus_css = r#"
        input.hide-placeholder-focus:focus::placeholder {
            color: transparent;
            opacity: 0;
        }
    "#;

    rsx! {
        style { "{focus_css}" }
        div {
            style: "display: flex; flex-grow: 1; gap: 0.5rem;",
            div {
                style: "flex-grow: 1; display: flex;",
                input {
                    r#type: "text",
                    // Added custom class 'hide-placeholder-focus'
                    class: "pico-input hide-placeholder-focus",
                    style: "margin-bottom: 0; width: 100%;",
                    inputmode: "decimal",

                    placeholder: "{placeholder}",
                    value: "{display_value}",

                    onkeydown: handle_input_keydown,
                    onfocus: move |_| handle_interaction_clone(),
                    oninput: move |event| { handle_new_input(event.value()) },
                    onclick: move |e| {
                        e.stop_propagation();
                        if is_touch_device() {
                            open_keypad_clone(e);
                        } else {
                            handle_interaction_click();
                        }
                    },
                }
            }
            if !is_touch_device() {
                Button {
                    title: "Display Numeric Keypad",
                    button_type: ButtonType::Secondary,
                    outline: true,
                    style: "width: 3rem; margin-bottom: 0; flex-shrink: 0;",

                    on_click: open_keypad,
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
                        rect {
                            x: "4",
                            y: "2",
                            width: "16",
                            height: "20",
                            rx: "2",
                            ry: "2",
                        }
                        line {
                            x1: "8",
                            y1: "6",
                            x2: "16",
                            y2: "6",
                        }
                        line {
                            x1: "12",
                            y1: "10",
                            x2: "12",
                            y2: "18",
                        }
                        line {
                            x1: "8",
                            y1: "14",
                            x2: "16",
                            y2: "14",
                        }
                    }
                }
            }
        }
    }
}
