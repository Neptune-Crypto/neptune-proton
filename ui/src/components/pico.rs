//! A set of reusable, lifetime-free Dioxus components for the Pico.css framework.
//! To use, ensure you have pico.min.css linked in your main application.

#![allow(non_snake_case)] // Allow PascalCase for component function names

use dioxus::prelude::*;
use dioxus::html::input_data::keyboard_types::Key;

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
    let class_str = if props.outline {
        match props.button_type {
            ButtonType::Primary => "secondary",
            ButtonType::Secondary => "secondary",
            ButtonType::Contrast => "contrast",
        }
    } else {
        props.button_type.to_class()
    };
    rsx! {
        button {
            class: "{class_str}",
            "data-theme": match props.button_type {
                ButtonType::Primary => "primary",
                ButtonType::Secondary => "secondary",
                ButtonType::Contrast => "contrast",
            },
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
    #[props(default = "text".to_string())]
    input_type: String,
    #[props(optional)]
    placeholder: Option<String>,
    #[props(default = false)]
    disabled: bool,
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
                open: true,
                // focus this element as soon as it is rendered into the DOM.
                autofocus: true,
                // Close when the dialog's backdrop is clicked.
                onclick: move |_| props.is_open.set(false),
                // Listen for keyboard events to close on "Escape".
                onkeydown: move |evt| {
                    if evt.key() == Key::Escape {
                        props.is_open.set(false);
                    }
                },
                // The <article> tag holds the content and stops the click
                // from propagating to the backdrop and closing the modal.
                article {
                    onclick: |evt| evt.stop_propagation(),
                    {props.children}
                }
            }
        }
    }
}
