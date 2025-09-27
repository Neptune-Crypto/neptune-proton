//! A set of reusable, lifetime-free Dioxus components for the Pico.css framework.
//! To use, ensure you have pico.min.css linked in your main application.

#![allow(non_snake_case)] // Allow PascalCase for component function names
#![allow(dead_code)] // Allow PascalCase for component function names

use dioxus::html::input_data::keyboard_types::Key;
use dioxus::prelude::*;
use std::time::Duration;

//=============================================================================
// Layout Components
//=============================================================================

/// A centered container for your content.
/// Wraps content in a `<main class="container">` element.
#[component]
pub fn Container(children: Element) -> Element {
    rsx! { main { class: "container", {children} } }
}

/// A responsive grid layout.
/// Wraps `GridItem` components in a `<div class="grid">`.
#[component]
pub fn Grid(children: Element) -> Element {
    rsx! { div { class: "grid", {children} } }
}

//=============================================================================
// Content Components
//=============================================================================

/// A card for grouping related content.
/// Wraps content in an `<article>` element.
#[component]
pub fn Card(children: Element) -> Element {
    rsx! { article { {children} } }
}

#[derive(Props, PartialEq, Clone)]
pub struct AccordionProps {
    title: String,
    children: Element,
}

/// An accordion for showing/hiding content, using the <details> element.
pub fn Accordion(props: AccordionProps) -> Element {
    rsx! {
        details {
            summary { role: "button", "{props.title}" }
            {props.children}
        }
    }
}

//=============================================================================
// Interactive Components
//=============================================================================

#[derive(Props, PartialEq, Clone)]
pub struct CloseButtonProps {
    on_click: Option<EventHandler<MouseEvent>>,
    #[props(default = "".to_string())]
    title: String,
}

pub fn CloseButton(props: CloseButtonProps) -> Element {
    rsx! {
        button {
            title: "{props.title}",
            class: "close",
            class: "primary",
            onclick: move |evt| {
                if let Some(handler) = &props.on_click {
                    handler.call(evt);
                }
            },
            "\u{2716}" // "✖"
        }
    }
}

#[derive(PartialEq, Clone, Default)]
pub enum ButtonType {
    #[default]
    Primary,
    Secondary,
    Contrast,
}

impl ButtonType {
    fn to_class(&self) -> &'static str {
        ""
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ButtonProps {
    children: Element,
    #[props(optional)]
    on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    button_type: ButtonType,
    #[props(default = false)]
    outline: bool,
    #[props(default = false)]
    disabled: bool,
}

/// A versatile button component.
pub fn Button(props: ButtonProps) -> Element {
    let mut class_parts = Vec::new();

    match props.button_type {
        ButtonType::Primary => class_parts.push("primary"),
        ButtonType::Secondary => class_parts.push("secondary"),
        ButtonType::Contrast => class_parts.push("contrast"),
    }

    if props.outline {
        class_parts.push("outline");
    }

    let class_str = class_parts.join(" ");

    rsx! {
        button {
            class: "{class_str}",
            disabled: props.disabled,
            onclick: move |evt| {
                if let Some(handler) = &props.on_click {
                    handler.call(evt);
                }
            },
            {props.children}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct InputProps {
    label: String,
    name: String,
    value: String,
    #[props(default = "text".to_string())]
    input_type: String,
    #[props(optional)]
    placeholder: Option<String>,
    #[props(default = false)]
    disabled: bool,
    #[props(default = false)]
    readonly: bool,
    #[props(optional)]
    on_input: Option<EventHandler<FormEvent>>,
    #[props(optional)]
    on_click: Option<EventHandler<MouseEvent>>,
    #[props(optional)]
    style: Option<String>,
    // Add the new optional 'min' prop
    #[props(optional, default)]
    min: Option<String>,
}

/// A labeled form input field.
pub fn Input(props: InputProps) -> Element {
    rsx! {
        label {
            "{props.label}",
            input {
                r#type:"{props.input_type}",
                name: "{props.name}",
                placeholder: "{props.placeholder.as_deref().unwrap_or(\"\")}",
                disabled: props.disabled,
                readonly: props.readonly,
                value: "{props.value}",
                style: "{props.style.as_deref().unwrap_or(\"\")}",
                // Apply the new 'min' attribute to the input element
                min: props.min,

                oninput: move |event| {
                    if let Some(handler) = &props.on_input {
                        handler.call(event);
                    }
                },
                onclick: move |evt| {
                    if let Some(handler) = &props.on_click {
                        handler.call(evt);
                    }
                },
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ModalProps {
    is_open: Signal<bool>,
    title: String,
    children: Element,
}

pub fn Modal(mut props: ModalProps) -> Element {
    rsx! {
        if (props.is_open)() {
            dialog {
                open: true,
                article {
                    header {
                        a {
                            href: "#",
                            "aria-label": "Close",
                            class: "close",
                            onclick: move |_| props.is_open.set(false)
                        }
                        h3 { style: "margin-bottom: 0;", "{props.title}" }
                    }
                    {props.children}
                }
            }
        }
    }
}

// A modal with no title bar that closes on backdrop click or Escape key.
#[derive(Props, PartialEq, Clone)]
pub struct NoTitleModalProps {
    is_open: Signal<bool>,
    children: Element,
}

pub fn NoTitleModal(mut props: NoTitleModalProps) -> Element {
    rsx! {
        if (props.is_open)() {
            dialog {
                tabindex: "0",
                open: true,
                autofocus: true,
                onclick: move |_| props.is_open.set(false),
                onkeydown: move |evt| {
                    if evt.key() == Key::Escape {
                        props.is_open.set(false);
                    }
                },
                article {
                    onclick: |evt| evt.stop_propagation(),
                    {props.children}
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct CopyButtonProps {
    /// The string that will be copied to the clipboard when the button is clicked.
    pub text_to_copy: String,
}

/// A button that copies a given text string to the clipboard and displays
/// a "Copied!" confirmation for 5 seconds.
#[allow(non_snake_case)]
pub fn CopyButton(props: CopyButtonProps) -> Element {
    let is_copied = use_signal(|| false);

    rsx! {
        if is_copied() {
            Button {
                button_type: ButtonType::Secondary,
                disabled: true,
                "Copied!"
            }
        } else {
            Button {
                on_click: move |_| {
                    let clipboard_text = props.text_to_copy.clone();

                    spawn({
                        let mut is_copied = is_copied.clone();

                        async move {
                            if crate::compat::clipboard_set(clipboard_text).await {
                                is_copied.set(true);
                                crate::compat::sleep(Duration::from_millis(5000)).await;
                                is_copied.set(false);
                            }
                        }
                    });
                },
                "Copy"
            }
        }
    }
}